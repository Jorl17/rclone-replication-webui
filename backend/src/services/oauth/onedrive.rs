use super::{OAuthProvider, RcloneToken, expiry_iso};
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;

pub struct OneDrive;

#[async_trait]
impl OAuthProvider for OneDrive {
    fn name(&self) -> &'static str {
        "onedrive"
    }

    fn authorize_url(&self, client_id: &str, redirect_uri: &str, state: &str) -> String {
        let client_id_enc = urlencoding::encode(client_id);
        let redirect_enc = urlencoding::encode(redirect_uri);
        let state_enc = urlencoding::encode(state);
        let scope_enc = urlencoding::encode("files.readwrite.all offline_access");
        format!(
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize\
             ?client_id={client_id_enc}\
             &response_type=code\
             &redirect_uri={redirect_enc}\
             &scope={scope_enc}\
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
        struct MsTokenResp {
            access_token: String,
            token_type: String,
            expires_in: Option<i64>,
            refresh_token: Option<String>,
        }

        let client = reqwest::Client::new();
        let resp = client
            .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .form(&[
                ("code", code),
                ("grant_type", "authorization_code"),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("redirect_uri", redirect_uri),
                ("scope", "files.readwrite.all offline_access"),
            ])
            .send()
            .await
            .context("Microsoft token exchange request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("OneDrive token exchange failed {status}: {body}"));
        }

        let parsed: MsTokenResp = resp.json().await.context("Microsoft token parse")?;
        Ok(RcloneToken {
            access_token: parsed.access_token,
            token_type: parsed.token_type,
            refresh_token: parsed.refresh_token.unwrap_or_default(),
            expiry: expiry_iso(parsed.expires_in.unwrap_or(3600)),
        })
    }
}
