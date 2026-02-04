use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use domain::models::{Draft, DraftPick};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDraftRequest {
    pub year: i32,
    pub rounds: i32,
    pub picks_per_round: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DraftResponse {
    pub id: Uuid,
    pub year: i32,
    pub status: String,
    pub rounds: i32,
    pub picks_per_round: i32,
    pub total_picks: i32,
}

impl From<Draft> for DraftResponse {
    fn from(draft: Draft) -> Self {
        Self {
            id: draft.id,
            year: draft.year,
            status: draft.status.to_string(),
            rounds: draft.rounds,
            picks_per_round: draft.picks_per_round,
            total_picks: draft.total_picks(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DraftPickResponse {
    pub id: Uuid,
    pub draft_id: Uuid,
    pub round: i32,
    pub pick_number: i32,
    pub overall_pick: i32,
    pub team_id: Uuid,
    pub player_id: Option<Uuid>,
    pub picked_at: Option<String>,
}

impl From<DraftPick> for DraftPickResponse {
    fn from(pick: DraftPick) -> Self {
        Self {
            id: pick.id,
            draft_id: pick.draft_id,
            round: pick.round,
            pick_number: pick.pick_number,
            overall_pick: pick.overall_pick,
            team_id: pick.team_id,
            player_id: pick.player_id,
            picked_at: pick.picked_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MakePickRequest {
    pub player_id: Uuid,
}

/// POST /api/v1/drafts - Create a new draft
#[utoipa::path(
    post,
    path = "/api/v1/drafts",
    request_body = CreateDraftRequest,
    responses(
        (status = 201, description = "Draft created successfully", body = DraftResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Draft for this year already exists")
    ),
    tag = "drafts"
)]
pub async fn create_draft(
    State(state): State<AppState>,
    Json(payload): Json<CreateDraftRequest>,
) -> ApiResult<(StatusCode, Json<DraftResponse>)> {
    let draft = state
        .draft_engine
        .create_draft(payload.year, payload.rounds, payload.picks_per_round)
        .await?;

    Ok((StatusCode::CREATED, Json(DraftResponse::from(draft))))
}

/// GET /api/v1/drafts - List all drafts
#[utoipa::path(
    get,
    path = "/api/v1/drafts",
    responses(
        (status = 200, description = "List of all drafts", body = Vec<DraftResponse>)
    ),
    tag = "drafts"
)]
pub async fn list_drafts(State(state): State<AppState>) -> ApiResult<Json<Vec<DraftResponse>>> {
    let drafts = state.draft_engine.get_all_drafts().await?;
    let response: Vec<DraftResponse> = drafts.into_iter().map(DraftResponse::from).collect();
    Ok(Json(response))
}

/// GET /api/v1/drafts/:id - Get draft by ID
#[utoipa::path(
    get,
    path = "/api/v1/drafts/{id}",
    responses(
        (status = 200, description = "Draft found", body = DraftResponse),
        (status = 404, description = "Draft not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Draft ID")
    ),
    tag = "drafts"
)]
pub async fn get_draft(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DraftResponse>> {
    let draft = state
        .draft_engine
        .get_draft(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Draft with id {} not found", id)))?;

    Ok(Json(DraftResponse::from(draft)))
}

/// POST /api/v1/drafts/:id/initialize - Initialize picks for a draft
#[utoipa::path(
    post,
    path = "/api/v1/drafts/{id}/initialize",
    responses(
        (status = 201, description = "Picks initialized successfully", body = Vec<DraftPickResponse>),
        (status = 404, description = "Draft not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Draft ID")
    ),
    tag = "drafts"
)]
pub async fn initialize_draft_picks(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<Vec<DraftPickResponse>>)> {
    let picks = state.draft_engine.initialize_picks(id).await?;
    let response: Vec<DraftPickResponse> = picks.into_iter().map(DraftPickResponse::from).collect();
    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/v1/drafts/:id/picks - Get all picks for a draft
#[utoipa::path(
    get,
    path = "/api/v1/drafts/{id}/picks",
    responses(
        (status = 200, description = "List of all picks for the draft", body = Vec<DraftPickResponse>)
    ),
    params(
        ("id" = Uuid, Path, description = "Draft ID")
    ),
    tag = "drafts"
)]
pub async fn get_draft_picks(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<DraftPickResponse>>> {
    let picks = state.draft_engine.get_all_picks(id).await?;
    let response: Vec<DraftPickResponse> = picks.into_iter().map(DraftPickResponse::from).collect();
    Ok(Json(response))
}

/// GET /api/v1/drafts/:id/picks/next - Get next available pick
#[utoipa::path(
    get,
    path = "/api/v1/drafts/{id}/picks/next",
    responses(
        (status = 200, description = "Next available pick (null if none available)", body = Option<DraftPickResponse>)
    ),
    params(
        ("id" = Uuid, Path, description = "Draft ID")
    ),
    tag = "drafts"
)]
pub async fn get_next_pick(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Option<DraftPickResponse>>> {
    let pick = state.draft_engine.get_next_pick(id).await?;
    Ok(Json(pick.map(DraftPickResponse::from)))
}

/// GET /api/v1/drafts/:id/picks/available - Get all available picks
#[utoipa::path(
    get,
    path = "/api/v1/drafts/{id}/picks/available",
    responses(
        (status = 200, description = "List of all available (unmade) picks", body = Vec<DraftPickResponse>)
    ),
    params(
        ("id" = Uuid, Path, description = "Draft ID")
    ),
    tag = "drafts"
)]
pub async fn get_available_picks(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<DraftPickResponse>>> {
    let picks = state.draft_engine.get_available_picks(id).await?;
    let response: Vec<DraftPickResponse> = picks.into_iter().map(DraftPickResponse::from).collect();
    Ok(Json(response))
}

/// POST /api/v1/picks/:id/make - Make a draft pick
#[utoipa::path(
    post,
    path = "/api/v1/picks/{id}/make",
    request_body = MakePickRequest,
    responses(
        (status = 200, description = "Pick made successfully", body = DraftPickResponse),
        (status = 404, description = "Pick not found"),
        (status = 400, description = "Invalid request or player already drafted")
    ),
    params(
        ("id" = Uuid, Path, description = "Pick ID")
    ),
    tag = "picks"
)]
pub async fn make_pick(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<MakePickRequest>,
) -> ApiResult<Json<DraftPickResponse>> {
    let pick = state.draft_engine.make_pick(id, payload.player_id).await?;
    Ok(Json(DraftPickResponse::from(pick)))
}

