use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

/// Rich prospect profile sourced from a single scouting publication
/// (e.g. Dane Brugler's "The Beast 2026"). Holds the prose, bullets,
/// year-by-year stats, and metadata that don't fit the team-keyed
/// `ScoutingReport` model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectProfile {
    pub id: Uuid,
    pub player_id: Uuid,
    /// Slug identifying the source, e.g. `the-beast-2026`.
    pub source: String,
    /// Free-text grade tier from the source: `1st`, `4th-5th`, `7th-FA`, `FA`, ...
    pub grade_tier: Option<String>,
    /// Overall rank across all positions when the source publishes one
    /// (e.g. The Beast Top 100). `None` for prospects outside the list.
    pub overall_rank: Option<i32>,
    /// Rank within position group (QB1 -> 1).
    pub position_rank: i32,
    /// Year class label as published, e.g. `4JR`, `5SR`, `7SR`.
    pub year_class: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub jersey_number: Option<String>,
    /// Raw 4-digit height encoding from The Beast (`6046` = 6'4 6/8").
    /// Preserved verbatim for traceability; the parsed value lives on `players.height_inches`.
    pub height_raw: Option<String>,
    pub nfl_comparison: Option<String>,
    pub background: Option<String>,
    pub summary: Option<String>,
    /// Bullet list of strengths.
    pub strengths: Vec<String>,
    /// Bullet list of weaknesses.
    pub weaknesses: Vec<String>,
    /// Year-by-year college statistics as JSON (shape varies by position).
    pub college_stats: Option<JsonValue>,
    pub scraped_at: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProspectProfile {
    pub fn new(
        player_id: Uuid,
        source: String,
        position_rank: i32,
        scraped_at: NaiveDate,
    ) -> DomainResult<Self> {
        Self::validate_source(&source)?;
        Self::validate_position_rank(position_rank)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            player_id,
            source,
            grade_tier: None,
            overall_rank: None,
            position_rank,
            year_class: None,
            birthday: None,
            jersey_number: None,
            height_raw: None,
            nfl_comparison: None,
            background: None,
            summary: None,
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            college_stats: None,
            scraped_at,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_overall_rank(mut self, rank: i32) -> DomainResult<Self> {
        if rank <= 0 {
            return Err(DomainError::ValidationError(format!(
                "Overall rank must be positive, got {}",
                rank
            )));
        }
        self.overall_rank = Some(rank);
        Ok(self)
    }

    pub fn with_grade_tier(mut self, tier: String) -> Self {
        self.grade_tier = Some(tier);
        self
    }

    pub fn with_year_class(mut self, year_class: String) -> Self {
        self.year_class = Some(year_class);
        self
    }

    pub fn with_birthday(mut self, birthday: NaiveDate) -> Self {
        self.birthday = Some(birthday);
        self
    }

    pub fn with_jersey_number(mut self, jersey: String) -> Self {
        self.jersey_number = Some(jersey);
        self
    }

    pub fn with_height_raw(mut self, height_raw: String) -> Self {
        self.height_raw = Some(height_raw);
        self
    }

    pub fn with_nfl_comparison(mut self, comparison: String) -> Self {
        self.nfl_comparison = Some(comparison);
        self
    }

    pub fn with_background(mut self, background: String) -> Self {
        self.background = Some(background);
        self
    }

    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }

    pub fn with_strengths(mut self, strengths: Vec<String>) -> Self {
        self.strengths = strengths;
        self
    }

    pub fn with_weaknesses(mut self, weaknesses: Vec<String>) -> Self {
        self.weaknesses = weaknesses;
        self
    }

    pub fn with_college_stats(mut self, stats: JsonValue) -> Self {
        self.college_stats = Some(stats);
        self
    }

    fn validate_source(source: &str) -> DomainResult<()> {
        if source.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Source cannot be empty".to_string(),
            ));
        }
        if source.len() > 50 {
            return Err(DomainError::ValidationError(
                "Source cannot exceed 50 characters".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_position_rank(rank: i32) -> DomainResult<()> {
        if rank <= 0 {
            return Err(DomainError::ValidationError(format!(
                "Position rank must be positive, got {}",
                rank
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ProspectProfile {
        ProspectProfile::new(
            Uuid::new_v4(),
            "the-beast-2026".to_string(),
            1,
            NaiveDate::from_ymd_opt(2026, 4, 2).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn test_new_profile_defaults() {
        let p = fixture();
        assert_eq!(p.source, "the-beast-2026");
        assert_eq!(p.position_rank, 1);
        assert!(p.overall_rank.is_none());
        assert!(p.strengths.is_empty());
        assert!(p.weaknesses.is_empty());
    }

    #[test]
    fn test_with_overall_rank_validates() {
        assert!(fixture().with_overall_rank(0).is_err());
        assert!(fixture().with_overall_rank(-3).is_err());
        let p = fixture().with_overall_rank(3).unwrap();
        assert_eq!(p.overall_rank, Some(3));
    }

    #[test]
    fn test_invalid_source() {
        let player_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 4, 2).unwrap();
        assert!(ProspectProfile::new(player_id, String::new(), 1, date).is_err());
        assert!(ProspectProfile::new(player_id, "x".repeat(51), 1, date).is_err());
    }

    #[test]
    fn test_invalid_position_rank() {
        let player_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 4, 2).unwrap();
        assert!(
            ProspectProfile::new(player_id, "the-beast-2026".to_string(), 0, date).is_err()
        );
    }

    #[test]
    fn test_builder_chain() {
        let p = fixture()
            .with_grade_tier("1st".to_string())
            .with_overall_rank(3)
            .unwrap()
            .with_year_class("4JR".to_string())
            .with_jersey_number("15".to_string())
            .with_nfl_comparison("Bernie Kosar".to_string())
            .with_strengths(vec!["Tall".to_string(), "Smart".to_string()])
            .with_weaknesses(vec!["Linear athlete".to_string()]);
        assert_eq!(p.grade_tier.as_deref(), Some("1st"));
        assert_eq!(p.overall_rank, Some(3));
        assert_eq!(p.strengths.len(), 2);
        assert_eq!(p.weaknesses.len(), 1);
        assert_eq!(p.nfl_comparison.as_deref(), Some("Bernie Kosar"));
    }
}
