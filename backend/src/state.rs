use crate::services::oauth::state::OAuthStateStore;
use crate::services::secrets::{self, SecretStore};
use crate::sse::broadcaster::SseBroadcaster;
use crate::sse::global::GlobalBroadcaster;
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{Mutex, Notify};
use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RunningTask {
    pub run_id: Uuid,
    pub triggered_by: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Buffer accumulé de toutes les lignes de log émises depuis le début de l'exécution.
    /// Permet aux nouveaux subscribers SSE (page rechargée, cron) de récupérer les logs depuis le début.
    pub log_buffer: Arc<StdMutex<Vec<String>>>,
    /// Signalé à chaque nouvelle ligne ajoutée au buffer — réveille les SSE en attente.
    pub log_notify: Arc<Notify>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub running_tasks: Arc<DashMap<Uuid, RunningTask>>,
    pub sse_broadcaster: Arc<SseBroadcaster>,
    pub global_broadcaster: Arc<GlobalBroadcaster>,
    pub config: Arc<crate::config::Config>,
    pub scheduler_handle: Arc<Mutex<Option<JobScheduler>>>,
    pub secret_store: Arc<dyn SecretStore>,
    pub oauth_state: Arc<OAuthStateStore>,
}

impl AppState {
    pub async fn new(db: DatabaseConnection, config: crate::config::Config) -> Self {
        let secret_store = secrets::build(config.secret_manager.as_ref()).await;
        AppState {
            db,
            running_tasks: Arc::new(DashMap::new()),
            sse_broadcaster: Arc::new(SseBroadcaster::new()),
            global_broadcaster: Arc::new(GlobalBroadcaster::new()),
            config: Arc::new(config),
            scheduler_handle: Arc::new(Mutex::new(None)),
            secret_store,
            oauth_state: OAuthStateStore::new(),
        }
    }
}
