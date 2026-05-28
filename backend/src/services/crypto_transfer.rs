//! Transfert chiffré fichier par fichier : pont entre rclone (transport) et age (chiffrement).
//!
//! Ces fonctions sont **synchrones** et bloquantes (elles pilotent des processus rclone via des
//! pipes std) : elles doivent être appelées depuis `tokio::task::spawn_blocking`.
//!
//! Principe (sauvegarde) :
//! ```text
//! rclone cat SRC:fichier  →  age(clé publique)  →  rclone rcat DST:fichier.age
//! ```
//! Restauration : on inverse (`rclone cat DST:fichier.age → age -d(clé privée) → rclone rcat CIBLE:fichier`).

use crate::services::crypto;
use age::x25519::{Identity, Recipient};
use anyhow::{Context, Result, anyhow};
use std::io::Read;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// Draine le stderr d'un enfant dans un thread dédié, pour éviter qu'un pipe stderr plein
/// ne bloque le processus pendant qu'on lit/écrit son stdout/stdin.
fn drain_stderr(child: &mut Child) -> std::thread::JoinHandle<String> {
    let stderr = child.stderr.take();
    std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(mut s) = stderr {
            let _ = s.read_to_string(&mut buf);
        }
        buf
    })
}

/// Branche `rclone cat src_file` → `transform` → `rclone rcat dst_file`.
///
/// `transform` reçoit le flux clair/chiffré en lecture (stdout de `cat`) et le writer vers
/// `rcat` (stdin) ; il consomme le writer (le ferme) pour signaler EOF à `rcat`.
fn run_pipe<F>(
    rclone_bin: &str,
    config_path: &Path,
    src_file: &str,
    dst_file: &str,
    transform: F,
) -> Result<()>
where
    F: FnOnce(&mut ChildStdout, ChildStdin) -> Result<()>,
{
    let cfg = config_path
        .to_str()
        .ok_or_else(|| anyhow!("chemin config non-UTF8"))?;

    let mut cat = Command::new(rclone_bin)
        .args(["--config", cfg, "cat", src_file])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("lancement de rclone cat ({src_file})"))?;

    let mut rcat = Command::new(rclone_bin)
        .args(["--config", cfg, "rcat", dst_file])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("lancement de rclone rcat ({dst_file})"))?;

    let mut cat_out = cat.stdout.take().expect("stdout de cat piped");
    let rcat_in = rcat.stdin.take().expect("stdin de rcat piped");
    let cat_err = drain_stderr(&mut cat);
    let rcat_err = drain_stderr(&mut rcat);

    // Pipe : cat.stdout → transform (age) → rcat.stdin. `transform` ferme rcat_in en sortie.
    let transform_result = transform(&mut cat_out, rcat_in);

    let cat_status = cat.wait().context("attente de rclone cat")?;
    let rcat_status = rcat.wait().context("attente de rclone rcat")?;
    let cat_err = cat_err.join().unwrap_or_default();
    let rcat_err = rcat_err.join().unwrap_or_default();

    transform_result.context("transformation du flux (chiffrement/déchiffrement)")?;

    if !cat_status.success() {
        anyhow::bail!("lecture source ({src_file}) échouée : {}", cat_err.trim());
    }
    if !rcat_status.success() {
        anyhow::bail!(
            "écriture destination ({dst_file}) échouée : {}",
            rcat_err.trim()
        );
    }
    Ok(())
}

/// Chiffre un fichier : `rclone cat src` → age(recipient) → `rclone rcat dst`.
pub fn encrypt_file(
    rclone_bin: &str,
    config_path: &Path,
    src_remote_file: &str,
    dst_remote_file: &str,
    recipient: &Recipient,
) -> Result<()> {
    run_pipe(
        rclone_bin,
        config_path,
        src_remote_file,
        dst_remote_file,
        |reader, writer| crypto::encrypt(recipient, reader, writer),
    )
}

/// Déchiffre un fichier : `rclone cat enc` → age -d(identity) → `rclone rcat dst`.
pub fn decrypt_file(
    rclone_bin: &str,
    config_path: &Path,
    enc_remote_file: &str,
    dst_remote_file: &str,
    identity: &Identity,
) -> Result<()> {
    run_pipe(
        rclone_bin,
        config_path,
        enc_remote_file,
        dst_remote_file,
        |reader, writer| crypto::decrypt(identity, reader, writer),
    )
}
