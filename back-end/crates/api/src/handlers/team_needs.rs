use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use domain::models::{Position, TeamNeed};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTeamNeedRequest {
    pub team_id: Uuid,
    pub position: Position,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTeamNeedRequest {
    pub priority: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TeamNeedResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub position: Position,
    pub priority: i32,
}

impl From<TeamNeed> for TeamNeedResponse {
    fn from(need: TeamNeed) -> Self {
        Self {
            id: need.id,
            team_id: need.team_id,
            position: need.position,
            priority: need.priority,
        }
    }
}

/// POST /api/v1/team-needs - Create new team need
#[utoipa::path(
    post,
    path = "/api/v1/team-needs",
    request_body = CreateTeamNeedRequest,
    responses(
        (status = 201, description = "Team need created successfully", body = TeamNeedResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Team need for this team and position already exists")
    ),
    tag = "team-needs"
)]
pub async fn create_team_need(
    State(state): State<AppState>,
    Json(req): Json<CreateTeamNeedRequest>,
) -> ApiResult<(StatusCode, Json<TeamNeedResponse>)> {
    let need = TeamNeed::new(req.team_id, req.position, req.priority)?;

    let created = state.team_need_repo.create(&need).await?;

    Ok((StatusCode::CREATED, Json(TeamNeedResponse::from(created))))
}

/// GET /api/v1/team-needs/:id - Get team need by ID
#[utoipa::path(
    get,
    path = "/api/v1/team-needs/{id}",
    responses(
        (status = 200, description = "Team need found", body = TeamNeedResponse),
        (status = 404, description = "Team need not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Team need ID")
    ),
    tag = "team-needs"
)]
pub async fn get_team_need(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TeamNeedResponse>> {
    let need = state
        .team_need_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Team need with id {} not found", id)))?;

    Ok(Json(TeamNeedResponse::from(need)))
}

/// GET /api/v1/teams/:team_id/needs - Get all needs for a team
#[utoipa::path(
    get,
    path = "/api/v1/teams/{team_id}/needs",
    responses(
        (status = 200, description = "List of needs for team", body = Vec<TeamNeedResponse>)
    ),
    params(
        ("team_id" = Uuid, Path, description = "Team ID")
    ),
    tag = "team-needs"
)]
pub async fn list_team_needs(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
) -> ApiResult<Json<Vec<TeamNeedResponse>>> {
    let needs = state.team_need_repo.find_by_team_id(team_id).await?;
    let response: Vec<TeamNeedResponse> = needs.into_iter().map(TeamNeedResponse::from).collect();
    Ok(Json(response))
}

/// PUT /api/v1/team-needs/:id - Update team need
#[utoipa::path(
    put,
    path = "/api/v1/team-needs/{id}",
    request_body = UpdateTeamNeedRequest,
    responses(
        (status = 200, description = "Team need updated successfully", body = TeamNeedResponse),
        (status = 404, description = "Team need not found"),
        (status = 400, description = "Invalid request")
    ),
    params(
        ("id" = Uuid, Path, description = "Team need ID")
    ),
    tag = "team-needs"
)]
pub async fn update_team_need(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTeamNeedRequest>,
) -> ApiResult<Json<TeamNeedResponse>> {
    let mut need = state
        .team_need_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Team need with id {} not found", id)))?;

    // Update priority with validation
    need.update_priority(req.priority)?;

    let updated = state.team_need_repo.update(&need).await?;

    Ok(Json(TeamNeedResponse::from(updated)))
}

/// DELETE /api/v1/team-needs/:id - Delete team need
#[utoipa::path(
    delete,
    path = "/api/v1/team-needs/{id}",
    responses(
        (status = 204, description = "Team need deleted successfully"),
        (status = 404, description = "Team need not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Team need ID")
    ),
    tag = "team-needs"
)]
pub async fn delete_team_need(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.team_need_repo.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use domain::models::{Conference, Division, Team};

    async fn setup_test_state() -> AppState {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        let pool = db::create_pool(&database_url)
            .await
            .expect("Failed to create pool");

        AppState::new(pool, None)
    }

    async fn create_test_team(state: &AppState, abbr: &str) -> Team {
        let team = Team::new(
            format!("Test Team {}", abbr),
            abbr.to_string(),
            "Test City".to_string(),
            Conference::AFC,
            Division::AFCEast,
        )
        .unwrap();
        state.team_repo.create(&team).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_team_need() {
        let state = setup_test_state().await;

        let team = create_test_team(&state, "TN1").await;

        let request = CreateTeamNeedRequest {
            team_id: team.id,
            position: Position::QB,
            priority: 10,
        };

        let result = create_team_need(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.team_id, team.id);
        assert_eq!(response.position, Position::QB);
        assert_eq!(response.priority, 10);
    }

    #[tokio::test]
    async fn test_get_team_need() {
        let state = setup_test_state().await;

        let team = create_test_team(&state, "TN2").await;

        let request = CreateTeamNeedRequest {
            team_id: team.id,
            position: Position::QB,
            priority: 10,
        };

        let (_, created_response) = create_team_need(State(state.clone()), Json(request))
            .await
            .unwrap();

        let result = get_team_need(State(state.clone()), Path(created_response.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.id, created_response.id);
    }

    #[tokio::test]
    async fn test_list_team_needs() {
        let state = setup_test_state().await;

        let team = create_test_team(&state, "TN3").await;

        let request1 = CreateTeamNeedRequest {
            team_id: team.id,
            position: Position::QB,
            priority: 10,
        };

        let request2 = CreateTeamNeedRequest {
            team_id: team.id,
            position: Position::WR,
            priority: 8,
        };

        let request3 = CreateTeamNeedRequest {
            team_id: team.id,
            position: Position::DE,
            priority: 5,
        };

        let _ = create_team_need(State(state.clone()), Json(request1))
            .await
            .unwrap();
        let _ = create_team_need(State(state.clone()), Json(request2))
            .await
            .unwrap();
        let _ = create_team_need(State(state.clone()), Json(request3))
            .await
            .unwrap();

        let result = list_team_needs(State(state.clone()), Path(team.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.len(), 3);
        // Should be ordered by priority (lowest number first)
        assert_eq!(response[0].priority, 5);
        assert_eq!(response[1].priority, 8);
        assert_eq!(response[2].priority, 10);
    }

    #[tokio::test]
    async fn test_update_team_need() {
        let state = setup_test_state().await;

        let team = create_test_team(&state, "TN4").await;

        let request = CreateTeamNeedRequest {
            team_id: team.id,
            position: Position::QB,
            priority: 10,
        };

        let (_, created_response) = create_team_need(State(state.clone()), Json(request))
            .await
            .unwrap();

        let update_request = UpdateTeamNeedRequest { priority: 5 };

        let result = update_team_need(
            State(state.clone()),
            Path(created_response.id),
            Json(update_request),
        )
        .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.priority, 5);
    }

    #[tokio::test]
    async fn test_delete_team_need() {
        let state = setup_test_state().await;

        let team = create_test_team(&state, "TN5").await;

        let request = CreateTeamNeedRequest {
            team_id: team.id,
            position: Position::QB,
            priority: 10,
        };

        let (_, created_response) = create_team_need(State(state.clone()), Json(request))
            .await
            .unwrap();

        let result = delete_team_need(State(state.clone()), Path(created_response.id)).await;
        assert!(result.is_ok());

        let status = result.unwrap();
        assert_eq!(status, StatusCode::NO_CONTENT);

        // Verify it's deleted
        let get_result = get_team_need(State(state.clone()), Path(created_response.id)).await;
        assert!(get_result.is_err());
    }
}
