//! Chiffrement asymétrique des données (schéma hybride X25519 via la crate `age`).
//!
//! Modèle de sécurité :
//! - La **clé publique** (recipient `age1...`) suffit pour chiffrer. Elle est stockée en BDD
//!   sur la tâche : le serveur peut donc sauvegarder en automatique (cron) sans jamais pouvoir
//!   déchiffrer.
//! - La **clé privée** (identity `AGE-SECRET-KEY-1...`) n'est **jamais** stockée. Elle est saisie
//!   par l'opérateur au moment de la restauration, gardée en mémoire le temps du run puis effacée.
//!
//! `age` fait un chiffrement hybride : une clé de session symétrique éphémère (ChaCha20-Poly1305,
//! avec authentification/intégrité) est scellée pour le destinataire via X25519. Chaque fichier est
//! chiffré indépendamment en flux (streaming), sans charger le contenu entier en mémoire.

use age::x25519::{Identity, Recipient};
use anyhow::{Context, Result, anyhow};
use std::io::{Read, Write};
use std::str::FromStr;

/// Parse une clé publique age (« recipient », commence par `age1`).
pub fn parse_recipient(s: &str) -> Result<Recipient> {
    Recipient::from_str(s.trim()).map_err(|e| anyhow!("clé publique age invalide : {e}"))
}

/// Parse une clé privée age (« identity », commence par `AGE-SECRET-KEY-1`).
pub fn parse_identity(s: &str) -> Result<Identity> {
    Identity::from_str(s.trim()).map_err(|e| anyhow!("clé privée age invalide : {e}"))
}

/// Chiffre le flux `input` vers `output` pour le destinataire (clé publique).
///
/// Le format de sortie est le format binaire age standard (en-tête + corps chiffré authentifié).
pub fn encrypt<R: Read, W: Write>(recipient: &Recipient, mut input: R, output: W) -> Result<()> {
    let encryptor =
        age::Encryptor::with_recipients(std::iter::once(recipient as &dyn age::Recipient))
            .context("création de l'encrypteur age")?;
    let mut writer = encryptor
        .wrap_output(output)
        .context("initialisation du flux chiffré")?;
    std::io::copy(&mut input, &mut writer).context("écriture du flux à chiffrer")?;
    writer.finish().context("finalisation du flux chiffré")?;
    Ok(())
}

/// Déchiffre le flux `input` vers `output` avec la clé privée (identity).
///
/// Échoue si la clé privée ne correspond pas au destinataire, ou si le flux a été altéré
/// (l'authentification AEAD de age garantit l'intégrité).
pub fn decrypt<R: Read, W: Write>(identity: &Identity, input: R, mut output: W) -> Result<()> {
    let decryptor = age::Decryptor::new(input).context("lecture de l'en-tête age")?;
    let mut reader = decryptor
        .decrypt(std::iter::once(identity as &dyn age::Identity))
        .context("déchiffrement impossible (clé privée incorrecte ou flux corrompu)")?;
    std::io::copy(&mut reader, &mut output).context("écriture du flux déchiffré")?;
    Ok(())
}

/// Génère une paire de clés age `(publique, privée)`.
///
/// En production la génération se fait **côté navigateur** (la clé privée n'atteint jamais le
/// serveur) ; cette fonction sert surtout aux tests et à un éventuel usage CLI.
#[cfg(test)]
pub fn generate_keypair() -> (String, String) {
    use age::secrecy::ExposeSecret;
    let id = Identity::generate();
    let public = id.to_public().to_string();
    let private = id.to_string().expose_secret().to_string();
    (public, private)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_in_memory() {
        let id = Identity::generate();
        let recipient = id.to_public();
        let plaintext = b"Donnees confidentielles a chiffrer";

        let mut ciphertext = Vec::new();
        encrypt(&recipient, &plaintext[..], &mut ciphertext).unwrap();
        assert_ne!(&ciphertext[..], &plaintext[..]);

        let mut decrypted = Vec::new();
        decrypt(&id, &ciphertext[..], &mut decrypted).unwrap();
        assert_eq!(&decrypted[..], &plaintext[..]);
    }

    #[test]
    fn roundtrip_via_key_strings() {
        let (public, private) = generate_keypair();
        let recipient = parse_recipient(&public).unwrap();
        let identity = parse_identity(&private).unwrap();
        let plaintext = b"payload via cles serialisees";

        let mut ciphertext = Vec::new();
        encrypt(&recipient, &plaintext[..], &mut ciphertext).unwrap();
        let mut decrypted = Vec::new();
        decrypt(&identity, &ciphertext[..], &mut decrypted).unwrap();
        assert_eq!(&decrypted[..], &plaintext[..]);
    }

    #[test]
    fn wrong_private_key_is_rejected() {
        let id1 = Identity::generate();
        let id2 = Identity::generate();

        let mut ciphertext = Vec::new();
        encrypt(&id1.to_public(), &b"secret"[..], &mut ciphertext).unwrap();

        let mut out = Vec::new();
        assert!(decrypt(&id2, &ciphertext[..], &mut out).is_err());
    }

    #[test]
    fn invalid_key_strings_are_rejected() {
        assert!(parse_recipient("pas-une-cle").is_err());
        assert!(parse_identity("AGE-SECRET-KEY-INVALIDE").is_err());
    }
}
