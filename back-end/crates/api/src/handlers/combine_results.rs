use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use domain::models::{CombineResults, CombineSource};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateCombineResultsRequest {
    pub player_id: Uuid,
    pub year: i32,
    #[serde(default)]
    pub source: Option<String>,
    pub forty_yard_dash: Option<f64>,
    pub bench_press: Option<i32>,
    pub vertical_jump: Option<f64>,
    pub broad_jump: Option<i32>,
    pub three_cone_drill: Option<f64>,
    pub twenty_yard_shuttle: Option<f64>,
    pub arm_length: Option<f64>,
    pub hand_size: Option<f64>,
    pub wingspan: Option<f64>,
    pub ten_yard_split: Option<f64>,
    pub twenty_yard_split: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateCombineResultsRequest {
    pub forty_yard_dash: Option<f64>,
    pub bench_press: Option<i32>,
    pub vertical_jump: Option<f64>,
    pub broad_jump: Option<i32>,
    pub three_cone_drill: Option<f64>,
    pub twenty_yard_shuttle: Option<f64>,
    pub arm_length: Option<f64>,
    pub hand_size: Option<f64>,
    pub wingspan: Option<f64>,
    pub ten_yard_split: Option<f64>,
    pub twenty_yard_split: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CombineResultsResponse {
    pub id: Uuid,
    pub player_id: Uuid,
    pub year: i32,
    pub source: String,
    pub forty_yard_dash: Option<f64>,
    pub bench_press: Option<i32>,
    pub vertical_jump: Option<f64>,
    pub broad_jump: Option<i32>,
    pub three_cone_drill: Option<f64>,
    pub twenty_yard_shuttle: Option<f64>,
    pub arm_length: Option<f64>,
    pub hand_size: Option<f64>,
    pub wingspan: Option<f64>,
    pub ten_yard_split: Option<f64>,
    pub twenty_yard_split: Option<f64>,
}

impl From<CombineResults> for CombineResultsResponse {
    fn from(results: CombineResults) -> Self {
        Self {
            id: results.id,
            player_id: results.player_id,
            year: results.year,
            source: results.source.to_string(),
            forty_yard_dash: results.forty_yard_dash,
            bench_press: results.bench_press,
            vertical_jump: results.vertical_jump,
            broad_jump: results.broad_jump,
            three_cone_drill: results.three_cone_drill,
            twenty_yard_shuttle: results.twenty_yard_shuttle,
            arm_length: results.arm_length,
            hand_size: results.hand_size,
            wingspan: results.wingspan,
            ten_yard_split: results.ten_yard_split,
            twenty_yard_split: results.twenty_yard_split,
        }
    }
}

/// POST /api/v1/combine-results - Create new combine results
#[utoipa::path(
    post,
    path = "/api/v1/combine-results",
    request_body = CreateCombineResultsRequest,
    responses(
        (status = 201, description = "Combine results created successfully", body = CombineResultsResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Combine results for this player and year already exist")
    ),
    tag = "combine-results"
)]
pub async fn create_combine_results(
    State(state): State<AppState>,
    Json(req): Json<CreateCombineResultsRequest>,
) -> ApiResult<(StatusCode, Json<CombineResultsResponse>)> {
    let source = match &req.source {
        Some(s) => s.parse::<CombineSource>().map_err(|e| {
            ApiError::BadRequest(format!("Invalid source: {}", e))
        })?,
        None => CombineSource::Combine,
    };

    let mut results = CombineResults::new(req.player_id, req.year)?
        .with_source(source);

    if let Some(time) = req.forty_yard_dash {
        results = results.with_forty_yard_dash(time)?;
    }
    if let Some(reps) = req.bench_press {
        results = results.with_bench_press(reps)?;
    }
    if let Some(inches) = req.vertical_jump {
        results = results.with_vertical_jump(inches)?;
    }
    if let Some(inches) = req.broad_jump {
        results = results.with_broad_jump(inches)?;
    }
    if let Some(time) = req.three_cone_drill {
        results = results.with_three_cone_drill(time)?;
    }
    if let Some(time) = req.twenty_yard_shuttle {
        results = results.with_twenty_yard_shuttle(time)?;
    }
    if let Some(inches) = req.arm_length {
        results = results.with_arm_length(inches)?;
    }
    if let Some(inches) = req.hand_size {
        results = results.with_hand_size(inches)?;
    }
    if let Some(inches) = req.wingspan {
        results = results.with_wingspan(inches)?;
    }
    if let Some(time) = req.ten_yard_split {
        results = results.with_ten_yard_split(time)?;
    }
    if let Some(time) = req.twenty_yard_split {
        results = results.with_twenty_yard_split(time)?;
    }

    let created = state.combine_results_repo.create(&results).await?;

    Ok((
        StatusCode::CREATED,
        Json(CombineResultsResponse::from(created)),
    ))
}

/// GET /api/v1/combine-results/:id - Get combine results by ID
#[utoipa::path(
    get,
    path = "/api/v1/combine-results/{id}",
    responses(
        (status = 200, description = "Combine results found", body = CombineResultsResponse),
        (status = 404, description = "Combine results not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Combine results ID")
    ),
    tag = "combine-results"
)]
pub async fn get_combine_results(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CombineResultsResponse>> {
    let results = state
        .combine_results_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Combine results with id {} not found", id)))?;

    Ok(Json(CombineResultsResponse::from(results)))
}

/// GET /api/v1/players/:player_id/combine-results - Get all combine results for a player
#[utoipa::path(
    get,
    path = "/api/v1/players/{player_id}/combine-results",
    responses(
        (status = 200, description = "List of combine results for player", body = Vec<CombineResultsResponse>)
    ),
    params(
        ("player_id" = Uuid, Path, description = "Player ID")
    ),
    tag = "combine-results"
)]
pub async fn get_player_combine_results(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> ApiResult<Json<Vec<CombineResultsResponse>>> {
    let results = state
        .combine_results_repo
        .find_by_player_id(player_id)
        .await?;
    let response: Vec<CombineResultsResponse> = results
        .into_iter()
        .map(CombineResultsResponse::from)
        .collect();
    Ok(Json(response))
}

/// PUT /api/v1/combine-results/:id - Update combine results
#[utoipa::path(
    put,
    path = "/api/v1/combine-results/{id}",
    request_body = UpdateCombineResultsRequest,
    responses(
        (status = 200, description = "Combine results updated successfully", body = CombineResultsResponse),
        (status = 404, description = "Combine results not found"),
        (status = 400, description = "Invalid request")
    ),
    params(
        ("id" = Uuid, Path, description = "Combine results ID")
    ),
    tag = "combine-results"
)]
pub async fn update_combine_results(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCombineResultsRequest>,
) -> ApiResult<Json<CombineResultsResponse>> {
    let mut results = state
        .combine_results_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Combine results with id {} not found", id)))?;

    // Update fields with validation
    results.update_forty_yard_dash(req.forty_yard_dash)?;
    results.update_bench_press(req.bench_press)?;
    results.update_vertical_jump(req.vertical_jump)?;
    results.update_broad_jump(req.broad_jump)?;
    results.update_three_cone_drill(req.three_cone_drill)?;
    results.update_twenty_yard_shuttle(req.twenty_yard_shuttle)?;
    results.update_arm_length(req.arm_length)?;
    results.update_hand_size(req.hand_size)?;
    results.update_wingspan(req.wingspan)?;
    results.update_ten_yard_split(req.ten_yard_split)?;
    results.update_twenty_yard_split_time(req.twenty_yard_split)?;

    let updated = state.combine_results_repo.update(&results).await?;

    Ok(Json(CombineResultsResponse::from(updated)))
}

