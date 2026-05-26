pub mod state;
mod dropbox;
mod google;
mod onedrive;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Format de token compatible rclone (sérialisé tel quel dans le champ `token`).
#[derive(Debug, Serialize, Deserialize)]
pub struct RcloneToken {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: String,
    /// ISO 8601 avec nanosecondes
    pub expiry: String,
}

/// Trait pour un provider OAuth.
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Construit l'URL d'autorisation à laquelle rediriger l'utilisateur.
    fn authorize_url(&self, client_id: &str, redirect_uri: &str, state: &str) -> String;

    /// Échange le `code` retourné par le provider contre un token OAuth.
    async fn exchange_code(
        &self,
        client_id: &str,
        client_secret: &str,
        code: &str,
        redirect_uri: &str,
    ) -> Result<RcloneToken>;
}

/// Sélectionne le provider OAuth correspondant au type rclone.
pub fn provider_for(remote_type: &str) -> Option<Box<dyn OAuthProvider>> {
    match remote_type {
        "dropbox" => Some(Box::new(dropbox::Dropbox)),
        "drive" => Some(Box::new(google::GoogleDrive)),
        "onedrive" => Some(Box::new(onedrive::OneDrive)),
        _ => None,
    }
}

/// Calcule la date d'expiration au format ISO 8601 nanosecondes (format rclone).
pub fn expiry_iso(seconds_from_now: i64) -> String {
    let dt = chrono::Utc::now() + chrono::Duration::seconds(seconds_from_now);
    // Format rclone : "2024-01-15T10:30:00.000000000+00:00"
    dt.format("%Y-%m-%dT%H:%M:%S%.9f%:z").to_string()
}
