use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use domain::models::{Player, Position};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePlayerRequest {
    pub first_name: String,
    pub last_name: String,
    pub position: Position,
    pub college: Option<String>,
    pub height_inches: Option<i32>,
    pub weight_pounds: Option<i32>,
    pub draft_year: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub position: Position,
    pub college: Option<String>,
    pub height_inches: Option<i32>,
    pub weight_pounds: Option<i32>,
    pub draft_year: i32,
    pub draft_eligible: bool,
}

impl From<Player> for PlayerResponse {
    fn from(player: Player) -> Self {
        Self {
            id: player.id,
            first_name: player.first_name,
            last_name: player.last_name,
            position: player.position,
            college: player.college,
            height_inches: player.height_inches,
            weight_pounds: player.weight_pounds,
            draft_year: player.draft_year,
            draft_eligible: player.draft_eligible,
        }
    }
}

/// GET /api/v1/players - List all players
#[utoipa::path(
    get,
    path = "/api/v1/players",
    responses(
        (status = 200, description = "List of all players", body = Vec<PlayerResponse>)
    ),
    tag = "players"
)]
pub async fn list_players(State(state): State<AppState>) -> ApiResult<Json<Vec<PlayerResponse>>> {
    let players = state.player_repo.find_all().await?;
    let response: Vec<PlayerResponse> = players.into_iter().map(PlayerResponse::from).collect();
    Ok(Json(response))
}

/// GET /api/v1/players/:id - Get player by ID
#[utoipa::path(
    get,
    path = "/api/v1/players/{id}",
    responses(
        (status = 200, description = "Player found", body = PlayerResponse),
        (status = 404, description = "Player not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Player ID")
    ),
    tag = "players"
)]
pub async fn get_player(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<PlayerResponse>> {
    let player = state
        .player_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Player with id {} not found", id)))?;

    Ok(Json(PlayerResponse::from(player)))
}

/// POST /api/v1/players - Create a new player
#[utoipa::path(
    post,
    path = "/api/v1/players",
    request_body = CreatePlayerRequest,
    responses(
        (status = 201, description = "Player created successfully", body = PlayerResponse),
        (status = 400, description = "Invalid request")
    ),
    tag = "players"
)]
pub async fn create_player(
    State(state): State<AppState>,
    Json(payload): Json<CreatePlayerRequest>,
) -> ApiResult<(StatusCode, Json<PlayerResponse>)> {
    let mut player = Player::new(
        payload.first_name,
        payload.last_name,
        payload.position,
        payload.draft_year,
    )?;

    // Add optional fields
    if let Some(college) = payload.college {
        player = player.with_college(college)?;
    }

    if let (Some(height), Some(weight)) = (payload.height_inches, payload.weight_pounds) {
        player = player.with_physical_stats(height, weight)?;
    }

    let created = state.player_repo.create(&player).await?;
    Ok((StatusCode::CREATED, Json(PlayerResponse::from(created))))
}
