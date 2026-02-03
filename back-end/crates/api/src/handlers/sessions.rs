use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;
use domain::models::{DraftEvent, DraftSession};

// DTOs for session endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub draft_id: Uuid,
    pub time_per_pick_seconds: i32,
    pub auto_pick_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub draft_id: Uuid,
    pub status: String,
    pub current_pick_number: i32,
    pub time_per_pick_seconds: i32,
    pub auto_pick_enabled: bool,
}

impl From<DraftSession> for SessionResponse {
    fn from(session: DraftSession) -> Self {
        Self {
            id: session.id,
            draft_id: session.draft_id,
            status: session.status.to_string(),
            current_pick_number: session.current_pick_number,
            time_per_pick_seconds: session.time_per_pick_seconds,
            auto_pick_enabled: session.auto_pick_enabled,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub created_at: String,
}

impl From<DraftEvent> for EventResponse {
    fn from(event: DraftEvent) -> Self {
        Self {
            id: event.id,
            session_id: event.session_id,
            event_type: event.event_type.to_string(),
            event_data: event.event_data,
            created_at: event.created_at.to_rfc3339(),
        }
    }
}

// Handlers

/// POST /api/v1/sessions
pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> ApiResult<(StatusCode, Json<SessionResponse>)> {
    // Create session
    let session = DraftSession::new(
        req.draft_id,
        req.time_per_pick_seconds,
        req.auto_pick_enabled,
    )?;

    let created = state.session_repo.create(&session).await?;

    // Record session created event
    let event = DraftEvent::session_created(
        created.id,
        req.draft_id,
        serde_json::json!({
            "time_per_pick_seconds": req.time_per_pick_seconds,
            "auto_pick_enabled": req.auto_pick_enabled,
        }),
    );
    state.event_repo.create(&event).await?;

    Ok((StatusCode::CREATED, Json(created.into())))
}

/// GET /api/v1/sessions/:id
pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SessionResponse>> {
    let session = state
        .session_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| domain::errors::DomainError::NotFound(format!("Session {}", id)))?;

    Ok(Json(session.into()))
}

/// POST /api/v1/sessions/:id/start
pub async fn start_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SessionResponse>> {
    let mut session = state
        .session_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| domain::errors::DomainError::NotFound(format!("Session {}", id)))?;

    session.start()?;
    let updated = state.session_repo.update(&session).await?;

    // Record session started event
    let event = DraftEvent::session_started(id);
    state.event_repo.create(&event).await?;

    // Broadcast status update via WebSocket
    let message = websocket::ServerMessage::draft_status(id, "InProgress".to_string());
    state.ws_manager.broadcast_to_session(id, message).await;

    Ok(Json(updated.into()))
}

/// POST /api/v1/sessions/:id/pause
pub async fn pause_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SessionResponse>> {
    let mut session = state
        .session_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| domain::errors::DomainError::NotFound(format!("Session {}", id)))?;

    session.pause()?;
    let updated = state.session_repo.update(&session).await?;

    // Record session paused event
    let event = DraftEvent::session_paused(id);
    state.event_repo.create(&event).await?;

    // Broadcast status update via WebSocket
    let message = websocket::ServerMessage::draft_status(id, "Paused".to_string());
    state.ws_manager.broadcast_to_session(id, message).await;

    Ok(Json(updated.into()))
}

/// GET /api/v1/sessions/:id/events
pub async fn get_session_events(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<EventResponse>>> {
    let events = state.event_repo.list_by_session(id).await?;
    let responses = events.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}
