use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::{PlayoffResult, TeamSeason};

use crate::errors::{DbError, DbResult};

/// Database model for team_seasons table
#[derive(Debug, Clone, FromRow)]
pub struct TeamSeasonDb {
    pub id: Uuid,
    pub team_id: Uuid,
    pub season_year: i32,
    pub wins: i32,
    pub losses: i32,
    pub ties: i32,
    pub playoff_result: Option<String>,
    pub draft_position: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TeamSeasonDb {
    /// Convert from domain TeamSeason to database TeamSeasonDb
    pub fn from_domain(season: &TeamSeason) -> Self {
        Self {
            id: season.id,
            team_id: season.team_id,
            season_year: season.season_year,
            wins: season.wins,
            losses: season.losses,
            ties: season.ties,
            playoff_result: season.playoff_result.as_ref().map(|pr| pr.to_string()),
            draft_position: season.draft_position,
            created_at: season.created_at,
            updated_at: season.updated_at,
        }
    }

    /// Convert from database TeamSeasonDb to domain TeamSeason
    pub fn to_domain(&self) -> DbResult<TeamSeason> {
        let playoff_result = match &self.playoff_result {
            Some(s) => Some(string_to_playoff_result(s)?),
            None => None,
        };

        Ok(TeamSeason {
            id: self.id,
            team_id: self.team_id,
            season_year: self.season_year,
            wins: self.wins,
            losses: self.losses,
            ties: self.ties,
            playoff_result,
            draft_position: self.draft_position,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

fn string_to_playoff_result(s: &str) -> DbResult<PlayoffResult> {
    s.parse()
        .map_err(|_| DbError::MappingError(format!("Invalid playoff result: {}", s)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playoff_result_mapping() {
        assert!(matches!(
            string_to_playoff_result("MissedPlayoffs"),
            Ok(PlayoffResult::MissedPlayoffs)
        ));
        assert!(matches!(
            string_to_playoff_result("WildCard"),
            Ok(PlayoffResult::WildCard)
        ));
        assert!(matches!(
            string_to_playoff_result("Divisional"),
            Ok(PlayoffResult::Divisional)
        ));
        assert!(matches!(
            string_to_playoff_result("Conference"),
            Ok(PlayoffResult::Conference)
        ));
        assert!(matches!(
            string_to_playoff_result("SuperBowlLoss"),
            Ok(PlayoffResult::SuperBowlLoss)
        ));
        assert!(matches!(
            string_to_playoff_result("SuperBowlWin"),
            Ok(PlayoffResult::SuperBowlWin)
        ));
        assert!(string_to_playoff_result("INVALID").is_err());
    }

    #[test]
    fn test_domain_to_db_conversion() {
        let season = TeamSeason::new(
            Uuid::new_v4(),
            2025,
            10,
            7,
            0,
            Some(PlayoffResult::WildCard),
            Some(15),
        )
        .unwrap();

        let season_db = TeamSeasonDb::from_domain(&season);
        assert_eq!(season_db.wins, 10);
        assert_eq!(season_db.losses, 7);
        assert_eq!(season_db.ties, 0);
        assert_eq!(season_db.playoff_result, Some("WildCard".to_string()));
        assert_eq!(season_db.draft_position, Some(15));
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let season_db = TeamSeasonDb {
            id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            season_year: 2025,
            wins: 10,
            losses: 7,
            ties: 0,
            playoff_result: Some("WildCard".to_string()),
            draft_position: Some(15),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = season_db.to_domain();
        assert!(result.is_ok());

        let season = result.unwrap();
        assert_eq!(season.wins, 10);
        assert_eq!(season.losses, 7);
        assert_eq!(season.ties, 0);
        assert_eq!(season.playoff_result, Some(PlayoffResult::WildCard));
        assert_eq!(season.draft_position, Some(15));
    }

    #[test]
    fn test_db_to_domain_no_playoff_result() {
        let season_db = TeamSeasonDb {
            id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            season_year: 2025,
            wins: 10,
            losses: 7,
            ties: 0,
            playoff_result: None,
            draft_position: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = season_db.to_domain();
        assert!(result.is_ok());

        let season = result.unwrap();
        assert_eq!(season.playoff_result, None);
        assert_eq!(season.draft_position, None);
    }
}
