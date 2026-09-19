pub mod events;
pub mod notifications;
pub mod oauth;
pub mod progress;
pub mod remotes;
pub mod runs;
pub mod tasks;

use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Remotes
        .route("/api/remotes", get(remotes::list).post(remotes::create))
        .route(
            "/api/remotes/{id}",
            get(remotes::get)
                .put(remotes::update)
                .delete(remotes::delete),
        )
        .route("/api/remotes/{id}/test", post(remotes::test_connectivity))
        // Tasks
        .route("/api/tasks", get(tasks::list).post(tasks::create))
        .route(
            "/api/tasks/{id}",
            get(tasks::get).patch(tasks::patch).delete(tasks::delete),
        )
        .route("/api/tasks/{id}/trigger", post(tasks::trigger))
        .route("/api/tasks/{id}/restore", post(tasks::restore))
        .route("/api/tasks/{id}/status", get(tasks::status))
        .route("/api/tasks/{id}/progress", get(progress::stream_progress))
        .route("/api/tasks/{id}/runs", get(runs::list_for_task))
        // Global SSE events
        .route("/api/events", get(events::stream_events))
        // OAuth flow (Dropbox, Google Drive, OneDrive)
        .route("/api/oauth/start", get(oauth::start))
        .route("/api/oauth/callback", get(oauth::callback))
        // Runs
        .route("/api/runs/{run_id}", get(runs::get_run))
        .route("/api/runs/{run_id}/logs", get(runs::get_run_logs))
        // Notifications
        .route(
            "/api/notifications",
            get(notifications::list).post(notifications::create),
        )
        .route("/api/notifications/preview", post(notifications::preview))
        .route(
            "/api/notifications/{id}",
            get(notifications::get)
                .put(notifications::update)
                .delete(notifications::delete),
        )
        .route(
            "/api/notifications/{id}/test",
            post(notifications::test_notification),
        )
        .with_state(state)
}
