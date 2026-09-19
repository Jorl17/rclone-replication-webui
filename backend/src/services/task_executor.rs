use crate::{
    entities::{encrypted_file_state, notification_channel, remote, task, task_run},
    errors::{AppError, AppResult},
    models::remote::RcloneRemote,
    models::notification::{ChannelTemplates, render_channel_event},
    services::{
        apprise, crypto, crypto_transfer,
        notification_message::{NotificationContext, NotificationEvent},
        rclone,
    },
    sse::broadcaster::SseEvent,
    sse::global::GlobalEvent,
    state::{AppState, RunningTask},
};
use chrono::Utc;
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Notify;
use uuid::Uuid;

/// Concurrence (nombre de fichiers chiffrés/déchiffrés en parallèle) pour le pipeline chiffré.
const ENCRYPT_CONCURRENCY: usize = 4;

/// Paramètres spécifiques à une restauration.
pub struct RestoreParams {
    /// Clé privée age saisie par l'opérateur. Jamais stockée ; effacée de la mémoire en fin de run.
    pub private_key: Option<zeroize::Zeroizing<String>>,
    /// Où écrire les données restaurées (par défaut : la source d'origine de la tâche).
    pub target_remote_id: Uuid,
    pub target_path: String,
}

pub enum ExecutionMode {
    Manual,
    Scheduled,
    Restore(RestoreParams),
}

impl ExecutionMode {
    pub fn label(&self) -> &'static str {
        match self {
            ExecutionMode::Manual => "manual",
            ExecutionMode::Scheduled => "scheduler",
            ExecutionMode::Restore(_) => "restore",
        }
    }
}

struct TaskSnapshot {
    source_remote_id: Uuid,
    source_path: String,
    dest_remote_id: Uuid,
    dest_path: String,
    rclone_flags: Vec<String>,
    notification_channel_id: Option<Uuid>,
    notify_on: Vec<String>,
    max_retries: i32,
    retry_delay_seconds: i32,
    encryption_enabled: bool,
    encryption_public_key: Option<String>,
}

