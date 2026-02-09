use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::handlers::drafts::DraftPickResponse;
use crate::state::AppState;
use domain::models::{ChartType, DraftEvent, DraftSession};

// DTOs for session endpoints

fn default_chart_type() -> ChartType {
    ChartType::JimmyJohnson
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub draft_id: Uuid,
    pub time_per_pick_seconds: i32,
    pub auto_pick_enabled: bool,
    #[serde(default = "default_chart_type")]
    pub chart_type: ChartType,
    #[serde(default)]
    pub controlled_team_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub draft_id: Uuid,
    pub status: String,
    pub current_pick_number: i32,
    pub time_per_pick_seconds: i32,
    pub auto_pick_enabled: bool,
    pub chart_type: ChartType,
    pub controlled_team_ids: Vec<Uuid>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
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
            chart_type: session.chart_type,
            controlled_team_ids: session.controlled_team_ids,
            started_at: session.started_at.map(|dt| dt.to_rfc3339()),
            completed_at: session.completed_at.map(|dt| dt.to_rfc3339()),
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
    // Validate draft exists
    let _draft = state
        .draft_repo
        .find_by_id(req.draft_id)
        .await?
        .ok_or_else(|| {
            domain::errors::DomainError::NotFound(format!(
                "Draft with id {} not found",
                req.draft_id
            ))
        })?;

    // Validate all controlled team IDs exist
    for team_id in &req.controlled_team_ids {
        state
            .team_repo
            .find_by_id(*team_id)
            .await?
            .ok_or_else(|| {
                domain::errors::DomainError::NotFound(format!(
                    "Team with id {} not found",
                    team_id
                ))
            })?;
    }

    // Create session
    let session = DraftSession::new(
        req.draft_id,
        req.time_per_pick_seconds,
        req.auto_pick_enabled,
        req.chart_type,
        req.controlled_team_ids.clone(),
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

#[derive(Debug, Serialize)]
pub struct AutoPickRunResponse {
    pub session: SessionResponse,
    pub picks_made: Vec<DraftPickResponse>,
}

/// POST /api/v1/sessions/:id/auto-pick-run
/// Loops through AI picks until reaching a user-controlled team's turn or draft completion.
pub async fn auto_pick_run(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AutoPickRunResponse>> {
    let mut session = state
        .session_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| domain::errors::DomainError::NotFound(format!("Session {}", id)))?;

    if session.status != domain::models::SessionStatus::InProgress {
        return Err(domain::errors::DomainError::InvalidState(
            "Session is not in progress".to_string(),
        )
        .into());
    }

    let mut picks_made = Vec::new();

    // Cache draft outside the loop (fetch once)
    let draft = state
        .draft_engine
        .get_draft(session.draft_id)
        .await?
        .ok_or_else(|| {
            domain::errors::DomainError::NotFound("Draft not found".to_string())
        })?;

    loop {
        // Get the next unmade pick
        let next_pick = state
            .draft_engine
            .get_next_pick(session.draft_id)
            .await?;
        let Some(pick) = next_pick else {
            // No more picks — draft complete
            break;
        };

        // Stop if this pick is user-controlled
        if !session.should_auto_pick(pick.team_id) {
            break;
        }

        // Execute auto-pick (with fallback on failure)
        let made_pick = match state.draft_engine.execute_auto_pick(pick.id).await {
            Ok(p) => p,
            Err(e) => {
                // Fallback: pick first available player
                tracing::warn!("Auto-pick failed, using fallback: {}", e);
                let available = state
                    .draft_engine
                    .get_available_players(session.draft_id, draft.year)
                    .await?;
                let first = available.first().ok_or_else(|| {
                    domain::errors::DomainError::ValidationError(
                        "No players available".to_string(),
                    )
                })?;
                state.draft_engine.make_pick(pick.id, first.id).await?
            }
        };

        // Advance session pick number in memory
        session.advance_pick()?;

        // Broadcast pick_made via WebSocket (only fetch team/player if player was assigned)
        if let Some(player_id) = made_pick.player_id {
            let team = state.team_repo.find_by_id(pick.team_id).await?;
            let player = state.player_repo.find_by_id(player_id).await?;

            if let (Some(team), Some(player)) = (team, player) {
                let ws_msg = websocket::ServerMessage::pick_made(
                    id,
                    pick.id,
                    pick.team_id,
                    player_id,
                    pick.round,
                    pick.pick_number,
                    format!("{} {}", player.first_name, player.last_name),
                    format!("{} {}", team.city, team.name),
                );
                state.ws_manager.broadcast_to_session(id, ws_msg).await;
            }
        }

        picks_made.push(DraftPickResponse::from(made_pick));

        // Small delay so WS messages arrive spaced out
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Batch session update — single DB write after all picks
    state.session_repo.update(&session).await?;

    Ok(Json(AutoPickRunResponse {
        session: SessionResponse::from(session),
        picks_made,
    }))
}

/// POST /api/v1/sessions/:id/advance-pick
/// Advance the session's current_pick_number by one.
pub async fn advance_pick(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SessionResponse>> {
    let mut session = state
        .session_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| domain::errors::DomainError::NotFound(format!("Session {}", id)))?;

    session.advance_pick()?;
    let updated = state.session_repo.update(&session).await?;

    Ok(Json(updated.into()))
}
