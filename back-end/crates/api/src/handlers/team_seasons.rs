use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use domain::models::{PlayoffResult, TeamSeason};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct TeamSeasonResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub season_year: i32,
    pub wins: i32,
    pub losses: i32,
    pub ties: i32,
    pub playoff_result: Option<PlayoffResult>,
    pub draft_position: Option<i32>,
    pub win_percentage: f64,
}

impl From<TeamSeason> for TeamSeasonResponse {
    fn from(season: TeamSeason) -> Self {
        let win_percentage = season.win_percentage();
        Self {
            id: season.id,
            team_id: season.team_id,
            season_year: season.season_year,
            wins: season.wins,
            losses: season.losses,
            ties: season.ties,
            playoff_result: season.playoff_result,
            draft_position: season.draft_position,
            win_percentage,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DraftOrderEntry {
    pub draft_position: i32,
    pub team_id: Uuid,
    pub wins: i32,
    pub losses: i32,
    pub ties: i32,
    pub playoff_result: Option<PlayoffResult>,
}

impl From<TeamSeason> for DraftOrderEntry {
    fn from(season: TeamSeason) -> Self {
        // Safety: get_draft_order uses find_by_year_ordered_by_draft_position,
        // which filters for draft_position IS NOT NULL, so unwrap is safe here.
        Self {
            draft_position: season.draft_position.unwrap(),
            team_id: season.team_id,
            wins: season.wins,
            losses: season.losses,
            ties: season.ties,
            playoff_result: season.playoff_result,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct TeamSeasonQuery {
    /// The season year to filter by
    pub year: i32,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct DraftOrderQuery {
    /// The draft year (uses year-1 standings, e.g., 2026 uses 2025 standings)
    pub year: i32,
}

/// GET /api/v1/teams/{team_id}/seasons/{year} - Get a single team's season for a given year
#[utoipa::path(
    get,
    path = "/api/v1/teams/{team_id}/seasons/{year}",
    params(
        ("team_id" = Uuid, Path, description = "Team ID"),
        ("year" = i32, Path, description = "Season year")
    ),
    responses(
        (status = 200, description = "Team season for the specified year", body = TeamSeasonResponse),
        (status = 404, description = "Team season not found")
    ),
    tag = "team-seasons"
)]
pub async fn get_team_season(
    State(state): State<AppState>,
    Path((team_id, year)): Path<(Uuid, i32)>,
) -> ApiResult<Json<TeamSeasonResponse>> {
    let season = state
        .team_season_repo
        .find_by_team_and_year(team_id, year)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "Team season not found for team {} year {}",
                team_id, year
            ))
        })?;

    Ok(Json(TeamSeasonResponse::from(season)))
}

/// GET /api/v1/team-seasons - List all team seasons for a given year
#[utoipa::path(
    get,
    path = "/api/v1/team-seasons",
    params(TeamSeasonQuery),
    responses(
        (status = 200, description = "List of team seasons for the specified year", body = Vec<TeamSeasonResponse>)
    ),
    tag = "team-seasons"
)]
pub async fn list_team_seasons(
    State(state): State<AppState>,
    Query(query): Query<TeamSeasonQuery>,
) -> ApiResult<Json<Vec<TeamSeasonResponse>>> {
    let seasons = state.team_season_repo.find_by_year(query.year).await?;
    let response: Vec<TeamSeasonResponse> =
        seasons.into_iter().map(TeamSeasonResponse::from).collect();
    Ok(Json(response))
}

/// GET /api/v1/draft-order - Get teams in draft order for a given year
///
/// The draft order is based on the previous season's standings.
/// For example, the 2026 draft uses 2025 season standings.
#[utoipa::path(
    get,
    path = "/api/v1/draft-order",
    params(DraftOrderQuery),
    responses(
        (status = 200, description = "Teams in draft order", body = Vec<DraftOrderEntry>)
    ),
    tag = "team-seasons"
)]
pub async fn get_draft_order(
    State(state): State<AppState>,
    Query(query): Query<DraftOrderQuery>,
) -> ApiResult<Json<Vec<DraftOrderEntry>>> {
    // Draft year uses previous season's standings
    let standings_year = query.year - 1;

    let seasons = state
        .team_season_repo
        .find_by_year_ordered_by_draft_position(standings_year)
        .await?;

    let response: Vec<DraftOrderEntry> = seasons.into_iter().map(DraftOrderEntry::from).collect();
    Ok(Json(response))
}
