use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListProfilesQuery {
    /// Source slug, e.g. `the-beast-2026`. Defaults to the-beast-2026.
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "the-beast-2026".to_string()
}

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

/// Compact profile shape used by the bulk list endpoint. Drops the heavy
/// prose fields (background/summary/strengths/weaknesses) to keep the
/// player-list payload small while still surfacing the grade tier and
/// overall rank for badge rendering.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProspectProfileSummary {
    pub player_id: Uuid,
    pub source: String,
    pub grade_tier: Option<String>,
    pub overall_rank: Option<i32>,
    pub position_rank: i32,
    pub nfl_comparison: Option<String>,
}

impl From<&domain::models::ProspectProfile> for ProspectProfileSummary {
    fn from(p: &domain::models::ProspectProfile) -> Self {
        Self {
            player_id: p.player_id,
            source: p.source.clone(),
            grade_tier: p.grade_tier.clone(),
            overall_rank: p.overall_rank,
            position_rank: p.position_rank,
            nfl_comparison: p.nfl_comparison.clone(),
        }
    }
}

/// GET /api/v1/prospect-profiles?source=the-beast-2026
///
/// Returns lightweight summaries for every profile from a given source. Used
/// by the player list and prospects rankings page to render grade-tier badges
/// without paying the cost of fetching prose for every prospect.
pub async fn list_prospect_profiles(
    State(state): State<AppState>,
    Query(q): Query<ListProfilesQuery>,
) -> ApiResult<Json<Vec<ProspectProfileSummary>>> {
    let profiles = state.prospect_profile_repo.find_by_source(&q.source).await?;
    let response: Vec<ProspectProfileSummary> = profiles.iter().map(Into::into).collect();
    Ok(Json(response))
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