/// DELETE /api/v1/combine-results/:id - Delete combine results
#[utoipa::path(
    delete,
    path = "/api/v1/combine-results/{id}",
    responses(
        (status = 204, description = "Combine results deleted successfully"),
        (status = 404, description = "Combine results not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Combine results ID")
    ),
    tag = "combine-results"
)]
pub async fn delete_combine_results(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.combine_results_repo.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use domain::models::{Player, Position};
    use sqlx::PgPool;

    async fn setup_test_state() -> (AppState, PgPool) {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        let pool = db::create_pool(&database_url)
            .await
            .expect("Failed to create pool");

        // Clean up test data before each test
        cleanup(&pool).await;

        (AppState::new(pool.clone(), None), pool)
    }

    async fn cleanup(pool: &PgPool) {
        sqlx::query("DELETE FROM combine_results")
            .execute(pool)
            .await
            .expect("Failed to cleanup combine_results");
        sqlx::query("DELETE FROM scouting_reports")
            .execute(pool)
            .await
            .expect("Failed to cleanup scouting_reports");
        sqlx::query("DELETE FROM draft_picks")
            .execute(pool)
            .await
            .expect("Failed to cleanup draft_picks");
        sqlx::query("DELETE FROM drafts")
            .execute(pool)
            .await
            .expect("Failed to cleanup drafts");
        sqlx::query("DELETE FROM players")
            .execute(pool)
            .await
            .expect("Failed to cleanup players");
    }

    async fn create_test_player(state: &AppState) -> Player {
        let player =
            Player::new("Test".to_string(), "Player".to_string(), Position::QB, 2026).unwrap();
        state.player_repo.create(&player).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_combine_results() {
        let (state, _pool) = setup_test_state().await;

        let player = create_test_player(&state).await;

        let request = CreateCombineResultsRequest {
            player_id: player.id,
            year: 2026,
            source: None,
            forty_yard_dash: Some(4.52),
            bench_press: Some(20),
            vertical_jump: Some(35.5),
            broad_jump: None,
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            arm_length: None,
            hand_size: None,
            wingspan: None,
            ten_yard_split: None,
            twenty_yard_split: None,
        };

        let result = create_combine_results(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.player_id, player.id);
        assert_eq!(response.year, 2026);
        assert_eq!(response.source, "combine");
        assert_eq!(response.forty_yard_dash, Some(4.52));
    }

    #[tokio::test]
    async fn test_create_combine_results_with_source() {
        let (state, _pool) = setup_test_state().await;

        let player = create_test_player(&state).await;

        let request = CreateCombineResultsRequest {
            player_id: player.id,
            year: 2026,
            source: Some("pro_day".to_string()),
            forty_yard_dash: Some(4.48),
            bench_press: None,
            vertical_jump: None,
            broad_jump: None,
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            arm_length: None,
            hand_size: None,
            wingspan: None,
            ten_yard_split: None,
            twenty_yard_split: None,
        };

        let result = create_combine_results(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.source, "pro_day");
    }

    #[tokio::test]
    async fn test_get_combine_results() {
        let (state, _pool) = setup_test_state().await;

        let player = create_test_player(&state).await;

        let request = CreateCombineResultsRequest {
            player_id: player.id,
            year: 2026,
            source: None,
            forty_yard_dash: Some(4.52),
            bench_press: None,
            vertical_jump: None,
            broad_jump: None,
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            arm_length: None,
            hand_size: None,
            wingspan: None,
            ten_yard_split: None,
            twenty_yard_split: None,
        };

        let (_, created_response) = create_combine_results(State(state.clone()), Json(request))
            .await
            .unwrap();

        let result = get_combine_results(State(state.clone()), Path(created_response.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.id, created_response.id);
    }

    #[tokio::test]
    async fn test_get_player_combine_results() {
        let (state, _pool) = setup_test_state().await;

        let player = create_test_player(&state).await;

        let request1 = CreateCombineResultsRequest {
            player_id: player.id,
            year: 2025,
            source: None,
            forty_yard_dash: Some(4.60),
            bench_press: None,
            vertical_jump: None,
            broad_jump: None,
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            arm_length: None,
            hand_size: None,
            wingspan: None,
            ten_yard_split: None,
            twenty_yard_split: None,
        };

        let request2 = CreateCombineResultsRequest {
            player_id: player.id,
            year: 2026,
            source: None,
            forty_yard_dash: Some(4.52),
            bench_press: None,
            vertical_jump: None,
            broad_jump: None,
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            arm_length: None,
            hand_size: None,
            wingspan: None,
            ten_yard_split: None,
            twenty_yard_split: None,
        };

        let _ = create_combine_results(State(state.clone()), Json(request1))
            .await
            .unwrap();
        let _ = create_combine_results(State(state.clone()), Json(request2))
            .await
            .unwrap();

        let result = get_player_combine_results(State(state.clone()), Path(player.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.len(), 2);
    }
}
