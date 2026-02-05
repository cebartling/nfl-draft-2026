use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use domain::models::{PlayoffResult, TeamSeason};

use crate::error::ApiResult;
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
        Self {
            draft_position: season.draft_position.unwrap_or(0),
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use sqlx::PgPool;

    use db::repositories::{SqlxTeamRepository, SqlxTeamSeasonRepository};
    use domain::models::{Conference, Division, Team};
    use domain::repositories::{TeamRepository, TeamSeasonRepository};

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        db::create_pool(&database_url)
            .await
            .expect("Failed to create pool")
    }

    async fn cleanup(pool: &PgPool) {
        sqlx::query!("DELETE FROM team_seasons")
            .execute(pool)
            .await
            .expect("Failed to cleanup team_seasons");
        sqlx::query!("DELETE FROM draft_picks")
            .execute(pool)
            .await
            .expect("Failed to cleanup picks");
        sqlx::query!("DELETE FROM drafts")
            .execute(pool)
            .await
            .expect("Failed to cleanup drafts");
        sqlx::query!("DELETE FROM teams")
            .execute(pool)
            .await
            .expect("Failed to cleanup teams");
    }

    #[tokio::test]
    async fn test_list_team_seasons() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        // Create a team
        let team_repo = SqlxTeamRepository::new(pool.clone());
        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();
        let team = team_repo.create(&team).await.unwrap();

        // Create a season
        let season_repo = SqlxTeamSeasonRepository::new(pool.clone());
        let season = TeamSeason::new(
            team.id,
            2025,
            10,
            7,
            0,
            Some(PlayoffResult::WildCard),
            Some(15),
        )
        .unwrap();
        season_repo.create(&season).await.unwrap();

        // Query via handler
        let state = AppState::new(pool.clone(), None);
        let result = list_team_seasons(State(state), Query(TeamSeasonQuery { year: 2025 })).await;

        assert!(result.is_ok());
        let seasons = result.unwrap().0;
        assert_eq!(seasons.len(), 1);
        assert_eq!(seasons[0].wins, 10);
        assert_eq!(seasons[0].losses, 7);

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_get_draft_order() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let team_repo = SqlxTeamRepository::new(pool.clone());
        let season_repo = SqlxTeamSeasonRepository::new(pool.clone());

        // Create two teams with different draft positions
        let team1 = Team::new(
            "Tennessee Titans".to_string(),
            "TEN".to_string(),
            "Nashville".to_string(),
            Conference::AFC,
            Division::AFCSouth,
        )
        .unwrap();
        let team1 = team_repo.create(&team1).await.unwrap();

        let team2 = Team::new(
            "Cleveland Browns".to_string(),
            "CLE".to_string(),
            "Cleveland".to_string(),
            Conference::AFC,
            Division::AFCNorth,
        )
        .unwrap();
        let team2 = team_repo.create(&team2).await.unwrap();

        // Team1 picks first, Team2 picks second
        let season1 = TeamSeason::new(team1.id, 2025, 3, 14, 0, None, Some(1)).unwrap();
        let season2 = TeamSeason::new(team2.id, 2025, 3, 14, 0, None, Some(2)).unwrap();

        season_repo.create(&season1).await.unwrap();
        season_repo.create(&season2).await.unwrap();

        // Query draft order for 2026 (uses 2025 standings)
        let state = AppState::new(pool.clone(), None);
        let result = get_draft_order(State(state), Query(DraftOrderQuery { year: 2026 })).await;

        assert!(result.is_ok());
        let order = result.unwrap().0;
        assert_eq!(order.len(), 2);
        assert_eq!(order[0].draft_position, 1);
        assert_eq!(order[0].team_id, team1.id);
        assert_eq!(order[1].draft_position, 2);
        assert_eq!(order[1].team_id, team2.id);

        cleanup(&pool).await;
    }
}
