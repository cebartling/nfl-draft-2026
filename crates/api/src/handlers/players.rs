use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::models::{Player, Position};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePlayerRequest {
    pub first_name: String,
    pub last_name: String,
    pub position: Position,
    pub college: Option<String>,
    pub height_inches: Option<i32>,
    pub weight_pounds: Option<i32>,
    pub draft_year: i32,
}

#[derive(Debug, Serialize)]
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
pub async fn list_players(State(state): State<AppState>) -> ApiResult<Json<Vec<PlayerResponse>>> {
    let players = state.player_repo.find_all().await?;
    let response: Vec<PlayerResponse> = players.into_iter().map(PlayerResponse::from).collect();
    Ok(Json(response))
}

/// GET /api/v1/players/:id - Get player by ID
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
pub async fn create_player(
    State(state): State<AppState>,
    Json(payload): Json<CreatePlayerRequest>,
) -> ApiResult<Json<PlayerResponse>> {
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
    Ok(Json(PlayerResponse::from(created)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use sqlx::PgPool;

    async fn setup_test_state() -> (AppState, PgPool) {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft".to_string()
            });

        let pool = db::create_pool(&database_url).await.expect("Failed to create pool");
        let state = AppState::new(pool.clone());

        // Cleanup
        sqlx::query!("DELETE FROM players")
            .execute(&pool)
            .await
            .expect("Failed to cleanup");

        (state, pool)
    }

    #[tokio::test]
    async fn test_create_player() {
        let (state, _pool) = setup_test_state().await;

        let request = CreatePlayerRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            position: Position::QB,
            college: Some("Texas".to_string()),
            height_inches: Some(75),
            weight_pounds: Some(220),
            draft_year: 2026,
        };

        let result = create_player(State(state), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.first_name, "John");
        assert_eq!(response.last_name, "Doe");
        assert_eq!(response.position, Position::QB);
        assert_eq!(response.college, Some("Texas".to_string()));
    }

    #[tokio::test]
    async fn test_create_player_minimal() {
        let (state, _pool) = setup_test_state().await;

        let request = CreatePlayerRequest {
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            position: Position::WR,
            college: None,
            height_inches: None,
            weight_pounds: None,
            draft_year: 2026,
        };

        let result = create_player(State(state), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.first_name, "Jane");
        assert_eq!(response.college, None);
    }

    #[tokio::test]
    async fn test_get_player() {
        let (state, _pool): (AppState, PgPool) = setup_test_state().await;

        // Create a player first
        let request = CreatePlayerRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            position: Position::QB,
            college: None,
            height_inches: None,
            weight_pounds: None,
            draft_year: 2026,
        };

        let created = create_player(State(state.clone()), Json(request)).await.unwrap().0;

        // Now get it by ID
        let result = get_player(State(state), Path(created.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.id, created.id);
        assert_eq!(response.first_name, "John");
    }

    #[tokio::test]
    async fn test_get_player_not_found() {
        let (state, _pool) = setup_test_state().await;

        let result = get_player(State(state), Path(Uuid::new_v4())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_players() {
        let (state, _pool): (AppState, PgPool) = setup_test_state().await;

        // Create two players
        let player1 = CreatePlayerRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            position: Position::QB,
            college: None,
            height_inches: None,
            weight_pounds: None,
            draft_year: 2026,
        };

        let player2 = CreatePlayerRequest {
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            position: Position::WR,
            college: None,
            height_inches: None,
            weight_pounds: None,
            draft_year: 2026,
        };

        create_player(State(state.clone()), Json(player1)).await.unwrap();
        create_player(State(state.clone()), Json(player2)).await.unwrap();

        // List all players
        let result = list_players(State(state)).await;
        assert!(result.is_ok());

        let players = result.unwrap().0;
        assert_eq!(players.len(), 2);
    }
}
