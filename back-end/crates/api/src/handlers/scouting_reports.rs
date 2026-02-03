use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use domain::models::{FitGrade, ScoutingReport};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateScoutingReportRequest {
    pub player_id: Uuid,
    pub team_id: Uuid,
    pub grade: f64,
    pub notes: Option<String>,
    pub fit_grade: Option<FitGrade>,
    pub injury_concern: Option<bool>,
    pub character_concern: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateScoutingReportRequest {
    pub grade: Option<f64>,
    pub notes: Option<String>,
    pub fit_grade: Option<FitGrade>,
    pub injury_concern: Option<bool>,
    pub character_concern: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScoutingReportResponse {
    pub id: Uuid,
    pub player_id: Uuid,
    pub team_id: Uuid,
    pub grade: f64,
    pub notes: Option<String>,
    pub fit_grade: Option<FitGrade>,
    pub injury_concern: bool,
    pub character_concern: bool,
}

impl From<ScoutingReport> for ScoutingReportResponse {
    fn from(report: ScoutingReport) -> Self {
        Self {
            id: report.id,
            player_id: report.player_id,
            team_id: report.team_id,
            grade: report.grade,
            notes: report.notes,
            fit_grade: report.fit_grade,
            injury_concern: report.injury_concern,
            character_concern: report.character_concern,
        }
    }
}

/// POST /api/v1/scouting-reports - Create new scouting report
#[utoipa::path(
    post,
    path = "/api/v1/scouting-reports",
    request_body = CreateScoutingReportRequest,
    responses(
        (status = 201, description = "Scouting report created successfully", body = ScoutingReportResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Scouting report for this team and player already exists")
    ),
    tag = "scouting-reports"
)]
pub async fn create_scouting_report(
    State(state): State<AppState>,
    Json(req): Json<CreateScoutingReportRequest>,
) -> ApiResult<(StatusCode, Json<ScoutingReportResponse>)> {
    let mut report = ScoutingReport::new(req.player_id, req.team_id, req.grade)?;

    if let Some(fit_grade) = req.fit_grade {
        report = report.with_fit_grade(fit_grade);
    }
    if let Some(notes) = req.notes {
        report = report.with_notes(notes)?;
    }
    if let Some(concern) = req.injury_concern {
        report = report.with_injury_concern(concern);
    }
    if let Some(concern) = req.character_concern {
        report = report.with_character_concern(concern);
    }

    let created = state.scouting_report_repo.create(&report).await?;

    Ok((
        StatusCode::CREATED,
        Json(ScoutingReportResponse::from(created)),
    ))
}

/// GET /api/v1/scouting-reports/:id - Get scouting report by ID
#[utoipa::path(
    get,
    path = "/api/v1/scouting-reports/{id}",
    responses(
        (status = 200, description = "Scouting report found", body = ScoutingReportResponse),
        (status = 404, description = "Scouting report not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Scouting report ID")
    ),
    tag = "scouting-reports"
)]
pub async fn get_scouting_report(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ScoutingReportResponse>> {
    let report = state
        .scouting_report_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Scouting report with id {} not found", id)))?;

    Ok(Json(ScoutingReportResponse::from(report)))
}

/// GET /api/v1/teams/:team_id/scouting-reports - Get all scouting reports for a team
#[utoipa::path(
    get,
    path = "/api/v1/teams/{team_id}/scouting-reports",
    responses(
        (status = 200, description = "List of scouting reports for team", body = Vec<ScoutingReportResponse>)
    ),
    params(
        ("team_id" = Uuid, Path, description = "Team ID")
    ),
    tag = "scouting-reports"
)]
pub async fn get_team_scouting_reports(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
) -> ApiResult<Json<Vec<ScoutingReportResponse>>> {
    let reports = state.scouting_report_repo.find_by_team_id(team_id).await?;
    let response: Vec<ScoutingReportResponse> = reports
        .into_iter()
        .map(ScoutingReportResponse::from)
        .collect();
    Ok(Json(response))
}

