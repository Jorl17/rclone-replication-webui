use crate::{
    entities::{notification_channel, task},
    errors::{AppError, AppResult},
    models::notification::{
        ChannelTemplates, ChannelWithTaskCount, CreateChannelRequest, PreviewRequest,
        PreviewResponse, UpdateChannelRequest, render_channel_event,
    },
    services::{
        apprise,
        notification_message::{NotificationContext, NotificationEvent, NotificationLanguage},
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sea_orm::*;
use serde_json::json;
use uuid::Uuid;

fn require_language(language: &str) -> AppResult<String> {
    if !NotificationLanguage::is_supported(language) {
        return Err(AppError::BadRequest("Invalid language".into()));
    }
    Ok(NotificationLanguage::parse(language).as_str().to_string())
}

fn channel_to_list_item(ch: notification_channel::Model, task_count: i64) -> ChannelWithTaskCount {
    ChannelWithTaskCount {
        id: ch.id,
        name: ch.name,
        apprise_url: ch.apprise_url,
        enabled: ch.enabled,
        language: ch.language,
        templates: ChannelTemplates::from_json(&ch.templates),
        created_at: ch.created_at.into(),
        updated_at: ch.updated_at.into(),
        task_count,
    }
}

fn preview_for(language: &str, templates: &ChannelTemplates) -> PreviewResponse {
    let parsed = NotificationLanguage::parse(language);
    let ctx = NotificationContext::preview(parsed);
    PreviewResponse {
        language: parsed.as_str().to_string(),
        error: render_channel_event(language, templates, NotificationEvent::Error, &ctx),
        success: render_channel_event(language, templates, NotificationEvent::Success, &ctx),
        skipped: render_channel_event(language, templates, NotificationEvent::Skipped, &ctx),
        test: render_channel_event(language, templates, NotificationEvent::Test, &ctx),
    }
}

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<ChannelWithTaskCount>>> {
    let channels = notification_channel::Entity::find()
        .order_by_asc(notification_channel::Column::Name)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let mut result = Vec::with_capacity(channels.len());
    for ch in channels {
        let count = task::Entity::find()
            .filter(task::Column::NotificationChannelId.eq(ch.id))
            .count(&state.db)
            .await
            .map_err(AppError::Database)? as i64;

        result.push(channel_to_list_item(ch, count));
    }
    Ok(Json(result))
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateChannelRequest>,
) -> AppResult<(StatusCode, Json<notification_channel::Model>)> {
    if req.name.is_empty() || req.apprise_url.is_empty() {
        return Err(AppError::BadRequest(
            "name and apprise_url are required".into(),
        ));
    }
    let language = require_language(&req.language)?;
    let model = notification_channel::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(req.name),
        apprise_url: Set(req.apprise_url),
        enabled: Set(req.enabled),
        language: Set(language),
        templates: Set(req.templates.to_json()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    let result = model.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<notification_channel::Model>> {
    let channel = notification_channel::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Channel not found".into()))?;
    Ok(Json(channel))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateChannelRequest>,
) -> AppResult<Json<notification_channel::Model>> {
    let existing = notification_channel::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Channel not found".into()))?;

    let mut model: notification_channel::ActiveModel = existing.into();
    model.name = Set(req.name);
    model.apprise_url = Set(req.apprise_url);
    model.enabled = Set(req.enabled);
    if let Some(language) = req.language {
        model.language = Set(require_language(&language)?);
    }
    if let Some(templates) = req.templates {
        model.templates = Set(templates.to_json());
    }
    model.updated_at = Set(chrono::Utc::now().into());

    let result = model.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(result))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let count = task::Entity::find()
        .filter(task::Column::NotificationChannelId.eq(id))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    if count > 0 {
        return Err(AppError::Conflict(
            "Cannot delete channel: it is used by one or more tasks".into(),
        ));
    }

    let result = notification_channel::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected == 0 {
        return Err(AppError::NotFound("Channel not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn preview(Json(req): Json<PreviewRequest>) -> AppResult<Json<PreviewResponse>> {
    let language = require_language(&req.language)?;
    Ok(Json(preview_for(&language, &req.templates)))
}

pub async fn test_notification(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let channel = notification_channel::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Channel not found".into()))?;

    let templates = ChannelTemplates::from_json(&channel.templates);
    let language = NotificationLanguage::parse(&channel.language);
    let ctx = NotificationContext::preview(language);
    let message = render_channel_event(
        &channel.language,
        &templates,
        NotificationEvent::Test,
        &ctx,
    );

    match apprise::send_notification(
        &state.config.apprise_bin,
        &[channel.apprise_url],
        &message.subject,
        &message.body,
    )
    .await
    {
        Ok(_) => Ok(Json(
            json!({"success": true, "message": "Test notification sent"}),
        )),
        Err(_) => Ok(Json(
            json!({"success": false, "message": "Test notification failed"}),
        )),
    }
}
