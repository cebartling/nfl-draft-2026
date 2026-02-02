use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraftStatus {
    NotStarted,
    InProgress,
    Paused,
    Completed,
}

impl std::fmt::Display for DraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DraftStatus::NotStarted => write!(f, "NotStarted"),
            DraftStatus::InProgress => write!(f, "InProgress"),
            DraftStatus::Paused => write!(f, "Paused"),
            DraftStatus::Completed => write!(f, "Completed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draft {
    pub id: Uuid,
    pub year: i32,
    pub status: DraftStatus,
    pub rounds: i32,
    pub picks_per_round: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Draft {
    pub fn new(year: i32, rounds: i32, picks_per_round: i32) -> DomainResult<Self> {
        Self::validate_year(year)?;
        Self::validate_rounds(rounds)?;
        Self::validate_picks_per_round(picks_per_round)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            year,
            status: DraftStatus::NotStarted,
            rounds,
            picks_per_round,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_status(mut self, status: DraftStatus) -> Self {
        self.status = status;
        self.updated_at = Utc::now();
        self
    }

    pub fn start(&mut self) -> DomainResult<()> {
        match self.status {
            DraftStatus::NotStarted | DraftStatus::Paused => {
                self.status = DraftStatus::InProgress;
                self.updated_at = Utc::now();
                Ok(())
            }
            DraftStatus::InProgress => Err(DomainError::InvalidState(
                "Draft is already in progress".to_string(),
            )),
            DraftStatus::Completed => Err(DomainError::InvalidState(
                "Draft is already completed".to_string(),
            )),
        }
    }

    pub fn pause(&mut self) -> DomainResult<()> {
        match self.status {
            DraftStatus::InProgress => {
                self.status = DraftStatus::Paused;
                self.updated_at = Utc::now();
                Ok(())
            }
            DraftStatus::NotStarted => Err(DomainError::InvalidState(
                "Cannot pause a draft that hasn't started".to_string(),
            )),
            DraftStatus::Paused => Err(DomainError::InvalidState(
                "Draft is already paused".to_string(),
            )),
            DraftStatus::Completed => Err(DomainError::InvalidState(
                "Cannot pause a completed draft".to_string(),
            )),
        }
    }

    pub fn complete(&mut self) -> DomainResult<()> {
        match self.status {
            DraftStatus::InProgress | DraftStatus::Paused => {
                self.status = DraftStatus::Completed;
                self.updated_at = Utc::now();
                Ok(())
            }
            DraftStatus::NotStarted => Err(DomainError::InvalidState(
                "Cannot complete a draft that hasn't started".to_string(),
            )),
            DraftStatus::Completed => Err(DomainError::InvalidState(
                "Draft is already completed".to_string(),
            )),
        }
    }

    pub fn total_picks(&self) -> i32 {
        self.rounds * self.picks_per_round
    }

    fn validate_year(year: i32) -> DomainResult<()> {
        if year < 2000 || year > 2100 {
            return Err(DomainError::ValidationError(
                "Draft year must be between 2000 and 2100".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_rounds(rounds: i32) -> DomainResult<()> {
        if rounds < 1 || rounds > 20 {
            return Err(DomainError::ValidationError(
                "Rounds must be between 1 and 20".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_picks_per_round(picks_per_round: i32) -> DomainResult<()> {
        if picks_per_round < 1 || picks_per_round > 100 {
            return Err(DomainError::ValidationError(
                "Picks per round must be between 1 and 100".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftPick {
    pub id: Uuid,
    pub draft_id: Uuid,
    pub round: i32,
    pub pick_number: i32,
    pub overall_pick: i32,
    pub team_id: Uuid,
    pub player_id: Option<Uuid>,
    pub picked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DraftPick {
    pub fn new(
        draft_id: Uuid,
        round: i32,
        pick_number: i32,
        overall_pick: i32,
        team_id: Uuid,
    ) -> DomainResult<Self> {
        Self::validate_round(round)?;
        Self::validate_pick_number(pick_number)?;
        Self::validate_overall_pick(overall_pick)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            draft_id,
            round,
            pick_number,
            overall_pick,
            team_id,
            player_id: None,
            picked_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn make_pick(&mut self, player_id: Uuid) -> DomainResult<()> {
        if self.player_id.is_some() {
            return Err(DomainError::InvalidState(
                "Pick has already been made".to_string(),
            ));
        }

        self.player_id = Some(player_id);
        self.picked_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn is_picked(&self) -> bool {
        self.player_id.is_some()
    }

    fn validate_round(round: i32) -> DomainResult<()> {
        if round < 1 {
            return Err(DomainError::ValidationError(
                "Round must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_pick_number(pick_number: i32) -> DomainResult<()> {
        if pick_number < 1 {
            return Err(DomainError::ValidationError(
                "Pick number must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_overall_pick(overall_pick: i32) -> DomainResult<()> {
        if overall_pick < 1 {
            return Err(DomainError::ValidationError(
                "Overall pick must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_draft() {
        let draft = Draft::new(2026, 7, 32).unwrap();
        assert_eq!(draft.year, 2026);
        assert_eq!(draft.rounds, 7);
        assert_eq!(draft.picks_per_round, 32);
        assert_eq!(draft.status, DraftStatus::NotStarted);
        assert_eq!(draft.total_picks(), 224);
    }

    #[test]
    fn test_draft_year_validation() {
        let result = Draft::new(1999, 7, 32);
        assert!(result.is_err());

        let result = Draft::new(2101, 7, 32);
        assert!(result.is_err());
    }

    #[test]
    fn test_draft_rounds_validation() {
        let result = Draft::new(2026, 0, 32);
        assert!(result.is_err());

        let result = Draft::new(2026, 21, 32);
        assert!(result.is_err());
    }

    #[test]
    fn test_draft_picks_per_round_validation() {
        let result = Draft::new(2026, 7, 0);
        assert!(result.is_err());

        let result = Draft::new(2026, 7, 101);
        assert!(result.is_err());
    }

    #[test]
    fn test_draft_status_transitions() {
        let mut draft = Draft::new(2026, 7, 32).unwrap();

        // Start draft
        assert!(draft.start().is_ok());
        assert_eq!(draft.status, DraftStatus::InProgress);

        // Cannot start again
        assert!(draft.start().is_err());

        // Pause draft
        assert!(draft.pause().is_ok());
        assert_eq!(draft.status, DraftStatus::Paused);

        // Cannot pause again
        assert!(draft.pause().is_err());

        // Resume draft
        assert!(draft.start().is_ok());
        assert_eq!(draft.status, DraftStatus::InProgress);

        // Complete draft
        assert!(draft.complete().is_ok());
        assert_eq!(draft.status, DraftStatus::Completed);

        // Cannot start completed draft
        assert!(draft.start().is_err());

        // Cannot pause completed draft
        assert!(draft.pause().is_err());

        // Cannot complete again
        assert!(draft.complete().is_err());
    }

    #[test]
    fn test_create_draft_pick() {
        let draft_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let pick = DraftPick::new(draft_id, 1, 1, 1, team_id).unwrap();
        assert_eq!(pick.draft_id, draft_id);
        assert_eq!(pick.round, 1);
        assert_eq!(pick.pick_number, 1);
        assert_eq!(pick.overall_pick, 1);
        assert_eq!(pick.team_id, team_id);
        assert!(pick.player_id.is_none());
        assert!(pick.picked_at.is_none());
        assert!(!pick.is_picked());
    }

    #[test]
    fn test_make_pick() {
        let draft_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();

        let mut pick = DraftPick::new(draft_id, 1, 1, 1, team_id).unwrap();

        // Make pick
        assert!(pick.make_pick(player_id).is_ok());
        assert_eq!(pick.player_id, Some(player_id));
        assert!(pick.picked_at.is_some());
        assert!(pick.is_picked());

        // Cannot make pick again
        let another_player_id = Uuid::new_v4();
        assert!(pick.make_pick(another_player_id).is_err());
    }

    #[test]
    fn test_draft_pick_validation() {
        let draft_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        // Invalid round
        let result = DraftPick::new(draft_id, 0, 1, 1, team_id);
        assert!(result.is_err());

        // Invalid pick number
        let result = DraftPick::new(draft_id, 1, 0, 1, team_id);
        assert!(result.is_err());

        // Invalid overall pick
        let result = DraftPick::new(draft_id, 1, 1, 0, team_id);
        assert!(result.is_err());
    }
}
