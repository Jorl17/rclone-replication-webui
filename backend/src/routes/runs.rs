use crate::{
    entities::task,
    errors::{AppError, AppResult},
    models::task_run::{LogLineView, RunLogPage, TaskRunDetail, TaskRunSummary},
    services::run_log::{self, LogQuery},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use sea_orm::{DatabaseBackend, EntityTrait, FromQueryResult, Statement};
use serde::Deserialize;
use uuid::Uuid;

pub async fn list_for_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<Vec<TaskRunSummary>>> {
    if task::Entity::find_by_id(task_id)
        .one(&state.db)
        .await?
        .is_none()
    {
        return Err(AppError::NotFound("Task not found".into()));
    }

    let rows = TaskRunListRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"
        SELECT id, task_id, triggered_by, status, started_at, finished_at,
               duration_ms, exit_code, stats
        FROM task_runs
        WHERE task_id = $1
        ORDER BY started_at DESC
        LIMIT 100
        "#,
        [task_id.into()],
    ))
    .all(&state.db)
    .await?;

    let summaries = rows
        .into_iter()
        .map(|r| TaskRunSummary {
            id: r.id,
            task_id: r.task_id,
            triggered_by: r.triggered_by,
            status: r.status,
            started_at: r.started_at.with_timezone(&chrono::Utc),
            finished_at: r.finished_at.map(|t| t.with_timezone(&chrono::Utc)),
            duration_ms: r.duration_ms,
            exit_code: r.exit_code,
            stats: r.stats,
        })
        .collect();

    Ok(Json(summaries))
}

#[derive(Debug, FromQueryResult)]
struct TaskRunListRow {
    id: Uuid,
    task_id: Uuid,
    triggered_by: String,
    status: String,
    started_at: chrono::DateTime<chrono::FixedOffset>,
    finished_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    duration_ms: Option<i64>,
    exit_code: Option<i32>,
    stats: Option<serde_json::Value>,
}

#[derive(Debug, FromQueryResult)]
struct TaskRunMetaRow {
    id: Uuid,
    task_id: Uuid,
    triggered_by: String,
    status: String,
    started_at: chrono::DateTime<chrono::FixedOffset>,
    finished_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    duration_ms: Option<i64>,
    exit_code: Option<i32>,
    stats: Option<serde_json::Value>,
    log_bytes: i64,
}

pub async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> AppResult<Json<TaskRunDetail>> {
    let row = TaskRunMetaRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"
        SELECT id, task_id, triggered_by, status, started_at, finished_at,
               duration_ms, exit_code, stats,
               octet_length(COALESCE(log_output, ''))::bigint AS log_bytes
        FROM task_runs
        WHERE id = $1
        "#,
        [run_id.into()],
    ))
    .one(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Run not found".into()))?;

    Ok(Json(TaskRunDetail {
        id: row.id,
        task_id: row.task_id,
        triggered_by: row.triggered_by,
        status: row.status,
        started_at: row.started_at.with_timezone(&chrono::Utc),
        finished_at: row.finished_at.map(|t| t.with_timezone(&chrono::Utc)),
        duration_ms: row.duration_ms,
        exit_code: row.exit_code,
        stats: row.stats,
        log_bytes: row.log_bytes,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LogPageParams {
    from: Option<String>,
    after: Option<i64>,
    before: Option<i64>,
    window: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct LogChunkRow {
    total_bytes: i64,
    chunk: Option<Vec<u8>>,
}

pub async fn get_run_logs(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
    Query(params): Query<LogPageParams>,
) -> AppResult<Json<RunLogPage>> {
    let query = log_query(params)?;
    let total = total_log_bytes(&state, run_id).await?;
    let (start, len) = run_log::fetch_range(query, total);
    let chunk = fetch_log_chunk(&state, run_id, start, len).await?;
    let window = run_log::lines_from_chunk(&chunk, start, total, run_log::starts_at_line(query));

    Ok(Json(RunLogPage {
        lines: window
            .lines
            .into_iter()
            .map(|line| LogLineView {
                offset: line.offset,
                text: line.text,
            })
            .collect(),
        total_bytes: window.total_bytes,
        at_start: window.at_start,
        at_end: window.at_end,
        prev_offset: window.prev_offset,
        next_offset: window.next_offset,
    }))
}

fn log_query(params: LogPageParams) -> AppResult<LogQuery> {
    let window_bytes = params
        .window
        .and_then(|n| usize::try_from(n).ok())
        .map(run_log::clamp_window)
        .unwrap_or(run_log::DEFAULT_WINDOW_BYTES);

    match (params.after, params.before, params.from.as_deref()) {
        (Some(_), Some(_), _) => Err(AppError::BadRequest("Invalid log query".into())),
        (Some(after), None, _) => Ok(LogQuery::After {
            offset: offset_from_i64(after)?,
            window_bytes,
        }),
        (None, Some(before), _) => Ok(LogQuery::Before {
            offset: offset_from_i64(before)?,
            window_bytes,
        }),
        (None, None, Some("start")) => Ok(LogQuery::FromStart { window_bytes }),
        (None, None, Some("end") | None) => Ok(LogQuery::FromEnd { window_bytes }),
        (None, None, Some(_)) => Err(AppError::BadRequest("Invalid log query".into())),
    }
}

fn offset_from_i64(value: i64) -> AppResult<usize> {
    usize::try_from(value).map_err(|_| AppError::BadRequest("Invalid log query".into()))
}

fn pg_substr_args(start: usize, len: usize) -> AppResult<(i32, i32)> {
    let start = i32::try_from(start.saturating_add(1))
        .map_err(|_| AppError::BadRequest("Invalid log query".into()))?;
    let len =
        i32::try_from(len).map_err(|_| AppError::BadRequest("Invalid log query".into()))?;
    Ok((start, len))
}

async fn total_log_bytes(state: &AppState, run_id: Uuid) -> AppResult<usize> {
    let row = LogChunkRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"
        SELECT octet_length(COALESCE(log_output, ''))::bigint AS total_bytes,
               NULL::bytea AS chunk
        FROM task_runs
        WHERE id = $1
        "#,
        [run_id.into()],
    ))
    .one(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Run not found".into()))?;

    usize::try_from(row.total_bytes).map_err(|_| AppError::BadRequest("Invalid log query".into()))
}

async fn fetch_log_chunk(
    state: &AppState,
    run_id: Uuid,
    start: usize,
    len: usize,
) -> AppResult<String> {
    if len == 0 {
        return Ok(String::new());
    }

    // Postgres substr(bytea, int, int) is 1-based and counts bytes, not characters.
    let (pg_start, pg_len) = pg_substr_args(start, len)?;
    let row = LogChunkRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"
        SELECT octet_length(COALESCE(log_output, ''))::bigint AS total_bytes,
               substr(convert_to(COALESCE(log_output, ''), 'UTF8'), $2, $3) AS chunk
        FROM task_runs
        WHERE id = $1
        "#,
        [run_id.into(), pg_start.into(), pg_len.into()],
    ))
    .one(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Run not found".into()))?;

    Ok(run_log::decode_chunk(row.chunk.as_deref().unwrap_or(&[])))
}