pub async fn spawn_task(state: AppState, task_id: Uuid, mode: ExecutionMode) -> AppResult<Uuid> {
    if state.running_tasks.contains_key(&task_id) {
        return Err(AppError::AlreadyRunning);
    }

    let t = task::Entity::find_by_id(task_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Task {task_id} not found")))?;

    let snapshot = TaskSnapshot {
        source_remote_id: t.source_remote_id,
        source_path: t.source_path,
        dest_remote_id: t.dest_remote_id,
        dest_path: t.dest_path,
        rclone_flags: t.rclone_flags,
        notification_channel_id: t.notification_channel_id,
        notify_on: t.notify_on,
        max_retries: t.max_retries,
        retry_delay_seconds: t.retry_delay_seconds,
        encryption_enabled: t.encryption_enabled,
        encryption_public_key: t.encryption_public_key,
    };

    let run = task_run::ActiveModel {
        id: Set(Uuid::new_v4()),
        task_id: Set(task_id),
        triggered_by: Set(mode.label().to_string()),
        status: Set("running".to_string()),
        started_at: Set(Utc::now().into()),
        ..Default::default()
    };
    let run_model = run.insert(&state.db).await.map_err(AppError::Database)?;
    let run_id = run_model.id;

    state.running_tasks.insert(
        task_id,
        RunningTask {
            run_id,
            triggered_by: mode.label().to_string(),
            started_at: Utc::now(),
            log_buffer: Arc::new(StdMutex::new(Vec::new())),
            log_notify: Arc::new(Notify::new()),
        },
    );

    state
        .global_broadcaster
        .publish(GlobalEvent::TaskStarted { task_id });

    tokio::spawn(async move {
        execute_task_background(state, task_id, run_id, snapshot, mode).await;
    });

    Ok(run_id)
}

async fn execute_task_background(
    state: AppState,
    task_id: Uuid,
    run_id: Uuid,
    snapshot: TaskSnapshot,
    mode: ExecutionMode,
) {
    let started_at = Utc::now();

    let remotes = match load_remotes(&state).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to load remotes for task {task_id}: {e:#}");
            finish_run(
                &state,
                task_id,
                run_id,
                started_at,
                1,
                format!("Error: {e:#}"),
                None,
            )
            .await;
            return;
        }
    };

    // Résolution de l'opération selon le mode :
    // - Restauration : on lit la destination de la tâche et on écrit vers la cible choisie.
    // - Sauvegarde (Manual/Scheduled) : source → destination de la tâche.
    let (op_src_remote_id, op_src_path, op_dst_remote_id, op_dst_path) = match &mode {
        ExecutionMode::Restore(p) => (
            snapshot.dest_remote_id,
            snapshot.dest_path.clone(),
            p.target_remote_id,
            p.target_path.clone(),
        ),
        _ => (
            snapshot.source_remote_id,
            snapshot.source_path.clone(),
            snapshot.dest_remote_id,
            snapshot.dest_path.clone(),
        ),
    };

    let src_remote = remotes.iter().find(|r| r.id == op_src_remote_id);
    let dst_remote = remotes.iter().find(|r| r.id == op_dst_remote_id);
    let (src_remote, dst_remote) = match (src_remote, dst_remote) {
        (Some(s), Some(d)) => (s, d),
        _ => {
            let msg = "Source or destination remote not found".to_string();
            tracing::error!("{msg} for task {task_id}");
            finish_run(&state, task_id, run_id, started_at, 1, msg, None).await;
            return;
        }
    };

    let src = build_rclone_path(src_remote, &op_src_path);
    let dst = build_rclone_path(dst_remote, &op_dst_path);

    // Buffers partagés (pour les SSE qui se connectent en cours d'exécution).
    let (shared_logs, log_notify) = state
        .running_tasks
        .get(&task_id)
        .map(|rt| (rt.log_buffer.clone(), rt.log_notify.clone()))
        .expect("running task entry missing");

    let mut log_buffer = String::new();

    let (exit_code, stats) = if snapshot.encryption_enabled {
        match &mode {
            ExecutionMode::Restore(p) => {
                run_encrypted_restore(
                    &state,
                    task_id,
                    p.private_key.as_ref().map(|z| z.as_str()),
                    &remotes,
                    &src,
                    &dst,
                    &snapshot.rclone_flags,
                    &mut log_buffer,
                    &shared_logs,
                    &log_notify,
                )
                .await
            }
            _ => {
                run_encrypted_backup(
                    &state,
                    task_id,
                    snapshot.encryption_public_key.as_deref(),
                    &remotes,
                    &src,
                    &dst,
                    &snapshot.rclone_flags,
                    &mut log_buffer,
                    &shared_logs,
                    &log_notify,
                )
                .await
            }
        }
    } else {
        run_plain(
            &state,
            task_id,
            run_id,
            &snapshot,
            &remotes,
            &src,
            &dst,
            &mut log_buffer,
            &shared_logs,
            &log_notify,
        )
        .await
    };

    let error_logs = if exit_code != 0 {
        extract_readable_logs(&log_buffer)
    } else {
        String::new()
    };
    finish_run(
        &state, task_id, run_id, started_at, exit_code, log_buffer, stats,
    )
    .await;

    // Notification uniquement à la fin (après tous les retries / le pipeline complet)
    if let Some(channel_id) = snapshot.notification_channel_id {
        let should_notify = if exit_code != 0 {
            snapshot.notify_on.iter().any(|n| n == "error")
        } else {
            snapshot.notify_on.iter().any(|n| n == "success")
        };

        if should_notify {
            let event = if exit_code != 0 {
                NotificationEvent::Error
            } else {
                NotificationEvent::Success
            };
            let ctx = NotificationContext::for_run(task_id, run_id, Some(exit_code), &error_logs);
            send_run_notification(&state, channel_id, event, &ctx).await;
        }
    }
}

