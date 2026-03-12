use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use domain::services::RasScoringService;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct MeasurementScoreResponse {
    pub measurement: String,
    pub raw_value: f64,
    pub percentile: f64,
    pub score: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RasScoreResponse {
    pub player_id: Uuid,
    pub overall_score: Option<f64>,
    pub size_score: Option<f64>,
    pub speed_score: Option<f64>,
    pub strength_score: Option<f64>,
    pub explosion_score: Option<f64>,
    pub agility_score: Option<f64>,
    pub measurements_used: usize,
    pub measurements_total: usize,
    pub individual_scores: Vec<MeasurementScoreResponse>,
    pub explanation: Option<String>,
}

impl From<domain::models::RasScore> for RasScoreResponse {
    fn from(ras: domain::models::RasScore) -> Self {
        Self {
            player_id: ras.player_id,
            overall_score: ras.overall_score,
            size_score: ras.size_score,
            speed_score: ras.speed_score,
            strength_score: ras.strength_score,
            explosion_score: ras.explosion_score,
            agility_score: ras.agility_score,
            measurements_used: ras.measurements_used,
            measurements_total: ras.measurements_total,
            individual_scores: ras
                .individual_scores
                .into_iter()
                .map(|s| MeasurementScoreResponse {
                    measurement: s.measurement,
                    raw_value: s.raw_value,
                    percentile: s.percentile,
                    score: s.score,
                })
                .collect(),
            explanation: ras.explanation,
        }
    }
}

/// GET /api/v1/players/:player_id/ras - Get RAS score for a player
#[utoipa::path(
    get,
    path = "/api/v1/players/{player_id}/ras",
    responses(
        (status = 200, description = "RAS score calculated", body = RasScoreResponse),
        (status = 404, description = "Player not found or no combine data")
    ),
    params(
        ("player_id" = Uuid, Path, description = "Player ID")
    ),
    tag = "combine-results"
)]
pub async fn get_player_ras(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> ApiResult<Json<RasScoreResponse>> {
    // Get player
    let player = state
        .player_repo
        .find_by_id(player_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Player with id {} not found", player_id)))?;

    // Get combine results (use first available)
    let combine_list = state
        .combine_results_repo
        .find_by_player_id(player_id)
        .await?;

    let combine = combine_list.first().ok_or_else(|| {
        ApiError::NotFound(format!("No combine results found for player {}", player_id))
    })?;

    // Calculate RAS
    let ras = state.ras_service.calculate_ras(&player, combine).await;

    Ok(Json(RasScoreResponse::from(ras)))
}

/// GET /api/v1/combine-results/ras - Get RAS scores for all players with combine data
#[utoipa::path(
    get,
    path = "/api/v1/combine-results/ras",
    responses(
        (status = 200, description = "RAS scores for all players with combine data", body = Vec<RasScoreResponse>)
    ),
    tag = "combine-results"
)]
pub async fn get_all_ras(State(state): State<AppState>) -> ApiResult<Json<Vec<RasScoreResponse>>> {
    // Fetch all data in three queries (no N+1)
    let combine_results = state.combine_results_repo.find_all().await?;
    let all_percentiles = state.combine_percentile_repo.find_all().await?;
    let all_players = state.player_repo.find_all().await?;

    // Build a player map for O(1) lookup
    let player_map: HashMap<Uuid, _> = all_players.into_iter().map(|p| (p.id, p)).collect();

    // Deduplicate: use first combine result per player (consistent with get_player_ras)
    let mut seen_players = std::collections::HashSet::new();
    let mut ras_scores = Vec::new();
    for cr in &combine_results {
        if !seen_players.insert(cr.player_id) {
            continue;
        }
        if let Some(player) = player_map.get(&cr.player_id) {
            let ras =
                RasScoringService::calculate_ras_with_percentiles(player, cr, &all_percentiles);
            ras_scores.push(RasScoreResponse::from(ras));
        } else {
            tracing::warn!(
                player_id = %cr.player_id,
                "Combine result references player_id not found in players table"
            );
        }
    }

    Ok(Json(ras_scores))
}
