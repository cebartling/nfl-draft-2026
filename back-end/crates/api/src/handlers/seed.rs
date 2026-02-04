use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const PLAYERS_2026_JSON: &str = include_str!("../../../../data/players_2026.json");

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
