use crate::services::notification_message::{
    NotificationEvent, NotificationLanguage, NotificationMessage,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Vue enrichie pour la liste (avec le nombre de tâches référençant ce canal)
#[derive(Debug, Serialize)]
pub struct ChannelWithTaskCount {
    pub id: Uuid,
    pub name: String,
    pub apprise_url: String,
    pub enabled: bool,
    pub language: String,
    pub templates: ChannelTemplates,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub task_count: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChannelTemplates {
    #[serde(default)]
    pub error: TemplateOverride,
    #[serde(default)]
    pub success: TemplateOverride,
    #[serde(default)]
    pub skipped: TemplateOverride,
    #[serde(default)]
    pub test: TemplateOverride,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TemplateOverride {
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub body: String,
}

impl ChannelTemplates {
    pub fn from_json(value: &serde_json::Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| serde_json::json!({}))
    }

    pub fn for_event(&self, event: NotificationEvent) -> &TemplateOverride {
        match event {
            NotificationEvent::Error => &self.error,
            NotificationEvent::Success => &self.success,
            NotificationEvent::Skipped => &self.skipped,
            NotificationEvent::Test => &self.test,
        }
    }
}

impl TemplateOverride {
    pub fn subject_override(&self) -> Option<&str> {
        let trimmed = self.subject.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    pub fn body_override(&self) -> Option<&str> {
        let trimmed = self.body.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub apprise_url: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub templates: ChannelTemplates,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: String,
    pub apprise_url: String,
    pub enabled: bool,
    pub language: Option<String>,
    pub templates: Option<ChannelTemplates>,
}

#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    pub language: String,
    #[serde(default)]
    pub templates: ChannelTemplates,
}

#[derive(Debug, Serialize)]
pub struct PreviewResponse {
    pub language: String,
    pub error: NotificationMessage,
    pub success: NotificationMessage,
    pub skipped: NotificationMessage,
    pub test: NotificationMessage,
}

pub fn render_channel_event(
    language: &str,
    templates: &ChannelTemplates,
    event: NotificationEvent,
    ctx: &crate::services::notification_message::NotificationContext,
) -> NotificationMessage {
    let override_ = templates.for_event(event);
    crate::services::notification_message::render(
        event,
        NotificationLanguage::parse(language),
        ctx,
        override_.subject_override(),
        override_.body_override(),
    )
}

fn default_true() -> bool {
    true
}

fn default_language() -> String {
    "fr".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::notification_message::{NotificationContext, NotificationEvent};
    use serde_json::json;

    #[test]
    fn missing_create_language_defaults_to_french() {
        let req: CreateChannelRequest =
            serde_json::from_str(r#"{"name":"alerts","apprise_url":"mmost://host/hook"}"#).unwrap();
        assert_eq!(req.language, "fr");
        assert!(req.enabled);
        assert_eq!(req.templates, ChannelTemplates::default());
    }

    #[test]
    fn empty_json_templates_are_blank_overrides() {
        let templates = ChannelTemplates::from_json(&json!({}));
        assert_eq!(templates, ChannelTemplates::default());
        assert_eq!(templates.error.subject_override(), None);
        assert_eq!(templates.error.body_override(), None);
    }

    #[test]
    fn blank_overrides_use_catalog_text() {
        let ctx = NotificationContext {
            task_id: "task-1".into(),
            run_id: "run-1".into(),
            exit_code: "1".into(),
            logs: "log".into(),
            reason: String::new(),
        };
        let catalog = render_channel_event("en", &ChannelTemplates::default(), NotificationEvent::Success, &ctx);
        let blank = ChannelTemplates {
            success: TemplateOverride {
                subject: "   ".into(),
                body: String::new(),
            },
            ..ChannelTemplates::default()
        };
        let rendered = render_channel_event("en", &blank, NotificationEvent::Success, &ctx);
        assert_eq!(rendered, catalog);
    }

    #[test]
    fn custom_overrides_win_per_event() {
        let ctx = NotificationContext {
            task_id: "task-1".into(),
            run_id: "run-1".into(),
            exit_code: "9".into(),
            logs: "log".into(),
            reason: String::new(),
        };
        let templates = ChannelTemplates {
            error: TemplateOverride {
                subject: "Fail {task_id}".into(),
                body: "code={exit_code}".into(),
            },
            ..ChannelTemplates::default()
        };
        let msg = render_channel_event("fr", &templates, NotificationEvent::Error, &ctx);
        assert_eq!(msg.subject, "Fail task-1");
        assert_eq!(msg.body, "code=9");
    }
}
