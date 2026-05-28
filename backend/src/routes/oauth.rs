use crate::{
    errors::{AppError, AppResult},
    services::oauth::{provider_for, state::PendingOAuthState},
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StartParams {
    /// Type rclone : `dropbox`, `drive`, `onedrive`
    pub provider: String,
    pub client_id: String,
    pub client_secret: String,
}

/// Détecte l'origine publique de l'application (scheme + host) depuis les headers.
fn detect_origin(headers: &HeaderMap) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http")
        .to_string();
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost")
        .to_string();
    format!("{scheme}://{host}")
}

/// Démarre un flow OAuth : génère un state, stocke les credentials temporairement,
/// et redirige vers la page d'autorisation du provider.
pub async fn start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<StartParams>,
) -> AppResult<Response> {
    let provider = provider_for(&params.provider).ok_or_else(|| {
        AppError::BadRequest(format!("Unknown OAuth provider: {}", params.provider))
    })?;

    let origin = detect_origin(&headers);
    let redirect_uri = format!("{origin}/api/oauth/callback");

    let state_key = state.oauth_state.store(PendingOAuthState {
        provider: params.provider.clone(),
        client_id: params.client_id.clone(),
        client_secret: params.client_secret,
        redirect_uri: redirect_uri.clone(),
        created_at: std::time::Instant::now(),
    });

    let authorize_url = provider.authorize_url(&params.client_id, &redirect_uri, &state_key);
    Ok(Redirect::to(&authorize_url).into_response())
}

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Endpoint de callback OAuth : échange le `code` contre un token et renvoie une page HTML
/// qui envoie le résultat au parent via `window.opener.postMessage`.
pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
) -> Response {
    if let Some(err) = params.error {
        let desc = params.error_description.unwrap_or_default();
        return render_result_page(&format!("{err}: {desc}"), None);
    }

    let (Some(code), Some(state_key)) = (params.code.as_deref(), params.state.as_deref()) else {
        return render_result_page("Missing code or state parameter", None);
    };

    let Some(pending) = state.oauth_state.take(state_key) else {
        return render_result_page("Invalid or expired OAuth state", None);
    };

    let Some(provider) = provider_for(&pending.provider) else {
        return render_result_page(&format!("Unknown provider: {}", pending.provider), None);
    };

    let token = match provider
        .exchange_code(
            &pending.client_id,
            &pending.client_secret,
            code,
            &pending.redirect_uri,
        )
        .await
    {
        Ok(t) => t,
        Err(e) => return render_result_page(&format!("Token exchange failed: {e}"), None),
    };

    let token_json = match serde_json::to_string(&token) {
        Ok(s) => s,
        Err(e) => return render_result_page(&format!("Token serialization failed: {e}"), None),
    };

    render_result_page("OAuth authorization successful!", Some(&token_json))
}

/// Génère la page HTML retournée par la callback qui notifie la fenêtre parente.
fn render_result_page(message: &str, token_json: Option<&str>) -> Response {
    let payload = match token_json {
        Some(t) => format!(
            r#"{{ "type": "rclone-ui-oauth", "success": true, "token": {} }}"#,
            t
        ),
        None => format!(
            r#"{{ "type": "rclone-ui-oauth", "success": false, "error": {} }}"#,
            serde_json::to_string(message).unwrap_or_else(|_| "\"Unknown error\"".to_string())
        ),
    };

    let safe_message = html_escape(message);
    let success = token_json.is_some();
    let color = if success { "#10b981" } else { "#ef4444" };
    let title = if success { "Connecté !" } else { "Échec" };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8" />
<title>{title} — rclone-ui</title>
<style>
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #0f172a; color: #e2e8f0;
    display: flex; align-items: center; justify-content: center;
    height: 100vh; margin: 0;
  }}
  .card {{
    background: #1e293b; padding: 2rem 3rem; border-radius: 12px;
    text-align: center; max-width: 480px;
  }}
  h1 {{ color: {color}; margin: 0 0 0.5rem; font-size: 1.25rem; }}
  p {{ color: #94a3b8; font-size: 0.875rem; }}
</style>
</head>
<body>
<div class="card">
  <h1>{title}</h1>
  <p>{safe_message}</p>
  <p style="font-size: 0.75rem;">Cette fenêtre va se fermer automatiquement…</p>
</div>
<script>
  (function() {{
    var payload = {payload};
    if (window.opener) {{
      try {{ window.opener.postMessage(payload, "*"); }} catch (e) {{}}
    }}
    setTimeout(function() {{ window.close(); }}, 800);
  }})();
</script>
</body>
</html>"#
    );

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(html),
    )
        .into_response()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
