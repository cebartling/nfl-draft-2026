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
