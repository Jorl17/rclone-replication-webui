use crate::{errors::AppResult, sse::broadcaster::SseEvent, state::AppState};
use async_stream::stream;
use axum::{
    extract::{Path, State},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures::Stream;
use serde_json::json;
use std::{convert::Infallible, pin::Pin};
use uuid::Uuid;

type SseStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;

/// Streame la progression d'une tâche en cours en SSE.
///
/// Comportement :
/// - Si la tâche n'est pas en cours : envoie un événement `idle` et termine.
/// - Sinon : envoie d'abord les logs déjà accumulés (rattrapage), puis stream les nouveaux logs
///   en temps réel jusqu'à réception de l'événement `Done`.
///
/// Cela permet aux clients qui se connectent après le démarrage (page rechargée, déclenchement cron,
/// ouverture d'un nouvel onglet) de récupérer tous les logs depuis le début.
pub async fn stream_progress(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    // Snapshot des Arc partagés (clone pour ne pas garder de guard DashMap pendant async)
    let snapshot = state
        .running_tasks
        .get(&task_id)
        .map(|rt| (rt.log_buffer.clone(), rt.log_notify.clone()));

    let event_stream: SseStream = if let Some((shared_logs, log_notify)) = snapshot {
        let mut subscriber = state.sse_broadcaster.subscribe(task_id);

        let s = stream! {
            let mut last_sent: usize = 0;

            loop {
                // 1. Drain : envoyer toutes les nouvelles lignes accumulées depuis le dernier flush
                let new_lines: Vec<String> = {
                    let logs = shared_logs.lock().unwrap_or_else(|p| p.into_inner());
                    if logs.len() > last_sent {
                        let slice = logs[last_sent..].to_vec();
                        last_sent = logs.len();
                        slice
                    } else {
                        Vec::new()
                    }
                };
                for line in new_lines {
                    let event = Event::default()
                        .event("log")
                        .data(json!({"line": line}).to_string());
                    yield Ok::<Event, Infallible>(event);
                }

                // 2. Attendre soit un nouveau log, soit l'événement Done
                tokio::select! {
                    _ = log_notify.notified() => {
                        // Nouveau log dispo → reboucle pour le drain
                    }
                    msg = subscriber.recv() => {
                        match msg {
                            Ok(SseEvent::Done { status, exit_code, duration_ms }) => {
                                // Drain final pour ne rien manquer
                                let new_lines: Vec<String> = {
                                    let logs = shared_logs.lock().unwrap_or_else(|p| p.into_inner());
                                    if logs.len() > last_sent {
                                        let slice = logs[last_sent..].to_vec();
                                        last_sent = logs.len();
                                        slice
                                    } else {
                                        Vec::new()
                                    }
                                };
                                let _ = last_sent; // silence "value assigned never read"
                                for line in new_lines {
                                    let event = Event::default()
                                        .event("log")
                                        .data(json!({"line": line}).to_string());
                                    yield Ok(event);
                                }

                                let done = Event::default()
                                    .event("done")
                                    .data(
                                        json!({
                                            "status": status,
                                            "exit_code": exit_code,
                                            "duration_ms": duration_ms,
                                        }).to_string(),
                                    );
                                yield Ok(done);
                                break;
                            }
                            Err(_) => {
                                // Channel fermé sans Done → la tâche a probablement été retirée.
                                let done = Event::default()
                                    .event("done")
                                    .data(json!({"status": "unknown"}).to_string());
                                yield Ok(done);
                                break;
                            }
                        }
                    }
                }
            }
        };

        Box::pin(s)
    } else {
        // Tâche pas en cours
        let idle_event = Event::default()
            .event("idle")
            .data(json!({"message": "No task currently running"}).to_string());
        Box::pin(futures::stream::once(async move {
            Ok::<Event, Infallible>(idle_event)
        }))
    };

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}