/// GET /api/v1/players/:player_id/scouting-reports - Get all scouting reports for a player
#[utoipa::path(
    get,
    path = "/api/v1/players/{player_id}/scouting-reports",
    responses(
        (status = 200, description = "List of scouting reports for player", body = Vec<ScoutingReportResponse>)
    ),
    params(
        ("player_id" = Uuid, Path, description = "Player ID")
    ),
    tag = "scouting-reports"
)]
pub async fn get_player_scouting_reports(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> ApiResult<Json<Vec<ScoutingReportResponse>>> {
    let reports = state
        .scouting_report_repo
        .find_by_player_id(player_id)
        .await?;
    let response: Vec<ScoutingReportResponse> = reports
        .into_iter()
        .map(ScoutingReportResponse::from)
        .collect();
    Ok(Json(response))
}

/// PUT /api/v1/scouting-reports/:id - Update scouting report
#[utoipa::path(
    put,
    path = "/api/v1/scouting-reports/{id}",
    request_body = UpdateScoutingReportRequest,
    responses(
        (status = 200, description = "Scouting report updated successfully", body = ScoutingReportResponse),
        (status = 404, description = "Scouting report not found"),
        (status = 400, description = "Invalid request")
    ),
    params(
        ("id" = Uuid, Path, description = "Scouting report ID")
    ),
    tag = "scouting-reports"
)]
pub async fn update_scouting_report(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateScoutingReportRequest>,
) -> ApiResult<Json<ScoutingReportResponse>> {
    let mut report = state
        .scouting_report_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Scouting report with id {} not found", id)))?;

    // Update fields with validation
    if let Some(grade) = req.grade {
        report.update_grade(grade)?;
    }
    if let Some(notes) = req.notes {
        report.update_notes(notes)?;
    }
    if let Some(fit_grade) = req.fit_grade {
        report.update_fit_grade(fit_grade)?;
    }
    if let Some(concern) = req.injury_concern {
        report.update_injury_concern(concern)?;
    }
    if let Some(concern) = req.character_concern {
        report.update_character_concern(concern)?;
    }

    let updated = state.scouting_report_repo.update(&report).await?;

    Ok(Json(ScoutingReportResponse::from(updated)))
}

/// DELETE /api/v1/scouting-reports/:id - Delete scouting report
#[utoipa::path(
    delete,
    path = "/api/v1/scouting-reports/{id}",
    responses(
        (status = 204, description = "Scouting report deleted successfully"),
        (status = 404, description = "Scouting report not found")
    ),
    params(
        ("id" = Uuid, Path, description = "Scouting report ID")
    ),
    tag = "scouting-reports"
)]
pub async fn delete_scouting_report(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.scouting_report_repo.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use domain::models::{Conference, Division, Player, Position, Team};
    use domain::repositories::{PlayerRepository, TeamRepository};

    async fn setup_test_state() -> AppState {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        let pool = db::create_pool(&database_url)
            .await
            .expect("Failed to create pool");

        AppState::new(pool)
    }

    async fn create_test_player(state: &AppState) -> Player {
        let player =
            Player::new("Test".to_string(), "Player".to_string(), Position::QB, 2026).unwrap();
        state.player_repo.create(&player).await.unwrap()
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
    async fn test_create_scouting_report() {
        let state = setup_test_state().await;

        let player = create_test_player(&state).await;
        let team = create_test_team(&state, "SR1").await;

        let request = CreateScoutingReportRequest {
            player_id: player.id,
            team_id: team.id,
            grade: 8.5,
            notes: Some("Excellent prospect".to_string()),
            fit_grade: Some(FitGrade::A),
            injury_concern: Some(false),
            character_concern: Some(false),
        };

        let result = create_scouting_report(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.player_id, player.id);
        assert_eq!(response.team_id, team.id);
        assert_eq!(response.grade, 8.5);
        assert_eq!(response.fit_grade, Some(FitGrade::A));
    }

    #[tokio::test]
    async fn test_get_scouting_report() {
        let state = setup_test_state().await;

        let player = create_test_player(&state).await;
        let team = create_test_team(&state, "SR2").await;

        let request = CreateScoutingReportRequest {
            player_id: player.id,
            team_id: team.id,
            grade: 8.5,
            notes: None,
            fit_grade: None,
            injury_concern: None,
            character_concern: None,
        };

        let (_, created_response) = create_scouting_report(State(state.clone()), Json(request))
            .await
            .unwrap();

        let result = get_scouting_report(State(state.clone()), Path(created_response.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.id, created_response.id);
    }

    #[tokio::test]
    async fn test_get_team_scouting_reports() {
        let state = setup_test_state().await;

        let player1 = create_test_player(&state).await;
        let player_repo = &state.player_repo;
        let player2 = player_repo
            .create(
                &Player::new(
                    "Second".to_string(),
                    "Player".to_string(),
                    Position::RB,
                    2026,
                )
                .unwrap(),
            )
            .await
            .unwrap();

        let team = create_test_team(&state, "SR3").await;

        let request1 = CreateScoutingReportRequest {
            player_id: player1.id,
            team_id: team.id,
            grade: 9.0,
            notes: None,
            fit_grade: None,
            injury_concern: None,
            character_concern: None,
        };

        let request2 = CreateScoutingReportRequest {
            player_id: player2.id,
            team_id: team.id,
            grade: 7.5,
            notes: None,
            fit_grade: None,
            injury_concern: None,
            character_concern: None,
        };

        create_scouting_report(State(state.clone()), Json(request1))
            .await
            .unwrap();
        create_scouting_report(State(state.clone()), Json(request2))
            .await
            .unwrap();

        let result = get_team_scouting_reports(State(state.clone()), Path(team.id)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.len(), 2);
    }
}
