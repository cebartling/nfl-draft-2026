use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use domain::models::{Conference, Division, Team};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTeamRequest {
    pub name: String,
    pub abbreviation: String,
    pub city: String,
    pub conference: Conference,
    pub division: Division,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub abbreviation: String,
    pub city: String,
    pub conference: Conference,
    pub division: Division,
}

impl From<Team> for TeamResponse {
    fn from(team: Team) -> Self {
        Self {
            id: team.id,
            name: team.name,
            abbreviation: team.abbreviation,
            city: team.city,
            conference: team.conference,
            division: team.division,
        }
    }
}

/// GET /api/v1/teams - List all teams
#[utoipa::path(
    get,
    path = "/api/v1/teams",
    responses(
        (status = 200, description = "List of all teams", body = Vec<TeamResponse>)
    ),
    tag = "teams"
)]
pub async fn list_teams(State(state): State<AppState>) -> ApiResult<Json<Vec<TeamResponse>>> {
    let teams = state.team_repo.find_all().await?;
    let response: Vec<TeamResponse> = teams.into_iter().map(TeamResponse::from).collect();
    Ok(Json(response))
}

/// GET /api/v1/teams/:id - Get team by ID
#[utoipa::path(
    get,
    path = "/api/v1/teams/{id}",
    responses(
        (status = 200, description = "Team found", body = TeamResponse),
        (status = 404, description = "Team not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    tag = "teams"
)]
pub async fn get_team(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TeamResponse>> {
    let team = state
        .team_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Team with id {} not found", id)))?;

    Ok(Json(TeamResponse::from(team)))
}

/// POST /api/v1/teams - Create a new team
#[utoipa::path(
    post,
    path = "/api/v1/teams",
    request_body = CreateTeamRequest,
    responses(
        (status = 201, description = "Team created successfully", body = TeamResponse),
        (status = 400, description = "Invalid request")
    ),
    tag = "teams"
)]
pub async fn create_team(
    State(state): State<AppState>,
    Json(payload): Json<CreateTeamRequest>,
) -> ApiResult<(StatusCode, Json<TeamResponse>)> {
    let team = Team::new(
        payload.name,
        payload.abbreviation,
        payload.city,
        payload.conference,
        payload.division,
    )?;

    let created = state.team_repo.create(&team).await?;
    Ok((StatusCode::CREATED, Json(TeamResponse::from(created))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
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
        sqlx::query!("DELETE FROM draft_picks")
            .execute(&pool)
            .await
            .expect("Failed to cleanup picks");
        sqlx::query!("DELETE FROM drafts")
            .execute(&pool)
            .await
            .expect("Failed to cleanup drafts");
        sqlx::query!("DELETE FROM teams")
            .execute(&pool)
            .await
            .expect("Failed to cleanup teams");

        (state, pool)
    }

    #[tokio::test]
    async fn test_create_team() {
        let (state, _pool) = setup_test_state().await;

        let request = CreateTeamRequest {
            name: "Dallas Cowboys".to_string(),
            abbreviation: "DAL".to_string(),
            city: "Dallas".to_string(),
            conference: Conference::NFC,
            division: Division::NFCEast,
        };

        let result = create_team(State(state), Json(request)).await;
        assert!(result.is_ok());

        let (_status, response) = result.unwrap();
        assert_eq!(response.0.name, "Dallas Cowboys");
        assert_eq!(response.0.abbreviation, "DAL");
    }

    #[tokio::test]
    async fn test_get_team() {
        let (state, _pool): (AppState, PgPool) = setup_test_state().await;

        // Create a team first
        let request = CreateTeamRequest {
            name: "Dallas Cowboys".to_string(),
            abbreviation: "DAL".to_string(),
            city: "Dallas".to_string(),
            conference: Conference::NFC,
            division: Division::NFCEast,
        };

        let (_status, created) = create_team(State(state.clone()), Json(request))
            .await
            .unwrap();

        // Now get it by ID
        let result = get_team(State(state), Path(created.0.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.id, created.0.id);
        assert_eq!(response.name, "Dallas Cowboys");
    }

    #[tokio::test]
    async fn test_get_team_not_found() {
        let (state, _pool) = setup_test_state().await;

        let result = get_team(State(state), Path(Uuid::new_v4())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_teams() {
        let (state, _pool): (AppState, PgPool) = setup_test_state().await;

        // Create two teams
        let team1 = CreateTeamRequest {
            name: "Dallas Cowboys".to_string(),
            abbreviation: "DAL".to_string(),
            city: "Dallas".to_string(),
            conference: Conference::NFC,
            division: Division::NFCEast,
        };

        let team2 = CreateTeamRequest {
            name: "Kansas City Chiefs".to_string(),
            abbreviation: "KC".to_string(),
            city: "Kansas City".to_string(),
            conference: Conference::AFC,
            division: Division::AFCWest,
        };

        let _ = create_team(State(state.clone()), Json(team1))
            .await
            .unwrap();
        let _ = create_team(State(state.clone()), Json(team2))
            .await
            .unwrap();

        // List all teams
        let result = list_teams(State(state)).await;
        assert!(result.is_ok());

        let teams = result.unwrap().0;
        assert_eq!(teams.len(), 2);
    }
}