/// Construit le chemin rclone (`remote:chemin`) en concaténant la racine du remote (clé `root`
/// gérée par l'application) au chemin de la tâche.
fn build_rclone_path(remote: &RcloneRemote, task_path: &str) -> String {
    let root = remote
        .config
        .as_object()
        .and_then(|o| o.get("root"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let full_path = if root.is_empty() {
        task_path.to_string()
    } else {
        format!(
            "{}/{}",
            root.trim_end_matches('/'),
            task_path.trim_start_matches('/')
        )
    };
    format!("{}:{}", remote.name, full_path)
}

/// Concatène un chemin relatif à un chemin rclone de base, en gérant proprement le séparateur
/// (pas de slash juste après `remote:`).
fn join_remote_path(base: &str, rel: &str) -> String {
    let rel = rel.trim_start_matches('/');
    if base.ends_with(':') || base.ends_with('/') {
        format!("{base}{rel}")
    } else {
        format!("{base}/{rel}")
    }
}

/// Pousse une ligne dans le buffer local (pour la BDD) + le buffer partagé (pour les SSE live).
fn push_log(
    log_buffer: &mut String,
    shared_logs: &Arc<StdMutex<Vec<String>>>,
    log_notify: &Arc<Notify>,
    line: &str,
) {
    log_buffer.push_str(line);
    log_buffer.push('\n');
    {
        let mut s = shared_logs.lock().unwrap_or_else(|p| p.into_inner());
        s.push(line.to_string());
    }
    log_notify.notify_waiters();
}

/// Synthétise un objet `stats` compatible avec le format rclone (consommé tel quel par le front).
fn encrypted_stats(
    transfers: usize,
    total: usize,
    checks: usize,
    deletes: usize,
    errors: usize,
    bytes: i64,
) -> serde_json::Value {
    serde_json::json!({
        "bytes": bytes,
        "totalBytes": bytes,
        "checks": checks,
        "totalChecks": checks,
        "transfers": transfers,
        "totalTransfers": total,
        "deletes": deletes,
        "deletedDirs": 0,
        "errors": errors,
        "fatalError": false,
        "renames": 0,
        "retryError": false,
        "speed": 0.0,
        "elapsedTime": 0.0,
        "transferTime": 0.0,
    })
}

/// Met à jour le manifeste pour un fichier source chiffré avec succès.
async fn upsert_manifest(db: &DatabaseConnection, task_id: Uuid, file: &rclone::RcloneFile) {
    let am = encrypted_file_state::ActiveModel {
        id: Set(Uuid::new_v4()),
        task_id: Set(task_id),
        src_path: Set(file.path.clone()),
        size: Set(file.size),
        modtime: Set(file.mod_time.clone()),
        updated_at: Set(Utc::now().into()),
    };
    let res = encrypted_file_state::Entity::insert(am)
        .on_conflict(
            OnConflict::columns([
                encrypted_file_state::Column::TaskId,
                encrypted_file_state::Column::SrcPath,
            ])
            .update_columns([
                encrypted_file_state::Column::Size,
                encrypted_file_state::Column::Modtime,
                encrypted_file_state::Column::UpdatedAt,
            ])
            .to_owned(),
        )
        .exec(db)
        .await;
    if let Err(e) = res {
        tracing::error!("manifest upsert failed for {}: {e}", file.path);
    }
}

/// Exécution classique (non chiffrée) : `rclone sync` avec retry à backoff linéaire.
#[allow(clippy::too_many_arguments)]
async fn run_plain(
    state: &AppState,
    task_id: Uuid,
    run_id: Uuid,
    snapshot: &TaskSnapshot,
    remotes: &[RcloneRemote],
    src: &str,
    dst: &str,
    log_buffer: &mut String,
    shared_logs: &Arc<StdMutex<Vec<String>>>,
    log_notify: &Arc<Notify>,
) -> (i32, Option<serde_json::Value>) {
    let max_attempts = (snapshot.max_retries + 1).max(1) as u32; // au moins 1 tentative
    let base_delay = snapshot.retry_delay_seconds.max(1) as u64;
    let mut exit_code = 1i32;
    let mut stats: Option<serde_json::Value> = None;

    for attempt in 1..=max_attempts {
        if attempt > 1 {
            let delay_secs = base_delay * (attempt as u64 - 1);
            let msg = format!("--- Tentative {attempt}/{max_attempts} dans {delay_secs}s ---");
            tracing::info!("Task {task_id}: {msg}");
            push_log(log_buffer, shared_logs, log_notify, &msg);
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
        }

        tracing::info!(
            "Starting rclone sync: {src} -> {dst} (run {run_id}, attempt {attempt}/{max_attempts})"
        );

        let process = match rclone::spawn_sync(
            &state.config.rclone_bin,
            remotes,
            src,
            dst,
            &snapshot.rclone_flags,
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                let msg = format!("Error: {e:#}");
                tracing::error!("Failed to spawn rclone for task {task_id}: {e:#}");
                push_log(log_buffer, shared_logs, log_notify, &msg);
                exit_code = 1;
                continue; // retry
            }
        };

        exit_code = match rclone::stream_output(process, |line| {
            push_log(log_buffer, shared_logs, log_notify, &line);
        })
        .await
        {
            Ok(s) => s.code().unwrap_or(-1),
            Err(e) => {
                tracing::error!("rclone stream error for task {task_id}: {e:#}");
                1
            }
        };

        stats = extract_rclone_stats(&*log_buffer);

        if exit_code == 0 {
            break; // succès, pas besoin de retry
        }

        if attempt < max_attempts {
            let msg = format!("--- Échec (code {exit_code}), nouvelle tentative... ---");
            push_log(log_buffer, shared_logs, log_notify, &msg);
        }
    }

    (exit_code, stats)
}

/// Sauvegarde chiffrée incrémentale : ne (re)chiffre que les fichiers source nouveaux/modifiés,
/// supprime côté destination les `.age` dont la source a disparu. Le serveur n'utilise que la
/// clé publique : il ne peut pas déchiffrer.
#[allow(clippy::too_many_arguments)]
async fn run_encrypted_backup(
    state: &AppState,
    task_id: Uuid,
    public_key: Option<&str>,
    remotes: &[RcloneRemote],
    src_base: &str,
    dst_base: &str,
    rclone_flags: &[String],
    log_buffer: &mut String,
    shared_logs: &Arc<StdMutex<Vec<String>>>,
    log_notify: &Arc<Notify>,
) -> (i32, Option<serde_json::Value>) {
    // 1. Clé publique (recipient age)
    let recipient = match public_key.map(crypto::parse_recipient) {
        Some(Ok(r)) => r,
        Some(Err(e)) => {
            push_log(
                log_buffer,
                shared_logs,
                log_notify,
                &format!("Erreur : {e:#}"),
            );
            return (1, None);
        }
        None => {
            push_log(
                log_buffer,
                shared_logs,
                log_notify,
                "Erreur : chiffrement activé mais aucune clé publique configurée",
            );
            return (1, None);
        }
    };

    // 2. Config rclone (écrite une fois, réutilisée pour tous les fichiers)
    let config_path = match rclone::write_rclone_config(remotes) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("rclone config write failed: {e:#}");
            push_log(
                log_buffer,
                shared_logs,
                log_notify,
                &format!("Erreur de configuration rclone : {e:#}"),
            );
            return (1, None);
        }
    };
    let _cleanup = ConfigGuard(config_path.clone());
    let rclone_bin = state.config.rclone_bin.clone();

    // 3. Listing de la source
    push_log(
        log_buffer,
        shared_logs,
        log_notify,
        &format!("Sauvegarde chiffrée : {src_base} → {dst_base}"),
    );
    push_log(
        log_buffer,
        shared_logs,
        log_notify,
        "Listing de la source...",
    );
    let source_files = match rclone::lsjson(&rclone_bin, &config_path, src_base, rclone_flags).await
    {
        Ok(f) => f,
        Err(e) => {
            push_log(
                log_buffer,
                shared_logs,
                log_notify,
                &format!("Erreur de listing de la source : {e:#}"),
            );
            return (1, None);
        }
    };

    // 4. Manifeste existant
    let manifest = encrypted_file_state::Entity::find()
        .filter(encrypted_file_state::Column::TaskId.eq(task_id))
        .all(&state.db)
        .await
        .unwrap_or_default();
    let manifest_map: HashMap<&str, (i64, &str)> = manifest
        .iter()
        .map(|m| (m.src_path.as_str(), (m.size, m.modtime.as_str())))
        .collect();
    let source_set: HashSet<&str> = source_files.iter().map(|f| f.path.as_str()).collect();

    // 5. Diff (nouveaux/modifiés à chiffrer, disparus à supprimer)
    let mut to_upload = Vec::new();
    let mut unchanged = 0usize;
    for f in &source_files {
        match manifest_map.get(f.path.as_str()) {
            Some((sz, mt)) if *sz == f.size && *mt == f.mod_time => unchanged += 1,
            _ => to_upload.push(f.clone()),
        }
    }
    let to_delete: Vec<String> = manifest
        .iter()
        .filter(|m| !source_set.contains(m.src_path.as_str()))
        .map(|m| m.src_path.clone())
        .collect();

    let total = to_upload.len();
    push_log(
        log_buffer,
        shared_logs,
        log_notify,
        &format!(
            "Chiffrement : {total} fichier(s) à traiter, {unchanged} inchangé(s), {} à supprimer",
            to_delete.len()
        ),
    );

    // 6. Chiffrement + upload (concurrence bornée)
    let mut uploaded = 0usize;
    let mut bytes = 0i64;
    let mut errors = 0usize;
    let mut deletes = 0usize;

    let mut files_iter = to_upload.into_iter();
    let mut join_set: tokio::task::JoinSet<(rclone::RcloneFile, Result<(), String>)> =
        tokio::task::JoinSet::new();

    // Lance le chiffrement d'un fichier dans un thread bloquant (clone les ressources par fichier).
    let spawn_one = |js: &mut tokio::task::JoinSet<(rclone::RcloneFile, Result<(), String>)>,
                     file: rclone::RcloneFile| {
        let rclone_bin = rclone_bin.clone();
        let cfg = config_path.clone();
        let recipient = recipient.clone();
        let flags = rclone_flags.to_vec();
        let src_file = join_remote_path(src_base, &file.path);
        let dst_file = join_remote_path(dst_base, &format!("{}.age", file.path));
        js.spawn_blocking(move || {
            let res = crypto_transfer::encrypt_file(
                &rclone_bin,
                &cfg,
                &src_file,
                &dst_file,
                &recipient,
                &flags,
            )
            .map_err(|e| e.to_string());
            (file, res)
        });
    };

    // Amorce le pipeline, puis draine en réalimentant : concurrence bornée + logs en temps réel.
    for _ in 0..ENCRYPT_CONCURRENCY {
        match files_iter.next() {
            Some(file) => spawn_one(&mut join_set, file),
            None => break,
        }
    }

    let mut done = 0usize;
    while let Some(joined) = join_set.join_next().await {
        done += 1;
        match joined {
            Ok((file, Ok(()))) => {
                uploaded += 1;
                bytes += file.size.max(0);
                push_log(
                    log_buffer,
                    shared_logs,
                    log_notify,
                    &format!("[{done}/{total}] chiffré : {}", file.path),
                );
                upsert_manifest(&state.db, task_id, &file).await;
            }
            Ok((file, Err(e))) => {
                errors += 1;
                push_log(
                    log_buffer,
                    shared_logs,
                    log_notify,
                    &format!("[{done}/{total}] ECHEC {} : {e}", file.path),
                );
                // Nettoyage best-effort du `.age` partiel pour ne pas laisser un fichier corrompu.
                let dst_file = join_remote_path(dst_base, &format!("{}.age", file.path));
                let _ =
                    rclone::deletefile(&rclone_bin, &config_path, &dst_file, rclone_flags).await;
            }
            Err(join_err) => {
                errors += 1;
                push_log(
                    log_buffer,
                    shared_logs,
                    log_notify,
                    &format!("[{done}/{total}] ECHEC interne : {join_err}"),
                );
            }
        }
        // Réalimente le pipeline avec le fichier suivant.
        if let Some(file) = files_iter.next() {
            spawn_one(&mut join_set, file);
        }
    }

    // 7. Suppressions (fichiers disparus de la source)
    for path in &to_delete {
        let dst_file = join_remote_path(dst_base, &format!("{path}.age"));
        match rclone::deletefile(&rclone_bin, &config_path, &dst_file, rclone_flags).await {
            Ok(()) => {
                deletes += 1;
                push_log(
                    log_buffer,
                    shared_logs,
                    log_notify,
                    &format!("supprimé : {path}"),
                );
                let _ = encrypted_file_state::Entity::delete_many()
                    .filter(encrypted_file_state::Column::TaskId.eq(task_id))
                    .filter(encrypted_file_state::Column::SrcPath.eq(path.clone()))
                    .exec(&state.db)
                    .await;
            }
            Err(e) => {
                errors += 1;
                push_log(
                    log_buffer,
                    shared_logs,
                    log_notify,
                    &format!("ECHEC suppression {path} : {e:#}"),
                );
            }
        }
    }

    push_log(
        log_buffer,
        shared_logs,
        log_notify,
        &format!(
            "Terminé : {uploaded} chiffré(s), {unchanged} inchangé(s), {deletes} supprimé(s), {errors} erreur(s)"
        ),
    );

    let exit_code = if errors == 0 { 0 } else { 1 };
    (
        exit_code,
        Some(encrypted_stats(
            uploaded, total, unchanged, deletes, errors, bytes,
        )),
    )
}

