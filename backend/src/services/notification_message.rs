//! Builds notification subjects and bodies from on-disk locale catalogs.
//!
//! Sending and the preview endpoint both call [`render`]. Catalogs live in
//! `backend/locales/{en,fr,pt}/notifications.json`. French is the fallback and
//! keeps the verbatim pre-i18n wording, including historical whitespace.

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

const EN_JSON: &str = include_str!("../../locales/en/notifications.json");
const FR_JSON: &str = include_str!("../../locales/fr/notifications.json");
const PT_JSON: &str = include_str!("../../locales/pt/notifications.json");

static EN: LazyLock<Catalog> = LazyLock::new(|| parse_catalog(EN_JSON));
static FR: LazyLock<Catalog> = LazyLock::new(|| parse_catalog(FR_JSON));
static PT: LazyLock<Catalog> = LazyLock::new(|| parse_catalog(PT_JSON));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationEvent {
    Error,
    Success,
    Skipped,
    Test,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationLanguage {
    En,
    Fr,
    Pt,
}

impl NotificationLanguage {
    pub const ALL: [Self; 3] = [Self::En, Self::Fr, Self::Pt];

    /// Unknown or missing values keep the historical French messages.
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "en" => Self::En,
            "pt" => Self::Pt,
            _ => Self::Fr,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
            Self::Pt => "pt",
        }
    }

    pub fn is_supported(value: &str) -> bool {
        Self::ALL.iter().any(|language| language.as_str() == value.trim().to_ascii_lowercase())
    }
}

#[derive(Debug, Clone, Default)]
pub struct NotificationContext {
    pub task_id: String,
    pub run_id: String,
    pub exit_code: String,
    pub logs: String,
    pub reason: String,
}

impl NotificationContext {
    pub fn for_run(
        task_id: impl ToString,
        run_id: impl ToString,
        exit_code: Option<i32>,
        logs: &str,
    ) -> Self {
        Self {
            task_id: task_id.to_string(),
            run_id: run_id.to_string(),
            exit_code: exit_code.map(|c| c.to_string()).unwrap_or_default(),
            logs: logs.lines().take(50).collect::<Vec<_>>().join("\n"),
            reason: String::new(),
        }
    }

