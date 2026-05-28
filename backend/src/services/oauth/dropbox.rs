use super::{OAuthProvider, RcloneToken, expiry_iso};
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;

pub struct Dropbox;

#[async_trait]
impl OAuthProvider for Dropbox {
    fn name(&self) -> &'static str {
        "dropbox"
    }

    fn authorize_url(&self, client_id: &str, redirect_uri: &str, state: &str) -> String {
        let client_id_enc = urlencoding::encode(client_id);
        let redirect_enc = urlencoding::encode(redirect_uri);
        let state_enc = urlencoding::encode(state);
        format!(
            "https://www.dropbox.com/oauth2/authorize\
             ?client_id={client_id_enc}\
             &response_type=code\
             &redirect_uri={redirect_enc}\
             &token_access_type=offline\
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
        struct DropboxTokenResp {
            access_token: String,
            token_type: String,
            expires_in: Option<i64>,
            refresh_token: Option<String>,
        }

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.dropbox.com/oauth2/token")
            .form(&[
                ("code", code),
                ("grant_type", "authorization_code"),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .context("Dropbox token exchange request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Dropbox token exchange failed {status}: {body}"));
        }

        let parsed: DropboxTokenResp = resp.json().await.context("Dropbox token parse")?;
        Ok(RcloneToken {
            access_token: parsed.access_token,
            token_type: parsed.token_type,
            refresh_token: parsed.refresh_token.unwrap_or_default(),
            expiry: expiry_iso(parsed.expires_in.unwrap_or(14400)),
        })
    }
}