/// POST /api/v1/drafts/:id/start - Start a draft
#[utoipa::path(
    post,
    path = "/api/v1/drafts/{id}/start",
    responses(
        (status = 200, description = "Draft started successfully", body = DraftResponse),
        (status = 404, description = "Draft not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Draft ID")
    ),
    tag = "drafts"
)]
pub async fn start_draft(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DraftResponse>> {
    let draft = state.draft_engine.start_draft(id).await?;
    Ok(Json(DraftResponse::from(draft)))
}

/// POST /api/v1/drafts/:id/pause - Pause a draft
#[utoipa::path(
    post,
    path = "/api/v1/drafts/{id}/pause",
    responses(
        (status = 200, description = "Draft paused successfully", body = DraftResponse),
        (status = 404, description = "Draft not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Draft ID")
    ),
    tag = "drafts"
)]
pub async fn pause_draft(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DraftResponse>> {
    let draft = state.draft_engine.pause_draft(id).await?;
    Ok(Json(DraftResponse::from(draft)))
}

/// POST /api/v1/drafts/:id/complete - Complete a draft
#[utoipa::path(
    post,
    path = "/api/v1/drafts/{id}/complete",
    responses(
        (status = 200, description = "Draft completed successfully", body = DraftResponse),
        (status = 404, description = "Draft not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Draft ID")
    ),
    tag = "drafts"
)]
pub async fn complete_draft(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DraftResponse>> {
    let draft = state.draft_engine.complete_draft(id).await?;
    Ok(Json(DraftResponse::from(draft)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use domain::models::{Conference, Division, Position};
    use sqlx::PgPool;

    async fn setup_test_state() -> (AppState, PgPool) {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        let pool = db::create_pool(&database_url)
            .await
            .expect("Failed to create pool");
        let state = AppState::new(pool.clone(), None);

        // Cleanup
        sqlx::query!("DELETE FROM draft_picks")
            .execute(&pool)
            .await
            .expect("Failed to cleanup picks");
        sqlx::query!("DELETE FROM drafts")
            .execute(&pool)
            .await
            .expect("Failed to cleanup drafts");
        sqlx::query!("DELETE FROM players")
            .execute(&pool)
            .await
            .expect("Failed to cleanup players");
        sqlx::query!("DELETE FROM teams")
            .execute(&pool)
            .await
            .expect("Failed to cleanup teams");

        (state, pool)
    }

    #[tokio::test]
    async fn test_create_draft() {
        let (state, _pool) = setup_test_state().await;

        let request = CreateDraftRequest {
            year: 2026,
            rounds: 7,
            picks_per_round: 32,
        };

        let result = create_draft(State(state), Json(request)).await;
        assert!(result.is_ok());

        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.0.year, 2026);
        assert_eq!(response.0.rounds, 7);
        assert_eq!(response.0.picks_per_round, 32);
        assert_eq!(response.0.total_picks, 224);
    }

    #[tokio::test]
    async fn test_get_draft() {
        let (state, _pool) = setup_test_state().await;

        // Create draft first
        let request = CreateDraftRequest {
            year: 2026,
            rounds: 7,
            picks_per_round: 32,
        };
        let (_status, created) = create_draft(State(state.clone()), Json(request))
            .await
            .unwrap();

        // Get draft
        let result = get_draft(State(state), Path(created.0.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.id, created.0.id);
        assert_eq!(response.year, 2026);
    }

    #[tokio::test]
    async fn test_list_drafts() {
        let (state, _pool) = setup_test_state().await;

        // Create two drafts
        let request1 = CreateDraftRequest {
            year: 2026,
            rounds: 7,
            picks_per_round: 32,
        };
        create_draft(State(state.clone()), Json(request1))
            .await
            .unwrap();

        let request2 = CreateDraftRequest {
            year: 2027,
            rounds: 7,
            picks_per_round: 32,
        };
        create_draft(State(state.clone()), Json(request2))
            .await
            .unwrap();

        // List all drafts
        let result = list_drafts(State(state)).await;
        assert!(result.is_ok());

        let drafts = result.unwrap().0;
        assert_eq!(drafts.len(), 2);
    }

    #[tokio::test]
    async fn test_initialize_draft_picks() {
        let (state, _pool) = setup_test_state().await;

        // Create teams first
        use domain::models::Team;
        let team1 = Team::new(
            "Team A".to_string(),
            "TMA".to_string(),
            "City A".to_string(),
            Conference::AFC,
            Division::AFCEast,
        )
        .unwrap();
        state.team_repo.create(&team1).await.unwrap();

        let team2 = Team::new(
            "Team B".to_string(),
            "TMB".to_string(),
            "City B".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();
        state.team_repo.create(&team2).await.unwrap();

        // Create draft
        let request = CreateDraftRequest {
            year: 2026,
            rounds: 7,
            picks_per_round: 2, // 2 teams
        };
        let (_status, created) = create_draft(State(state.clone()), Json(request))
            .await
            .unwrap();

        // Initialize picks
        let result = initialize_draft_picks(State(state), Path(created.0.id)).await;
        assert!(result.is_ok());

        let (_status, picks) = result.unwrap();
        // 2 teams * 7 rounds = 14 picks
        assert_eq!(picks.0.len(), 14);
        assert_eq!(picks.0[0].overall_pick, 1);
        assert_eq!(picks.0[13].overall_pick, 14);
    }

    #[tokio::test]
    async fn test_make_pick() {
        let (state, _pool) = setup_test_state().await;

        // Create team
        use domain::models::Team;
        let team = Team::new(
            "Team A".to_string(),
            "TMA".to_string(),
            "City A".to_string(),
            Conference::AFC,
            Division::AFCEast,
        )
        .unwrap();
        let created_team = state.team_repo.create(&team).await.unwrap();

        // Create player
        use domain::models::Player;
        let player =
            Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();
        let created_player = state.player_repo.create(&player).await.unwrap();

        // Create draft
        let request = CreateDraftRequest {
            year: 2026,
            rounds: 1,
            picks_per_round: 1,
        };
        let (_status, created_draft) = create_draft(State(state.clone()), Json(request))
            .await
            .unwrap();

        // Initialize picks
        let (_status, picks) =
            initialize_draft_picks(State(state.clone()), Path(created_draft.0.id))
                .await
                .unwrap();
        let pick_id = picks.0[0].id;

        // Make pick
        let make_pick_req = MakePickRequest {
            player_id: created_player.id,
        };
        let result = make_pick(State(state), Path(pick_id), Json(make_pick_req)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.player_id, Some(created_player.id));
        assert!(response.picked_at.is_some());
    }
}
