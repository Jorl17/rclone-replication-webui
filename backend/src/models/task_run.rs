use serde::Serialize;
use uuid::Uuid;

/// Vue sans log_output pour les listes (inclut stats pour le tableau)
#[derive(Debug, Serialize)]
pub struct TaskRunSummary {
    pub id: Uuid,
    pub task_id: Uuid,
    pub triggered_by: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<i64>,
    pub exit_code: Option<i32>,
    pub stats: Option<serde_json::Value>,
}

/// Run metadata without the log blob.
#[derive(Debug, Serialize)]
pub struct TaskRunDetail {
    pub id: Uuid,
    pub task_id: Uuid,
    pub triggered_by: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<i64>,
    pub exit_code: Option<i32>,
    pub stats: Option<serde_json::Value>,
    pub log_bytes: i64,
}

#[derive(Debug, Serialize)]
pub struct LogLineView {
    pub offset: usize,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct RunLogPage {
    pub lines: Vec<LogLineView>,
    pub total_bytes: usize,
    pub at_start: bool,
    pub at_end: bool,
    pub prev_offset: Option<usize>,
    pub next_offset: Option<usize>,
}
