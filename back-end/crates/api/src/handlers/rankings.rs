use axum::extract::{Path, State};
use axum::Json;
use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct RankingSourceResponse {
    pub id: Uuid,
    pub name: String,
    pub url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerRankingResponse {
    pub source_name: String,
    pub source_id: Uuid,
    pub rank: i32,
    pub scraped_at: NaiveDate,
}

/// GET /api/v1/ranking-sources - List all ranking sources
#[utoipa::path(
    get,
    path = "/api/v1/ranking-sources",
    responses(
        (status = 200, description = "List of ranking sources", body = Vec<RankingSourceResponse>)
    ),
    tag = "rankings"
)]
pub async fn list_ranking_sources(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<RankingSourceResponse>>> {
    let sources = state.ranking_source_repo.find_all().await?;
    let response: Vec<RankingSourceResponse> = sources
        .into_iter()
        .map(|s| RankingSourceResponse {
            id: s.id,
            name: s.name,
            url: s.url,
            description: s.description,
        })
        .collect();
    Ok(Json(response))
}

/// GET /api/v1/players/{player_id}/rankings - Get all rankings for a player
#[utoipa::path(
    get,
    path = "/api/v1/players/{player_id}/rankings",
    responses(
        (status = 200, description = "List of rankings for player", body = Vec<PlayerRankingResponse>)
    ),
    params(
        ("player_id" = Uuid, Path, description = "Player ID")
    ),
    tag = "rankings"
)]
pub async fn get_player_rankings(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> ApiResult<Json<Vec<PlayerRankingResponse>>> {
    let rankings = state
        .prospect_ranking_repo
        .find_by_player_with_source(player_id)
        .await?;

    let response: Vec<PlayerRankingResponse> = rankings
        .into_iter()
        .map(|r| PlayerRankingResponse {
            source_name: r.source_name,
            source_id: r.source_id,
            rank: r.rank,
            scraped_at: r.scraped_at,
        })
        .collect();

    Ok(Json(response))
}

/// GET /api/v1/ranking-sources/{source_id}/rankings - Get full big board for a source
#[utoipa::path(
    get,
    path = "/api/v1/ranking-sources/{source_id}/rankings",
    responses(
        (status = 200, description = "Full big board for source", body = Vec<SourceRankingResponse>)
    ),
    params(
        ("source_id" = Uuid, Path, description = "Ranking source ID")
    ),
    tag = "rankings"
)]
pub async fn get_source_rankings(
    State(state): State<AppState>,
    Path(source_id): Path<Uuid>,
) -> ApiResult<Json<Vec<SourceRankingResponse>>> {
    let rankings = state.prospect_ranking_repo.find_by_source(source_id).await?;

    let response: Vec<SourceRankingResponse> = rankings
        .into_iter()
        .map(|r| SourceRankingResponse {
            player_id: r.player_id,
            rank: r.rank,
            scraped_at: r.scraped_at,
        })
        .collect();

    Ok(Json(response))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SourceRankingResponse {
    pub player_id: Uuid,
    pub rank: i32,
    pub scraped_at: NaiveDate,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AllRankingEntry {
    pub player_id: Uuid,
    pub source_name: String,
    pub source_id: Uuid,
    pub rank: i32,
    pub scraped_at: NaiveDate,
}

/// GET /api/v1/rankings - Get all rankings across all sources in one request
#[utoipa::path(
    get,
    path = "/api/v1/rankings",
    responses(
        (status = 200, description = "All rankings across all sources", body = Vec<AllRankingEntry>)
    ),
    tag = "rankings"
)]
pub async fn get_all_rankings(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<AllRankingEntry>>> {
    let rankings = state.prospect_ranking_repo.find_all_with_source().await?;

    let response: Vec<AllRankingEntry> = rankings
        .into_iter()
        .map(|r| AllRankingEntry {
            player_id: r.player_id,
            source_name: r.source_name,
            source_id: r.source_id,
            rank: r.rank,
            scraped_at: r.scraped_at,
        })
        .collect();

    Ok(Json(response))
}