    pub fn preview(language: NotificationLanguage) -> Self {
        Self {
            task_id: "00000000-0000-4000-8000-000000000001".into(),
            run_id: "00000000-0000-4000-8000-000000000002".into(),
            exit_code: "1".into(),
            logs: "Transferred: 0 B / 0 B\n[ERROR] something went wrong".into(),
            reason: catalog(language).reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
struct Catalog {
    reason: String,
    error: NotificationMessage,
    success: NotificationMessage,
    skipped: NotificationMessage,
    test: NotificationMessage,
}

fn parse_catalog(raw: &str) -> Catalog {
    serde_json::from_str(raw).expect("notification catalog must be valid JSON")
}

fn catalog(language: NotificationLanguage) -> &'static Catalog {
    match language {
        NotificationLanguage::En => &EN,
        NotificationLanguage::Fr => &FR,
        NotificationLanguage::Pt => &PT,
    }
}

fn template_for(catalog: &Catalog, event: NotificationEvent) -> &NotificationMessage {
    match event {
        NotificationEvent::Error => &catalog.error,
        NotificationEvent::Success => &catalog.success,
        NotificationEvent::Skipped => &catalog.skipped,
        NotificationEvent::Test => &catalog.test,
    }
}

pub fn render(
    event: NotificationEvent,
    language: NotificationLanguage,
    ctx: &NotificationContext,
    custom_subject: Option<&str>,
    custom_body: Option<&str>,
) -> NotificationMessage {
    let catalog = catalog(language);
    let mut ctx = ctx.clone();
    if ctx.reason.is_empty() {
        ctx.reason = catalog.reason.clone();
    }

    let defaults = template_for(catalog, event);
    let subject = match nonempty(custom_subject) {
        Some(template) => interpolate(template, &ctx),
        None => interpolate(&defaults.subject, &ctx),
    };
    let mut body = match nonempty(custom_body) {
        Some(template) => interpolate(template, &ctx),
        None => interpolate(&defaults.body, &ctx),
    };
    truncate_utf8(&mut body, 4000);
    NotificationMessage { subject, body }
}

fn truncate_utf8(value: &mut String, max_bytes: usize) {
    if value.len() <= max_bytes {
        return;
    }
    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value.truncate(end);
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|s| !s.is_empty())
}

fn interpolate(template: &str, ctx: &NotificationContext) -> String {
    template
        .replace("{task_id}", &ctx.task_id)
        .replace("{run_id}", &ctx.run_id)
        .replace("{exit_code}", &ctx.exit_code)
        .replace("{logs}", &ctx.logs)
        .replace("{reason}", &ctx.reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ctx() -> NotificationContext {
        NotificationContext {
            task_id: "task-1".into(),
            run_id: "run-1".into(),
            exit_code: "1".into(),
            logs: "Transferred: 0 B / 0 B\n[ERROR] something went wrong".into(),
            reason: String::new(),
        }
    }

    /// Byte-for-byte copy of the pre-i18n `format!` in `task_executor`.
    fn legacy_french_error_body(task_id: &str, run_id: &str, exit_code: &str, logs: &str) -> String {
        format!(
            "**Tâche** : `{task_id}`\n\
                     **Run** : `{run_id}`\n\
                     **Code de sortie** : `{exit_code}`\n\
                     \n\
                     **Logs** :\n\
                     ```\n\
                     {logs}\n\
                     ```"
        )
    }

    fn legacy_french_success_body(task_id: &str, run_id: &str) -> String {
        format!(
            "**Tâche** : `{task_id}`\n\
                     **Run** : `{run_id}`\n\
                     \n\
                     La synchronisation s'est terminée avec succès."
        )
    }

    fn legacy_french_skipped_body(task_id: &str) -> String {
        format!(
            "**Tâche** : `{task_id}`\n\
                 **Raison** : L'exécution précédente est encore en cours\n\
                 \n\
                 La planification cron a tenté de lancer cette tâche, \
                 mais la synchronisation précédente n'est pas terminée. \
                 L'exécution a été _ignorée_."
        )
    }

    #[test]
    fn catalogs_exist_for_every_supported_language() {
        for language in NotificationLanguage::ALL {
            let catalog = catalog(language);
            assert!(!catalog.reason.is_empty(), "{}", language.as_str());
            assert!(!catalog.error.subject.is_empty(), "{}", language.as_str());
        }
    }

    #[test]
    fn french_error_matches_legacy_executor() {
        let ctx = sample_ctx();
        let mut expected = legacy_french_error_body(&ctx.task_id, &ctx.run_id, &ctx.exit_code, &ctx.logs);
        expected.truncate(4000);
        let msg = render(NotificationEvent::Error, NotificationLanguage::Fr, &ctx, None, None);
        assert_eq!(msg.subject, "Échec de la tâche de réplication");
        assert_eq!(msg.body, expected);
    }

    #[test]
    fn french_success_matches_legacy_executor() {
        let ctx = sample_ctx();
        let expected = legacy_french_success_body(&ctx.task_id, &ctx.run_id);
        let msg = render(NotificationEvent::Success, NotificationLanguage::Fr, &ctx, None, None);
        assert_eq!(msg.subject, "Tâche de réplication terminée avec succès");
        assert_eq!(msg.body, expected);
    }

    #[test]
    fn french_skipped_matches_legacy_scheduler() {
        let ctx = sample_ctx();
        let expected = legacy_french_skipped_body(&ctx.task_id);
        let msg = render(NotificationEvent::Skipped, NotificationLanguage::Fr, &ctx, None, None);
        assert_eq!(msg.subject, "Tâche de réplication ignorée");
        assert_eq!(msg.body, expected);
    }

    #[test]
    fn french_test_matches_legacy_route() {
        let msg = render(
            NotificationEvent::Test,
            NotificationLanguage::Fr,
            &sample_ctx(),
            None,
            None,
        );
        assert_eq!(msg.subject, "Test — rclone-ui");
        assert_eq!(
            msg.body,
            "Ce message confirme que le canal de notification **fonctionne correctement**.\n\n_Envoyé depuis rclone-ui._"
        );
    }

    #[test]
    fn unknown_language_is_french() {
        assert_eq!(NotificationLanguage::parse(""), NotificationLanguage::Fr);
        assert_eq!(NotificationLanguage::parse("de"), NotificationLanguage::Fr);
    }

    #[test]
    fn empty_custom_fields_use_catalog() {
        let ctx = sample_ctx();
        let default = render(
            NotificationEvent::Success,
            NotificationLanguage::En,
            &ctx,
            None,
            None,
        );
        let blank = render(
            NotificationEvent::Success,
            NotificationLanguage::En,
            &ctx,
            Some("   "),
            Some(""),
        );
        assert_eq!(default, blank);
    }

    #[test]
    fn custom_template_interpolates() {
        let ctx = sample_ctx();
        let msg = render(
            NotificationEvent::Error,
            NotificationLanguage::En,
            &ctx,
            Some("Fail {task_id}"),
            Some("code={exit_code}"),
        );
        assert_eq!(msg.subject, "Fail task-1");
        assert_eq!(msg.body, "code=1");
    }

    #[test]
    fn is_supported_accepts_only_catalog_codes() {
        assert!(NotificationLanguage::is_supported("en"));
        assert!(NotificationLanguage::is_supported("FR"));
        assert!(NotificationLanguage::is_supported("pt"));
        assert!(!NotificationLanguage::is_supported("de"));
        assert!(!NotificationLanguage::is_supported(""));
    }

    #[test]
    fn preview_context_uses_catalog_reason() {
        let ctx = NotificationContext::preview(NotificationLanguage::Fr);
        assert_eq!(ctx.reason, catalog(NotificationLanguage::Fr).reason);
    }

    #[test]
    fn body_is_truncated_to_4000_bytes() {
        let long = "x".repeat(5000);
        let msg = render(
            NotificationEvent::Test,
            NotificationLanguage::En,
            &sample_ctx(),
            None,
            Some(&long),
        );
        assert_eq!(msg.body.len(), 4000);
    }

    #[test]
    fn truncate_does_not_split_multibyte_characters() {
        let long = "é".repeat(3000);
        let msg = render(
            NotificationEvent::Test,
            NotificationLanguage::Fr,
            &sample_ctx(),
            None,
            Some(&long),
        );
        assert!(msg.body.len() <= 4000);
        assert!(msg.body.is_char_boundary(msg.body.len()));
        assert!(msg.body.ends_with('é') || msg.body.is_empty());
    }
}
