use std::collections::{HashMap, HashSet};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use domain::models::{Draft, DraftPick, FitGrade, Position};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDraftRequest {
    pub name: String,
    pub year: i32,
    pub rounds: i32,
    pub picks_per_round: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DraftResponse {
    pub id: Uuid,
    pub name: String,
    pub year: i32,
    pub status: String,
    pub rounds: i32,
    pub picks_per_round: Option<i32>,
    pub total_picks: Option<i32>,
    pub is_realistic: bool,
}

impl From<Draft> for DraftResponse {
    fn from(draft: Draft) -> Self {
        let is_realistic = draft.is_realistic();
        let total_picks = draft.total_picks();
        Self {
            id: draft.id,
            name: draft.name,
            year: draft.year,
            status: draft.status.to_string(),
            rounds: draft.rounds,
            picks_per_round: draft.picks_per_round,
            total_picks,
            is_realistic,
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
    pub original_team_id: Option<Uuid>,
    pub is_compensatory: bool,
    pub is_traded: bool,
    pub notes: Option<String>,
}

impl From<DraftPick> for DraftPickResponse {
    fn from(pick: DraftPick) -> Self {
        let is_traded = pick.is_traded();
        Self {
            id: pick.id,
            draft_id: pick.draft_id,
            round: pick.round,
            pick_number: pick.pick_number,
            overall_pick: pick.overall_pick,
            team_id: pick.team_id,
            player_id: pick.player_id,
            picked_at: pick.picked_at.map(|dt| dt.to_rfc3339()),
            original_team_id: pick.original_team_id,
            is_compensatory: pick.is_compensatory,
            is_traded,
            notes: pick.notes,
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
        (status = 400, description = "Invalid request")
    ),
    tag = "drafts"
)]
pub async fn create_draft(
    State(state): State<AppState>,
    Json(payload): Json<CreateDraftRequest>,
) -> ApiResult<(StatusCode, Json<DraftResponse>)> {
    let draft = match payload.picks_per_round {
        Some(picks_per_round) => {
            state
                .draft_engine
                .create_draft(payload.name, payload.year, payload.rounds, picks_per_round)
                .await?
        }
        None => {
            state
                .draft_engine
                .create_realistic_draft(payload.name, payload.year, payload.rounds)
                .await?
        }
    };

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
    // Check if this is a realistic draft
    let draft = state
        .draft_engine
        .get_draft(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Draft with id {} not found", id)))?;

    let picks = if draft.is_realistic() {
        // For realistic drafts, use the draft order JSON data with trade metadata
        initialize_realistic_picks(&state, id, draft.rounds, draft.year).await?
    } else {
        // For custom drafts, use the existing standings-based logic
        state.draft_engine.initialize_picks(id).await?
    };

    let response: Vec<DraftPickResponse> = picks.into_iter().map(DraftPickResponse::from).collect();
    Ok((StatusCode::CREATED, Json(response)))
}

/// Initialize picks for a realistic draft from draft order JSON data.
async fn initialize_realistic_picks(
    state: &AppState,
    draft_id: Uuid,
    rounds: i32,
    year: i32,
) -> Result<Vec<DraftPick>, ApiError> {
    // Check if picks already exist
    let existing = state.draft_pick_repo.find_by_draft_id(draft_id).await?;
    if !existing.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "Draft picks have already been initialized for draft {}",
            draft_id
        )));
    }

    // Load draft order JSON for the given year.
    // Try multiple paths to support both local development and Docker environments.
    let filename = format!("draft_order_{}.json", year);
    let candidates = [
        format!("data/{}", filename),                         // back-end/data/ (when CWD is back-end/, e.g. `cargo run` from back-end/)
        format!("../../data/{}", filename),                   // back-end/data/ (cargo test from crate directory)
        format!("/app/data/{}", filename),                    // Docker container path
    ];

    let data = match candidates
        .iter()
        .find_map(|path| seed_data::draft_order_loader::parse_draft_order_file(path).ok())
    {
        Some(data) => data,
        None => {
            return Err(ApiError::InternalError(format!(
                "Draft order data file not found for year {}. Tried: {:?}",
                year, candidates
            )));
        }
    };

    // Create picks from the JSON data
    let picks = seed_data::draft_order_loader::initialize_realistic_draft_picks(
        &data,
        draft_id,
        rounds,
        state.team_repo.as_ref(),
        state.draft_pick_repo.as_ref(),
    )
    .await
    .map_err(|e| {
        ApiError::InternalError(format!("Failed to initialize realistic draft picks: {}", e))
    })?;

    Ok(picks)
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

// --- Available Players (consolidated endpoint) ---

#[derive(Debug, Serialize, ToSchema)]
pub struct RankingBadgeResponse {
    pub source_name: String,
    pub abbreviation: String,
    pub rank: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AvailablePlayerResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub position: Position,
    pub college: Option<String>,
    pub height_inches: Option<i32>,
    pub weight_pounds: Option<i32>,
    pub draft_year: i32,
    pub draft_eligible: bool,
    // Scouting report for the requesting team (if exists)
    pub scouting_grade: Option<f64>,
    pub fit_grade: Option<FitGrade>,
    pub injury_concern: Option<bool>,
    pub character_concern: Option<bool>,
    // Big board rankings across all sources
    pub rankings: Vec<RankingBadgeResponse>,
}

#[derive(Debug, Deserialize)]
pub struct AvailablePlayersQuery {
    pub team_id: Option<Uuid>,
}

/// GET /api/v1/drafts/:id/available-players?team_id=<uuid>
///
/// Returns all undrafted players for the given draft, each enriched with
/// scouting report data (for `team_id`) and big-board ranking badges.
#[utoipa::path(
    get,
    path = "/api/v1/drafts/{id}/available-players",
    responses(
        (status = 200, description = "Consolidated available players", body = Vec<AvailablePlayerResponse>),
        (status = 404, description = "Draft not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Draft ID"),
        ("team_id" = Option<Uuid>, Query, description = "Team ID for scouting report lookup")
    ),
    tag = "drafts"
)]
pub async fn get_available_players(
    State(state): State<AppState>,
    Path(draft_id): Path<Uuid>,
    Query(params): Query<AvailablePlayersQuery>,
) -> ApiResult<Json<Vec<AvailablePlayerResponse>>> {
    // 1. Verify draft exists and get picked player IDs concurrently
    let (draft_result, picks_result) = tokio::join!(
        state.draft_repo.find_by_id(draft_id),
        state.draft_pick_repo.find_by_draft_id(draft_id),
    );

    let draft = draft_result?
        .ok_or_else(|| ApiError::NotFound(format!("Draft with id {} not found", draft_id)))?;

    let picks = picks_result?;
    let picked_ids: HashSet<Uuid> = picks.iter().filter_map(|p| p.player_id).collect();

    // 2. Fetch players (scoped to draft year), rankings, sources, and optionally scouting reports concurrently
    let players_fut = state.player_repo.find_by_draft_year(draft.year);
    let rankings_fut = state.prospect_ranking_repo.find_all_with_source();
    let sources_fut = state.ranking_source_repo.find_all();

    let (all_players, all_rankings, sources, scouting_map) = if let Some(team_id) = params.team_id {
        let scouting_fut = state.scouting_report_repo.find_by_team_id(team_id);
        let (players_res, rankings_res, sources_res, scouting_res) =
            tokio::join!(players_fut, rankings_fut, sources_fut, scouting_fut);
        let map: HashMap<Uuid, domain::models::ScoutingReport> = scouting_res?
            .into_iter()
            .map(|r| (r.player_id, r))
            .collect();
        (players_res?, rankings_res?, sources_res?, map)
    } else {
        let (players_res, rankings_res, sources_res) =
            tokio::join!(players_fut, rankings_fut, sources_fut);
        (players_res?, rankings_res?, sources_res?, HashMap::new())
    };

    // 3. Filter out already-picked players
    let available: Vec<_> = all_players
        .into_iter()
        .filter(|p| !picked_ids.contains(&p.id))
        .collect();

    // 4. Build abbreviation lookup and rankings map
    let abbreviation_map: HashMap<String, String> = sources
        .into_iter()
        .map(|s| (s.name.clone(), s.abbreviation.clone()))
        .collect();

    let mut rankings_map: HashMap<Uuid, Vec<RankingBadgeResponse>> = HashMap::new();
    for entry in all_rankings {
        let abbreviation = abbreviation_map
            .get(&entry.source_name)
            .cloned()
            .unwrap_or_else(|| {
                entry
                    .source_name
                    .chars()
                    .take(2)
                    .collect::<String>()
                    .to_uppercase()
            });
        rankings_map
            .entry(entry.player_id)
            .or_default()
            .push(RankingBadgeResponse {
                source_name: entry.source_name,
                abbreviation,
                rank: entry.rank,
            });
    }
    for badges in rankings_map.values_mut() {
        badges.sort_by_key(|b| b.rank);
    }

    // 5. Assemble response, sorted by scouting grade desc (graded first)
    let mut response: Vec<AvailablePlayerResponse> = available
        .into_iter()
        .map(|player| {
            let report = scouting_map.get(&player.id);
            let rankings = rankings_map.remove(&player.id).unwrap_or_default();
            AvailablePlayerResponse {
                id: player.id,
                first_name: player.first_name,
                last_name: player.last_name,
                position: player.position,
                college: player.college,
                height_inches: player.height_inches,
                weight_pounds: player.weight_pounds,
                draft_year: player.draft_year,
                draft_eligible: player.draft_eligible,
                scouting_grade: report.map(|r| r.grade),
                fit_grade: report.and_then(|r| r.fit_grade),
                injury_concern: report.map(|r| r.injury_concern),
                character_concern: report.map(|r| r.character_concern),
                rankings,
            }
        })
        .collect();

    response.sort_by(|a, b| {
        match (a.scouting_grade, b.scouting_grade) {
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(ga), Some(gb)) => gb.partial_cmp(&ga).unwrap_or(std::cmp::Ordering::Equal),
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a.last_name.cmp(&b.last_name))
        .then_with(|| a.first_name.cmp(&b.first_name))
    });

    Ok(Json(response))
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

        // Cleanup (delete in order of foreign key dependencies)
        sqlx::query("DELETE FROM draft_picks")
            .execute(&pool)
            .await
            .expect("Failed to cleanup picks");
        sqlx::query("DELETE FROM drafts")
            .execute(&pool)
            .await
            .expect("Failed to cleanup drafts");
        sqlx::query("DELETE FROM players")
            .execute(&pool)
            .await
            .expect("Failed to cleanup players");
        sqlx::query("DELETE FROM team_seasons")
            .execute(&pool)
            .await
            .expect("Failed to cleanup team_seasons");
        sqlx::query("DELETE FROM teams")
            .execute(&pool)
            .await
            .expect("Failed to cleanup teams");

        (state, pool)
    }

    #[tokio::test]
    async fn test_create_draft() {
        let (state, _pool) = setup_test_state().await;

        let request = CreateDraftRequest {
            name: "Test Draft".to_string(),
            year: 2026,
            rounds: 7,
            picks_per_round: Some(32),
        };

        let result = create_draft(State(state), Json(request)).await;
        assert!(result.is_ok());

        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.0.name, "Test Draft");
        assert_eq!(response.0.year, 2026);
        assert_eq!(response.0.rounds, 7);
        assert_eq!(response.0.picks_per_round, Some(32));
        assert_eq!(response.0.total_picks, Some(224));
    }

    #[tokio::test]
    async fn test_get_draft() {
        let (state, _pool) = setup_test_state().await;

        // Create draft first
        let request = CreateDraftRequest {
            name: "Test Draft".to_string(),
            year: 2026,
            rounds: 7,
            picks_per_round: Some(32),
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
            name: "Draft 1".to_string(),
            year: 2026,
            rounds: 7,
            picks_per_round: Some(32),
        };
        let _ = create_draft(State(state.clone()), Json(request1))
            .await
            .unwrap();

        let request2 = CreateDraftRequest {
            name: "Draft 2".to_string(),
            year: 2027,
            rounds: 7,
            picks_per_round: Some(32),
        };
        let _ = create_draft(State(state.clone()), Json(request2))
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
            name: "Test Draft".to_string(),
            year: 2026,
            rounds: 7,
            picks_per_round: Some(2), // 2 teams
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
        let _created_team = state.team_repo.create(&team).await.unwrap();

        // Create player
        use domain::models::Player;
        let player =
            Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();
        let created_player = state.player_repo.create(&player).await.unwrap();

        // Create draft
        let request = CreateDraftRequest {
            name: "Test Draft".to_string(),
            year: 2026,
            rounds: 1,
            picks_per_round: Some(1),
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
