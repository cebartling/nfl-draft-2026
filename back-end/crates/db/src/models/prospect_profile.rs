use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::ProspectProfile;

use crate::errors::{DbError, DbResult};

/// Database model for prospect_profiles table.
#[derive(Debug, Clone, FromRow)]
pub struct ProspectProfileDb {
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
    pub strengths: JsonValue,
    pub weaknesses: JsonValue,
    pub college_stats: Option<JsonValue>,
    pub scraped_at: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProspectProfileDb {
    pub fn from_domain(p: &ProspectProfile) -> Self {
        Self {
            id: p.id,
            player_id: p.player_id,
            source: p.source.clone(),
            grade_tier: p.grade_tier.clone(),
            overall_rank: p.overall_rank,
            position_rank: p.position_rank,
            year_class: p.year_class.clone(),
            birthday: p.birthday,
            jersey_number: p.jersey_number.clone(),
            height_raw: p.height_raw.clone(),
            nfl_comparison: p.nfl_comparison.clone(),
            background: p.background.clone(),
            summary: p.summary.clone(),
            strengths: serde_json::to_value(&p.strengths).unwrap_or_else(|_| JsonValue::Array(vec![])),
            weaknesses: serde_json::to_value(&p.weaknesses).unwrap_or_else(|_| JsonValue::Array(vec![])),
            college_stats: p.college_stats.clone(),
            scraped_at: p.scraped_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }

    pub fn to_domain(&self) -> DbResult<ProspectProfile> {
        let strengths: Vec<String> = serde_json::from_value(self.strengths.clone())
            .map_err(|e| {
                DbError::MappingError(format!("Failed to parse strengths JSONB: {}", e))
            })?;
        let weaknesses: Vec<String> = serde_json::from_value(self.weaknesses.clone())
            .map_err(|e| {
                DbError::MappingError(format!("Failed to parse weaknesses JSONB: {}", e))
            })?;

        Ok(ProspectProfile {
            id: self.id,
            player_id: self.player_id,
            source: self.source.clone(),
            grade_tier: self.grade_tier.clone(),
            overall_rank: self.overall_rank,
            position_rank: self.position_rank,
            year_class: self.year_class.clone(),
            birthday: self.birthday,
            jersey_number: self.jersey_number.clone(),
            height_raw: self.height_raw.clone(),
            nfl_comparison: self.nfl_comparison.clone(),
            background: self.background.clone(),
            summary: self.summary.clone(),
            strengths,
            weaknesses,
            college_stats: self.college_stats.clone(),
            scraped_at: self.scraped_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

