use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PercentileQuery {
    pub position: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CombinePercentileResponse {
    pub id: Uuid,
    pub position: String,
    pub measurement: String,
    pub sample_size: i32,
    pub min_value: f64,
    pub p10: f64,
    pub p20: f64,
    pub p30: f64,
    pub p40: f64,
    pub p50: f64,
    pub p60: f64,
    pub p70: f64,
    pub p80: f64,
    pub p90: f64,
    pub max_value: f64,
    pub years_start: i32,
    pub years_end: i32,
}

impl From<domain::models::CombinePercentile> for CombinePercentileResponse {
    fn from(p: domain::models::CombinePercentile) -> Self {
        Self {
            id: p.id,
            position: p.position,
            measurement: p.measurement.to_string(),
            sample_size: p.sample_size,
            min_value: p.min_value,
            p10: p.p10,
            p20: p.p20,
            p30: p.p30,
            p40: p.p40,
            p50: p.p50,
            p60: p.p60,
            p70: p.p70,
            p80: p.p80,
            p90: p.p90,
            max_value: p.max_value,
            years_start: p.years_start,
            years_end: p.years_end,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertPercentileRequest {
    pub position: String,
    pub measurement: String,
    pub sample_size: i32,
    pub min_value: f64,
    pub p10: f64,
    pub p20: f64,
    pub p30: f64,
    pub p40: f64,
    pub p50: f64,
    pub p60: f64,
    pub p70: f64,
    pub p80: f64,
    pub p90: f64,
    pub max_value: f64,
    #[serde(default = "default_years_start")]
    pub years_start: i32,
    #[serde(default = "default_years_end")]
    pub years_end: i32,
}

fn default_years_start() -> i32 {
    2000
}

fn default_years_end() -> i32 {
    2025
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkUpsertPercentilesRequest {
    pub percentiles: Vec<UpsertPercentileRequest>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkUpsertResponse {
    pub message: String,
    pub upserted_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// GET /api/v1/combine-percentiles - Get combine percentiles, optionally filtered by position
#[utoipa::path(
    get,
    path = "/api/v1/combine-percentiles",
    responses(
        (status = 200, description = "List of combine percentiles", body = Vec<CombinePercentileResponse>)
    ),
    params(
        ("position" = Option<String>, Query, description = "Filter by position (e.g., QB, WR)")
    ),
    tag = "combine-percentiles"
)]
pub async fn get_combine_percentiles(
    State(state): State<AppState>,
    Query(query): Query<PercentileQuery>,
) -> ApiResult<Json<Vec<CombinePercentileResponse>>> {
    let results = match &query.position {
        Some(position) => {
            state
                .combine_percentile_repo
                .find_by_position(position)
                .await?
        }
        None => state.combine_percentile_repo.find_all().await?,
    };

    let response: Vec<CombinePercentileResponse> =
        results.into_iter().map(CombinePercentileResponse::from).collect();

    Ok(Json(response))
}

/// POST /api/v1/admin/seed-percentiles - Bulk upsert combine percentile data
#[utoipa::path(
    post,
    path = "/api/v1/admin/seed-percentiles",
    request_body = BulkUpsertPercentilesRequest,
    responses(
        (status = 200, description = "Percentiles seeded successfully", body = BulkUpsertResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "admin"
)]
pub async fn seed_percentiles(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<BulkUpsertPercentilesRequest>,
) -> ApiResult<Json<BulkUpsertResponse>> {
    // Validate seed API key
    let expected_key = match &state.seed_api_key {
        Some(key) => key.clone(),
        None => {
            return Err(ApiError::NotFound("Not found".to_string()));
        }
    };

    let provided_key = headers
        .get("X-Seed-Api-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided_key != expected_key {
        return Err(ApiError::Unauthorized(
            "Invalid or missing API key".to_string(),
        ));
    }

    let mut upserted_count = 0;
    let mut errors: Vec<String> = Vec::new();

    for item in &req.percentiles {
        let measurement = match item.measurement.parse::<domain::models::Measurement>() {
            Ok(m) => m,
            Err(e) => {
                errors.push(format!(
                    "Invalid measurement '{}' for {}: {}",
                    item.measurement, item.position, e
                ));
                continue;
            }
        };

        let percentile = match domain::models::CombinePercentile::new(
            item.position.clone(),
            measurement,
        ) {
            Ok(p) => p,
            Err(e) => {
                errors.push(format!(
                    "Invalid position '{}': {}",
                    item.position, e
                ));
                continue;
            }
        };

        let percentile = match percentile
            .with_percentiles(
                item.sample_size,
                item.min_value,
                item.p10,
                item.p20,
                item.p30,
                item.p40,
                item.p50,
                item.p60,
                item.p70,
                item.p80,
                item.p90,
                item.max_value,
            )
            .and_then(|p| p.with_years(item.years_start, item.years_end))
        {
            Ok(p) => p,
            Err(e) => {
                errors.push(format!(
                    "Validation error for {} {}: {}",
                    item.position, item.measurement, e
                ));
                continue;
            }
        };

        match state.combine_percentile_repo.upsert(&percentile).await {
            Ok(_) => upserted_count += 1,
            Err(e) => {
                errors.push(format!(
                    "Database error for {} {}: {}",
                    item.position, item.measurement, e
                ));
            }
        }
    }

    let message = format!(
        "Percentile seeding complete: {} upserted, {} errors",
        upserted_count,
        errors.len()
    );

    Ok(Json(BulkUpsertResponse {
        message,
        upserted_count,
        error_count: errors.len(),
        errors,
    }))
}

/// DELETE /api/v1/admin/percentiles - Delete all percentile data
#[utoipa::path(
    delete,
    path = "/api/v1/admin/percentiles",
    responses(
        (status = 200, description = "Percentiles deleted", body = BulkUpsertResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "admin"
)]
pub async fn delete_all_percentiles(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<BulkUpsertResponse>> {
    let expected_key = match &state.seed_api_key {
        Some(key) => key.clone(),
        None => {
            return Err(ApiError::NotFound("Not found".to_string()));
        }
    };

    let provided_key = headers
        .get("X-Seed-Api-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided_key != expected_key {
        return Err(ApiError::Unauthorized(
            "Invalid or missing API key".to_string(),
        ));
    }

    let deleted = state.combine_percentile_repo.delete_all().await?;

    Ok(Json(BulkUpsertResponse {
        message: format!("Deleted {} percentile records", deleted),
        upserted_count: 0,
        error_count: 0,
        errors: vec![],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_response_from_domain() {
        let p = domain::models::CombinePercentile::new(
            "QB".to_string(),
            domain::models::Measurement::FortyYardDash,
        )
        .unwrap()
        .with_percentiles(100, 4.2, 4.3, 4.35, 4.4, 4.45, 4.5, 4.55, 4.6, 4.65, 4.7, 5.0)
        .unwrap();

        let resp = CombinePercentileResponse::from(p);
        assert_eq!(resp.position, "QB");
        assert_eq!(resp.measurement, "forty_yard_dash");
        assert_eq!(resp.sample_size, 100);
        assert_eq!(resp.p50, 4.5);
    }
}