/// Restauration d'une destination chiffrée : déchiffre chaque `.age` vers la cible choisie.
/// Nécessite la clé privée (saisie par l'opérateur, jamais stockée).
#[allow(clippy::too_many_arguments)]
async fn run_encrypted_restore(
    state: &AppState,
    _task_id: Uuid,
    private_key: Option<&str>,
    remotes: &[RcloneRemote],
    enc_base: &str,
    target_base: &str,
    rclone_flags: &[String],
    log_buffer: &mut String,
    shared_logs: &Arc<StdMutex<Vec<String>>>,
    log_notify: &Arc<Notify>,
) -> (i32, Option<serde_json::Value>) {
    // 1. Clé privée (identity age)
    let identity = match private_key.map(crypto::parse_identity) {
        Some(Ok(id)) => id,
        Some(Err(e)) => {
            push_log(
                log_buffer,
                shared_logs,
                log_notify,
                &format!("Erreur : {e:#}"),
            );
            return (1, None);
        }
        None => {
            push_log(
                log_buffer,
                shared_logs,
                log_notify,
                "Erreur : restauration d'une destination chiffrée sans clé privée",
            );
            return (1, None);
        }
    };

    let config_path = match rclone::write_rclone_config(remotes) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("rclone config write failed: {e:#}");
            push_log(
                log_buffer,
                shared_logs,
                log_notify,
                &format!("Erreur de configuration rclone : {e:#}"),
            );
            return (1, None);
        }
    };
    let _cleanup = ConfigGuard(config_path.clone());
    let rclone_bin = state.config.rclone_bin.clone();

    push_log(
        log_buffer,
        shared_logs,
        log_notify,
        &format!("Restauration chiffrée : {enc_base} → {target_base}"),
    );
    push_log(
        log_buffer,
        shared_logs,
        log_notify,
        "Listing de la destination chiffrée...",
    );
    let enc_files: Vec<rclone::RcloneFile> =
        match rclone::lsjson(&rclone_bin, &config_path, enc_base, rclone_flags).await {
            Ok(f) => f.into_iter().filter(|f| f.path.ends_with(".age")).collect(),
            Err(e) => {
                push_log(
                    log_buffer,
                    shared_logs,
                    log_notify,
                    &format!("Erreur de listing : {e:#}"),
                );
                return (1, None);
            }
        };

    let total = enc_files.len();
    push_log(
        log_buffer,
        shared_logs,
        log_notify,
        &format!("Restauration : {total} fichier(s) chiffré(s) à déchiffrer"),
    );

    let mut restored = 0usize;
    let mut bytes = 0i64;
    let mut errors = 0usize;

    let mut files_iter = enc_files.into_iter();
    let mut join_set: tokio::task::JoinSet<(rclone::RcloneFile, Result<(), String>)> =
        tokio::task::JoinSet::new();

    // Lance le déchiffrement d'un fichier dans un thread bloquant.
    let spawn_one = |js: &mut tokio::task::JoinSet<(rclone::RcloneFile, Result<(), String>)>,
                     file: rclone::RcloneFile| {
        let rclone_bin = rclone_bin.clone();
        let cfg = config_path.clone();
        let identity = identity.clone();
        let flags = rclone_flags.to_vec();
        let enc_file = join_remote_path(enc_base, &file.path);
        let plain_rel = file
            .path
            .strip_suffix(".age")
            .unwrap_or(&file.path)
            .to_string();
        let dst_file = join_remote_path(target_base, &plain_rel);
        js.spawn_blocking(move || {
            let res = crypto_transfer::decrypt_file(
                &rclone_bin,
                &cfg,
                &enc_file,
                &dst_file,
                &identity,
                &flags,
            )
            .map_err(|e| e.to_string());
            (file, res)
        });
    };

    for _ in 0..ENCRYPT_CONCURRENCY {
        match files_iter.next() {
            Some(file) => spawn_one(&mut join_set, file),
            None => break,
        }
    }

    let mut done = 0usize;
    while let Some(joined) = join_set.join_next().await {
        done += 1;
        match joined {
            Ok((file, Ok(()))) => {
                restored += 1;
                bytes += file.size.max(0);
                push_log(
                    log_buffer,
                    shared_logs,
                    log_notify,
                    &format!("[{done}/{total}] déchiffré : {}", file.path),
                );
            }
            Ok((file, Err(e))) => {
                errors += 1;
                push_log(
                    log_buffer,
                    shared_logs,
                    log_notify,
                    &format!("[{done}/{total}] ECHEC {} : {e}", file.path),
                );
            }
            Err(join_err) => {
                errors += 1;
                push_log(
                    log_buffer,
                    shared_logs,
                    log_notify,
                    &format!("[{done}/{total}] ECHEC interne : {join_err}"),
                );
            }
        }
        if let Some(file) = files_iter.next() {
            spawn_one(&mut join_set, file);
        }
    }

    push_log(
        log_buffer,
        shared_logs,
        log_notify,
        &format!("Terminé : {restored} déchiffré(s), {errors} erreur(s)"),
    );

    let exit_code = if errors == 0 { 0 } else { 1 };
    (
        exit_code,
        Some(encrypted_stats(restored, total, 0, 0, errors, bytes)),
    )
}

