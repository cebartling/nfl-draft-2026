use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum PlayoffResult {
    MissedPlayoffs,
    WildCard,
    Divisional,
    Conference,
    SuperBowlLoss,
    SuperBowlWin,
}

impl std::fmt::Display for PlayoffResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayoffResult::MissedPlayoffs => write!(f, "MissedPlayoffs"),
            PlayoffResult::WildCard => write!(f, "WildCard"),
            PlayoffResult::Divisional => write!(f, "Divisional"),
            PlayoffResult::Conference => write!(f, "Conference"),
            PlayoffResult::SuperBowlLoss => write!(f, "SuperBowlLoss"),
            PlayoffResult::SuperBowlWin => write!(f, "SuperBowlWin"),
        }
    }
}

impl std::str::FromStr for PlayoffResult {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MissedPlayoffs" => Ok(PlayoffResult::MissedPlayoffs),
            "WildCard" => Ok(PlayoffResult::WildCard),
            "Divisional" => Ok(PlayoffResult::Divisional),
            "Conference" => Ok(PlayoffResult::Conference),
            "SuperBowlLoss" => Ok(PlayoffResult::SuperBowlLoss),
            "SuperBowlWin" => Ok(PlayoffResult::SuperBowlWin),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid playoff result: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamSeason {
    pub id: Uuid,
    pub team_id: Uuid,
    pub season_year: i32,
    pub wins: i32,
    pub losses: i32,
    pub ties: i32,
    pub playoff_result: Option<PlayoffResult>,
    pub draft_position: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TeamSeason {
    pub fn new(
        team_id: Uuid,
        season_year: i32,
        wins: i32,
        losses: i32,
        ties: i32,
        playoff_result: Option<PlayoffResult>,
        draft_position: Option<i32>,
    ) -> DomainResult<Self> {
        Self::validate_season_year(season_year)?;
        Self::validate_record(wins, losses, ties)?;
        Self::validate_draft_position(draft_position)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            team_id,
            season_year,
            wins,
            losses,
            ties,
            playoff_result,
            draft_position,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn win_percentage(&self) -> f64 {
        let total_games = self.wins + self.losses + self.ties;
        if total_games == 0 {
            return 0.0;
        }
        // NFL uses (wins + 0.5 * ties) / total_games
        (self.wins as f64 + 0.5 * self.ties as f64) / total_games as f64
    }

    fn validate_season_year(year: i32) -> DomainResult<()> {
        if !(1920..=2100).contains(&year) {
            return Err(DomainError::ValidationError(
                "Season year must be between 1920 and 2100".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_record(wins: i32, losses: i32, ties: i32) -> DomainResult<()> {
        if !(0..=17).contains(&wins) {
            return Err(DomainError::ValidationError(
                "Wins must be between 0 and 17".to_string(),
            ));
        }
        if !(0..=17).contains(&losses) {
            return Err(DomainError::ValidationError(
                "Losses must be between 0 and 17".to_string(),
            ));
        }
        if !(0..=17).contains(&ties) {
            return Err(DomainError::ValidationError(
                "Ties must be between 0 and 17".to_string(),
            ));
        }
        if wins + losses + ties > 17 {
            return Err(DomainError::ValidationError(
                "Total games (wins + losses + ties) cannot exceed 17".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_draft_position(position: Option<i32>) -> DomainResult<()> {
        if let Some(pos) = position {
            if !(1..=32).contains(&pos) {
                return Err(DomainError::ValidationError(
                    "Draft position must be between 1 and 32".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_team_season() {
        let team_id = Uuid::new_v4();
        let season = TeamSeason::new(
            team_id,
            2025,
            10,
            7,
            0,
            Some(PlayoffResult::WildCard),
            Some(15),
        );

        assert!(season.is_ok());
        let season = season.unwrap();
        assert_eq!(season.team_id, team_id);
        assert_eq!(season.season_year, 2025);
        assert_eq!(season.wins, 10);
        assert_eq!(season.losses, 7);
        assert_eq!(season.ties, 0);
        assert_eq!(season.playoff_result, Some(PlayoffResult::WildCard));
        assert_eq!(season.draft_position, Some(15));
    }

    #[test]
    fn test_win_percentage_no_ties() {
        let season = TeamSeason::new(Uuid::new_v4(), 2025, 10, 7, 0, None, None).unwrap();

        let expected = 10.0 / 17.0;
        assert!((season.win_percentage() - expected).abs() < 0.001);
    }

    #[test]
    fn test_win_percentage_with_ties() {
        let season = TeamSeason::new(Uuid::new_v4(), 2025, 8, 7, 2, None, None).unwrap();

        // (8 + 0.5 * 2) / 17 = 9 / 17
        let expected = 9.0 / 17.0;
        assert!((season.win_percentage() - expected).abs() < 0.001);
    }

    #[test]
    fn test_win_percentage_zero_games() {
        let season = TeamSeason::new(Uuid::new_v4(), 2025, 0, 0, 0, None, None).unwrap();

        assert_eq!(season.win_percentage(), 0.0);
    }

    #[test]
    fn test_invalid_season_year_too_low() {
        let result = TeamSeason::new(Uuid::new_v4(), 1919, 10, 7, 0, None, None);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::ValidationError(_)
        ));
    }

    #[test]
    fn test_invalid_season_year_too_high() {
        let result = TeamSeason::new(Uuid::new_v4(), 2101, 10, 7, 0, None, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_wins_negative() {
        let result = TeamSeason::new(Uuid::new_v4(), 2025, -1, 7, 0, None, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_wins_too_high() {
        let result = TeamSeason::new(Uuid::new_v4(), 2025, 18, 0, 0, None, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_total_games() {
        let result = TeamSeason::new(
            Uuid::new_v4(),
            2025,
            10,
            5,
            5, // 10 + 5 + 5 = 20 > 17
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_draft_position_too_low() {
        let result = TeamSeason::new(Uuid::new_v4(), 2025, 10, 7, 0, None, Some(0));

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_draft_position_too_high() {
        let result = TeamSeason::new(Uuid::new_v4(), 2025, 10, 7, 0, None, Some(33));

        assert!(result.is_err());
    }

    #[test]
    fn test_playoff_result_from_str() {
        assert_eq!(
            "MissedPlayoffs".parse::<PlayoffResult>().unwrap(),
            PlayoffResult::MissedPlayoffs
        );
        assert_eq!(
            "WildCard".parse::<PlayoffResult>().unwrap(),
            PlayoffResult::WildCard
        );
        assert_eq!(
            "Divisional".parse::<PlayoffResult>().unwrap(),
            PlayoffResult::Divisional
        );
        assert_eq!(
            "Conference".parse::<PlayoffResult>().unwrap(),
            PlayoffResult::Conference
        );
        assert_eq!(
            "SuperBowlLoss".parse::<PlayoffResult>().unwrap(),
            PlayoffResult::SuperBowlLoss
        );
        assert_eq!(
            "SuperBowlWin".parse::<PlayoffResult>().unwrap(),
            PlayoffResult::SuperBowlWin
        );
    }

    #[test]
    fn test_playoff_result_from_str_invalid() {
        let result = "Invalid".parse::<PlayoffResult>();
        assert!(result.is_err());
    }

    #[test]
    fn test_playoff_result_display() {
        assert_eq!(PlayoffResult::MissedPlayoffs.to_string(), "MissedPlayoffs");
        assert_eq!(PlayoffResult::WildCard.to_string(), "WildCard");
        assert_eq!(PlayoffResult::Divisional.to_string(), "Divisional");
        assert_eq!(PlayoffResult::Conference.to_string(), "Conference");
        assert_eq!(PlayoffResult::SuperBowlLoss.to_string(), "SuperBowlLoss");
        assert_eq!(PlayoffResult::SuperBowlWin.to_string(), "SuperBowlWin");
    }
}
