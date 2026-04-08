use axum::extract::{Path, State};
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Public-facing shape of a `ProspectProfile`. Mirrors the domain model with
/// `serde_json::Value` for the JSONB columns so the frontend can render flexibly.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProspectProfileResponse {
    pub id: Uuid,
    pub player_id: Uuid,
    pub source: String,
    pub grade_tier: Option<String>,
    pub overall_rank: Option<i32>,
    pub position_rank: i32,
    pub year_class: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub jersey_number: Option<String>,
    pub height_raw: Option<String>,
    pub nfl_comparison: Option<String>,
    pub background: Option<String>,
    pub summary: Option<String>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub college_stats: Option<JsonValue>,
    pub scraped_at: NaiveDate,
}

impl From<domain::models::ProspectProfile> for ProspectProfileResponse {
    fn from(p: domain::models::ProspectProfile) -> Self {
        Self {
            id: p.id,
            player_id: p.player_id,
            source: p.source,
            grade_tier: p.grade_tier,
            overall_rank: p.overall_rank,
            position_rank: p.position_rank,
            year_class: p.year_class,
            birthday: p.birthday,
            jersey_number: p.jersey_number,
            height_raw: p.height_raw,
            nfl_comparison: p.nfl_comparison,
            background: p.background,
            summary: p.summary,
            strengths: p.strengths,
            weaknesses: p.weaknesses,
            college_stats: p.college_stats,
            scraped_at: p.scraped_at,
        }
    }
}

/// GET /api/v1/players/{player_id}/profile
///
/// Returns the most recently scraped prospect profile for the given player,
/// or 404 if no profile exists. Currently the only source is "the-beast-2026".
pub async fn get_player_profile(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> ApiResult<Json<ProspectProfileResponse>> {
    let profile = state
        .prospect_profile_repo
        .find_latest_by_player(player_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("No prospect profile found for player {}", player_id))
        })?;

    Ok(Json(profile.into()))
}