/// Extrait les lignes lisibles des logs rclone pour inclusion dans les notifications.
fn extract_readable_logs(log_output: &str) -> String {
    let mut lines = Vec::new();
    for raw in log_output.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('{') {
            if let Ok(obj) = serde_json::from_str::<serde_json::Value>(trimmed) {
                let level = obj.get("level").and_then(|v| v.as_str()).unwrap_or("");
                let msg = obj.get("msg").and_then(|v| v.as_str()).unwrap_or("").trim();
                if msg.is_empty() {
                    continue;
                }
                if level == "error" {
                    lines.push(format!("[ERROR] {msg}"));
                } else {
                    lines.push(msg.to_string());
                }
            }
        } else {
            lines.push(trimmed.to_string());
        }
    }
    lines.join("\n")
}

/// Parcourt les lignes du log rclone (JSON) depuis la fin pour trouver la dernière
/// ligne contenant un objet "stats" et l'extrait.
fn extract_rclone_stats(log_output: &str) -> Option<serde_json::Value> {
    for line in log_output.lines().rev() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line)
            && obj.get("stats").is_some()
        {
            return obj.get("stats").cloned();
        }
    }
    None
}

async fn finish_run(
    state: &AppState,
    task_id: Uuid,
    run_id: Uuid,
    started_at: chrono::DateTime<Utc>,
    exit_code: i32,
    log_output: String,
    stats: Option<serde_json::Value>,
) {
    let finished_at = Utc::now();
    let duration_ms = (finished_at - started_at).num_milliseconds();
    let status = if exit_code == 0 { "success" } else { "failure" };

    let update_result = task_run::Entity::update_many()
        .col_expr(task_run::Column::Status, Expr::value(status))
        .col_expr(
            task_run::Column::FinishedAt,
            Expr::value(chrono::DateTime::<chrono::FixedOffset>::from(finished_at)),
        )
        .col_expr(task_run::Column::DurationMs, Expr::value(duration_ms))
        .col_expr(task_run::Column::ExitCode, Expr::value(exit_code))
        .col_expr(task_run::Column::LogOutput, Expr::value(log_output))
        .col_expr(task_run::Column::Stats, Expr::value(stats))
        .filter(task_run::Column::Id.eq(run_id))
        .exec(&state.db)
        .await;

    if let Err(e) = update_result {
        tracing::error!("Failed to update run {run_id}: {e:#}");
    }

    state.sse_broadcaster.publish(
        task_id,
        SseEvent::Done {
            status: status.to_string(),
            exit_code: Some(exit_code),
            duration_ms,
        },
    );
    state.sse_broadcaster.close(&task_id);

    state.global_broadcaster.publish(GlobalEvent::TaskFinished {
        task_id,
        status: status.to_string(),
    });

    state.running_tasks.remove(&task_id);

    tracing::info!("Run {run_id} finished status={status} in {duration_ms}ms");
}

