use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use domain::models::{DraftEvent, PickTrade, TradeProposal};
use crate::error::ApiResult;
use crate::state::AppState;
use websocket::ServerMessage;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProposeTradeRequest {
    pub session_id: Uuid,
    pub from_team_id: Uuid,
    pub to_team_id: Uuid,
    pub from_team_picks: Vec<Uuid>,
    pub to_team_picks: Vec<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TradeResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub from_team_id: Uuid,
    pub to_team_id: Uuid,
    pub status: String,
    pub from_team_value: i32,
    pub to_team_value: i32,
    pub value_difference: i32,
}

impl From<PickTrade> for TradeResponse {
    fn from(trade: PickTrade) -> Self {
        Self {
            id: trade.id,
            session_id: trade.session_id,
            from_team_id: trade.from_team_id,
            to_team_id: trade.to_team_id,
            status: format!("{:?}", trade.status),
            from_team_value: trade.from_team_value,
            to_team_value: trade.to_team_value,
            value_difference: trade.value_difference,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TradeProposalResponse {
    pub trade: TradeResponse,
    pub from_team_picks: Vec<Uuid>,
    pub to_team_picks: Vec<Uuid>,
}

impl From<TradeProposal> for TradeProposalResponse {
    fn from(proposal: TradeProposal) -> Self {
        Self {
            trade: proposal.trade.into(),
            from_team_picks: proposal.from_team_picks,
            to_team_picks: proposal.to_team_picks,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TradeActionRequest {
    pub team_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/api/v1/trades",
    request_body = ProposeTradeRequest,
    responses(
        (status = 201, description = "Trade proposed", body = TradeProposalResponse),
        (status = 400, description = "Invalid trade"),
        (status = 409, description = "Pick already in trade")
    ),
    tag = "trades"
)]
pub async fn propose_trade(
    State(state): State<AppState>,
    Json(payload): Json<ProposeTradeRequest>,
) -> ApiResult<(StatusCode, Json<TradeProposalResponse>)> {
    let proposal = state.trade_engine.propose_trade(
        payload.session_id,
        payload.from_team_id,
        payload.to_team_id,
        payload.from_team_picks.clone(),
        payload.to_team_picks.clone(),
    ).await?;

    // Create and store draft event for audit trail
    let event = DraftEvent::trade_proposed(
        payload.session_id,
        proposal.trade.id,
        payload.from_team_id,
        payload.to_team_id,
    );
    state.event_repo.create(&event).await?;

    // Fetch team names for the WebSocket message
    let from_team = state.team_repo.find_by_id(payload.from_team_id).await?
        .ok_or_else(|| crate::error::ApiError::NotFound(format!("From team {} not found", payload.from_team_id)))?;
    let to_team = state.team_repo.find_by_id(payload.to_team_id).await?
        .ok_or_else(|| crate::error::ApiError::NotFound(format!("To team {} not found", payload.to_team_id)))?;

    // Broadcast to all WebSocket clients in session
    state.ws_manager.broadcast_to_session(
        payload.session_id,
        ServerMessage::trade_proposed(
            payload.session_id,
            proposal.trade.id,
            payload.from_team_id,
            payload.to_team_id,
            from_team.name,
            to_team.name,
            payload.from_team_picks,
            payload.to_team_picks,
            proposal.trade.from_team_value,
            proposal.trade.to_team_value,
        ),
    ).await;

    Ok((StatusCode::CREATED, Json(proposal.into())))
}

#[utoipa::path(
    post,
    path = "/api/v1/trades/{id}/accept",
    request_body = TradeActionRequest,
    responses(
        (status = 200, description = "Trade accepted and executed", body = TradeResponse)
    ),
    tag = "trades"
)]
pub async fn accept_trade(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TradeActionRequest>,
) -> ApiResult<Json<TradeResponse>> {
    let trade = state.trade_engine.accept_trade(id, payload.team_id).await?;

    // Create and store draft event
    let event = DraftEvent::trade_executed(trade.session_id, trade.id);
    state.event_repo.create(&event).await?;

    // Broadcast trade execution to session
    state.ws_manager.broadcast_to_session(
        trade.session_id,
        ServerMessage::trade_executed(
            trade.session_id,
            trade.id,
            trade.from_team_id,
            trade.to_team_id,
        ),
    ).await;

    Ok(Json(trade.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/trades/{id}/reject",
    request_body = TradeActionRequest,
    responses((status = 200, description = "Trade rejected", body = TradeResponse)),
    tag = "trades"
)]
pub async fn reject_trade(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TradeActionRequest>,
) -> ApiResult<Json<TradeResponse>> {
    let trade = state.trade_engine.reject_trade(id, payload.team_id).await?;

    // Create and store draft event for rejection
    let event = DraftEvent::trade_rejected(trade.session_id, trade.id, payload.team_id);
    state.event_repo.create(&event).await?;

    // Broadcast trade rejection to session
    state.ws_manager.broadcast_to_session(
        trade.session_id,
        ServerMessage::trade_rejected(
            trade.session_id,
            trade.id,
            payload.team_id,
        ),
    ).await;

    Ok(Json(trade.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/trades/{id}",
    responses((status = 200, description = "Trade details", body = TradeProposalResponse)),
    tag = "trades"
)]
pub async fn get_trade(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TradeProposalResponse>> {
    let proposal = state.trade_engine.get_trade(id).await?
        .ok_or_else(|| crate::error::ApiError::NotFound(format!("Trade {} not found", id)))?;
    Ok(Json(proposal.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/teams/{team_id}/pending-trades",
    responses((status = 200, description = "Pending trades", body = Vec<TradeProposalResponse>)),
    tag = "trades"
)]
pub async fn get_pending_trades(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
) -> ApiResult<Json<Vec<TradeProposalResponse>>> {
    let proposals = state.trade_engine.get_pending_trades(team_id).await?;
    Ok(Json(proposals.into_iter().map(Into::into).collect()))
}
