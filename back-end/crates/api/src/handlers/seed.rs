use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const PLAYERS_2026_JSON: &str = include_str!("../../../../data/players_2026.json");
const TEAMS_NFL_JSON: &str = include_str!("../../../../data/teams_nfl.json");
const TEAM_SEASONS_2025_JSON: &str = include_str!("../../../../data/team_seasons_2025.json");

#[derive(Debug, Serialize, ToSchema)]
pub struct SeedResponse {
    pub message: String,
    pub success_count: usize,
    pub skipped_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
    pub validation_warnings: Vec<String>,
}

/// Seed the database with embedded 2026 player data
///
/// Requires the `X-Seed-Api-Key` header matching the server's `SEED_API_KEY` environment variable.
/// Returns 404 if `SEED_API_KEY` is not configured (endpoint is hidden).
#[utoipa::path(
    post,
    path = "/api/v1/admin/seed-players",
    tag = "admin",
    responses(
        (status = 200, description = "Players seeded successfully", body = SeedResponse),
        (status = 401, description = "Unauthorized - invalid or missing API key"),
        (status = 404, description = "Not found - endpoint not enabled"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn seed_players(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<SeedResponse>> {
    // If SEED_API_KEY is not configured, hide the endpoint entirely
    let expected_key = match &state.seed_api_key {
        Some(key) => key,
        None => {
            return Err(ApiError::NotFound("Not found".to_string()));
        }
    };

    // Validate the API key from the request header
    let provided_key = headers
        .get("X-Seed-Api-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided_key != expected_key {
        return Err(ApiError::Unauthorized(
            "Invalid or missing API key".to_string(),
        ));
    }

    // Parse the embedded player data
    let data = seed_data::loader::parse_player_json(PLAYERS_2026_JSON).map_err(|e| {
        ApiError::InternalError(format!("Failed to parse embedded player data: {}", e))
    })?;

    // Validate the data
    let validation = seed_data::validator::validate_player_data(&data);
    let validation_warnings = validation.warnings;

    if !validation.valid {
        return Ok(Json(SeedResponse {
            message: "Seeding aborted due to validation errors".to_string(),
            success_count: 0,
            skipped_count: 0,
            error_count: validation.errors.len(),
            errors: validation.errors,
            validation_warnings,
        }));
    }

    // Load players into the database
    let stats = seed_data::loader::load_players(&data, state.player_repo.as_ref())
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to load players: {}", e)))?;

    let message = format!(
        "Seeding complete: {} succeeded, {} skipped, {} errors",
        stats.success,
        stats.skipped,
        stats.errors.len()
    );

    Ok(Json(SeedResponse {
        message,
        success_count: stats.success,
        skipped_count: stats.skipped,
        error_count: stats.errors.len(),
        errors: stats.errors,
        validation_warnings,
    }))
}

/// Seed the database with embedded NFL team data
///
/// Requires the `X-Seed-Api-Key` header matching the server's `SEED_API_KEY` environment variable.
/// Returns 404 if `SEED_API_KEY` is not configured (endpoint is hidden).
#[utoipa::path(
    post,
    path = "/api/v1/admin/seed-teams",
    tag = "admin",
    responses(
        (status = 200, description = "Teams seeded successfully", body = SeedResponse),
        (status = 401, description = "Unauthorized - invalid or missing API key"),
        (status = 404, description = "Not found - endpoint not enabled"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn seed_teams(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<SeedResponse>> {
    // If SEED_API_KEY is not configured, hide the endpoint entirely
    let expected_key = match &state.seed_api_key {
        Some(key) => key,
        None => {
            return Err(ApiError::NotFound("Not found".to_string()));
        }
    };

    // Validate the API key from the request header
    let provided_key = headers
        .get("X-Seed-Api-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided_key != expected_key {
        return Err(ApiError::Unauthorized(
            "Invalid or missing API key".to_string(),
        ));
    }

    // Parse the embedded team data
    let data = seed_data::team_loader::parse_team_json(TEAMS_NFL_JSON).map_err(|e| {
        ApiError::InternalError(format!("Failed to parse embedded team data: {}", e))
    })?;

    // Validate the data
    let validation = seed_data::team_validator::validate_team_data(&data);
    let validation_warnings = validation.warnings;

    if !validation.valid {
        return Ok(Json(SeedResponse {
            message: "Seeding aborted due to validation errors".to_string(),
            success_count: 0,
            skipped_count: 0,
            error_count: validation.errors.len(),
            errors: validation.errors,
            validation_warnings,
        }));
    }

    // Load teams into the database
    let stats = seed_data::team_loader::load_teams(&data, state.team_repo.as_ref())
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to load teams: {}", e)))?;

    let message = format!(
        "Seeding complete: {} succeeded, {} skipped, {} errors",
        stats.success,
        stats.skipped,
        stats.errors.len()
    );

    Ok(Json(SeedResponse {
        message,
        success_count: stats.success,
        skipped_count: stats.skipped,
        error_count: stats.errors.len(),
        errors: stats.errors,
        validation_warnings,
    }))
}

/// Seed the database with embedded 2025 team season data
///
/// Requires the `X-Seed-Api-Key` header matching the server's `SEED_API_KEY` environment variable.
/// Returns 404 if `SEED_API_KEY` is not configured (endpoint is hidden).
#[utoipa::path(
    post,
    path = "/api/v1/admin/seed-team-seasons",
    tag = "admin",
    responses(
        (status = 200, description = "Team seasons seeded successfully", body = SeedResponse),
        (status = 401, description = "Unauthorized - invalid or missing API key"),
        (status = 404, description = "Not found - endpoint not enabled"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn seed_team_seasons(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<SeedResponse>> {
    // If SEED_API_KEY is not configured, hide the endpoint entirely
    let expected_key = match &state.seed_api_key {
        Some(key) => key,
        None => {
            return Err(ApiError::NotFound("Not found".to_string()));
        }
    };

    // Validate the API key from the request header
    let provided_key = headers
        .get("X-Seed-Api-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided_key != expected_key {
        return Err(ApiError::Unauthorized(
            "Invalid or missing API key".to_string(),
        ));
    }

    // Parse the embedded team season data
    let data = seed_data::team_season_loader::parse_team_season_json(TEAM_SEASONS_2025_JSON)
        .map_err(|e| {
            ApiError::InternalError(format!("Failed to parse embedded team season data: {}", e))
        })?;

    // Validate the data
    let validation = seed_data::team_season_validator::validate_team_season_data(&data);
    let validation_warnings = validation.warnings;

    if !validation.valid {
        return Ok(Json(SeedResponse {
            message: "Seeding aborted due to validation errors".to_string(),
            success_count: 0,
            skipped_count: 0,
            error_count: validation.errors.len(),
            errors: validation.errors,
            validation_warnings,
        }));
    }

    // Load team seasons into the database
    let stats = seed_data::team_season_loader::load_team_seasons(
        &data,
        state.team_repo.as_ref(),
        state.team_season_repo.as_ref(),
    )
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to load team seasons: {}", e)))?;

    let message = format!(
        "Seeding complete: {} processed, {} created, {} updated, {} errors",
        stats.seasons_processed,
        stats.seasons_created,
        stats.seasons_updated,
        stats.errors.len()
    );

    Ok(Json(SeedResponse {
        message,
        success_count: stats.seasons_created + stats.seasons_updated,
        skipped_count: stats.teams_skipped,
        error_count: stats.errors.len(),
        errors: stats.errors,
        validation_warnings,
    }))
}