async fn send_run_notification(
    state: &AppState,
    channel_id: Uuid,
    event: NotificationEvent,
    ctx: &NotificationContext,
) {
    let channel = match notification_channel::Entity::find_by_id(channel_id)
        .filter(notification_channel::Column::Enabled.eq(true))
        .one(&state.db)
        .await
    {
        Ok(Some(ch)) => ch,
        _ => return,
    };

    let templates = ChannelTemplates::from_json(&channel.templates);
    let message = render_channel_event(&channel.language, &templates, event, ctx);
    let _ = apprise::send_notification(
        &state.config.apprise_bin,
        &[channel.apprise_url],
        &message.subject,
        &message.body,
    )
    .await;
}

async fn load_remotes(state: &AppState) -> anyhow::Result<Vec<RcloneRemote>> {
    let remotes = remote::Entity::find().all(&state.db).await?;

    let mut result = Vec::with_capacity(remotes.len());
    for r in remotes {
        let mut config = r.config;
        // Fusionner les secrets stockés dans le SecretStore (Scaleway SM ou autre)
        if state.secret_store.is_active()
            && let Some(stored) = state.secret_store.get(r.id).await?
            && let Some(obj) = config.as_object_mut()
        {
            for (k, v) in stored {
                obj.insert(k, serde_json::Value::String(v));
            }
        }
        result.push(RcloneRemote {
            id: r.id,
            name: r.name,
            remote_type: r.remote_type,
            config,
        });
    }
    Ok(result)
}

/// Guard RAII : supprime le fichier de config rclone temporaire en fin de pipeline chiffré.
struct ConfigGuard(std::path::PathBuf);
impl Drop for ConfigGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
