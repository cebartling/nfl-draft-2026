use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct FeldmanFreaksQuery {
    pub year: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeldmanFreakListResponse {
    pub player_id: Uuid,
    pub rank: i32,
    pub description: String,
    pub article_url: Option<String>,
}

/// GET /api/v1/feldman-freaks?year=2026
pub async fn list_feldman_freaks(
    State(state): State<AppState>,
    Query(query): Query<FeldmanFreaksQuery>,
) -> ApiResult<Json<Vec<FeldmanFreakListResponse>>> {
    let freaks = state.feldman_freak_repo.find_by_year(query.year).await?;

    let response: Vec<FeldmanFreakListResponse> = freaks
        .into_iter()
        .map(|f| FeldmanFreakListResponse {
            player_id: f.player_id,
            rank: f.rank,
            description: f.description,
            article_url: f.article_url,
        })
        .collect();

    Ok(Json(response))
}
