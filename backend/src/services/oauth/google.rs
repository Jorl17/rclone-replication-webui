use super::{expiry_iso, OAuthProvider, RcloneToken};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

pub struct GoogleDrive;

#[async_trait]
impl OAuthProvider for GoogleDrive {
    fn name(&self) -> &'static str {
        "drive"
    }

    fn authorize_url(&self, client_id: &str, redirect_uri: &str, state: &str) -> String {
        let client_id_enc = urlencoding::encode(client_id);
        let redirect_enc = urlencoding::encode(redirect_uri);
        let state_enc = urlencoding::encode(state);
        let scope_enc = urlencoding::encode("https://www.googleapis.com/auth/drive");
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth\
             ?client_id={client_id_enc}\
             &response_type=code\
             &redirect_uri={redirect_enc}\
             &scope={scope_enc}\
             &access_type=offline\
             &prompt=consent\
             &state={state_enc}"
        )
    }

    async fn exchange_code(
        &self,
        client_id: &str,
        client_secret: &str,
        code: &str,
        redirect_uri: &str,
    ) -> Result<RcloneToken> {
        #[derive(Deserialize)]
        struct GoogleTokenResp {
            access_token: String,
            token_type: String,
            expires_in: Option<i64>,
            refresh_token: Option<String>,
        }

        let client = reqwest::Client::new();
        let resp = client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("code", code),
                ("grant_type", "authorization_code"),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .context("Google token exchange request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Google token exchange failed {status}: {body}"));
        }

        let parsed: GoogleTokenResp = resp.json().await.context("Google token parse")?;
        Ok(RcloneToken {
            access_token: parsed.access_token,
            token_type: parsed.token_type,
            refresh_token: parsed.refresh_token.unwrap_or_default(),
            expiry: expiry_iso(parsed.expires_in.unwrap_or(3600)),
        })
    }
}
