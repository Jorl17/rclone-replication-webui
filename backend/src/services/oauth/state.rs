use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

const STATE_TTL: Duration = Duration::from_secs(600); // 10 min

#[derive(Debug, Clone)]
pub struct PendingOAuthState {
    pub provider: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub created_at: Instant,
}

#[derive(Debug, Default)]
pub struct OAuthStateStore {
    inner: DashMap<String, PendingOAuthState>,
}

impl OAuthStateStore {
    pub fn new() -> Arc<Self> {
        let store = Arc::new(OAuthStateStore::default());
        // Cleanup périodique (toutes les minutes)
        let store_clone = store.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                store_clone.cleanup();
            }
        });
        store
    }

    /// Stocke un état OAuth en cours et retourne le token de state unique.
    pub fn store(&self, state: PendingOAuthState) -> String {
        let key = Uuid::new_v4().to_string();
        self.inner.insert(key.clone(), state);
        key
    }

    /// Récupère et supprime un état OAuth (use-once).
    pub fn take(&self, key: &str) -> Option<PendingOAuthState> {
        self.inner.remove(key).map(|(_, v)| v).filter(|s| {
            s.created_at.elapsed() < STATE_TTL
        })
    }

    /// Supprime les états expirés.
    fn cleanup(&self) {
        let now = Instant::now();
        self.inner.retain(|_, v| now.duration_since(v.created_at) < STATE_TTL);
    }
}
