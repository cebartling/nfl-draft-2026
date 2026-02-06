use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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
    pub picks_per_round: Option<i32>,
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
            picks_per_round: Some(picks_per_round),
            created_at: now,
            updated_at: now,
        })
    }

    /// Create a realistic draft with variable-length rounds (picks loaded from data)
    pub fn new_realistic(year: i32, rounds: i32) -> DomainResult<Self> {
        Self::validate_year(year)?;
        Self::validate_rounds(rounds)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            year,
            status: DraftStatus::NotStarted,
            rounds,
            picks_per_round: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Returns true if this is a realistic draft (variable round sizes, loaded from data)
    pub fn is_realistic(&self) -> bool {
        self.picks_per_round.is_none()
    }

    /// Set the draft status directly, bypassing state validation.
    ///
    /// **WARNING**: This method is intended for internal use and testing only.
    /// It bypasses the state validation enforced by `start()`, `pause()`, and `complete()` methods.
    ///
    /// For production code, use the proper state transition methods:
    /// - `start()` - transition to InProgress
    /// - `pause()` - transition to Paused
    /// - `complete()` - transition to Completed
    ///
    /// This method is primarily used by the repository layer when loading drafts from the database.
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

    /// Returns total picks for custom drafts, None for realistic drafts
    pub fn total_picks(&self) -> Option<i32> {
        self.picks_per_round.map(|ppr| self.rounds * ppr)
    }

    fn validate_year(year: i32) -> DomainResult<()> {
        if !(2000..=2100).contains(&year) {
            return Err(DomainError::ValidationError(
                "Draft year must be between 2000 and 2100".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_rounds(rounds: i32) -> DomainResult<()> {
        if !(1..=20).contains(&rounds) {
            return Err(DomainError::ValidationError(
                "Rounds must be between 1 and 20".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_picks_per_round(picks_per_round: i32) -> DomainResult<()> {
        if !(1..=100).contains(&picks_per_round) {
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
    pub original_team_id: Option<Uuid>,
    pub is_compensatory: bool,
    pub notes: Option<String>,
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
            original_team_id: None,
            is_compensatory: false,
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create a realistic draft pick with trade/compensatory metadata
    #[allow(clippy::too_many_arguments)]
    pub fn new_realistic(
        draft_id: Uuid,
        round: i32,
        pick_number: i32,
        overall_pick: i32,
        team_id: Uuid,
        original_team_id: Option<Uuid>,
        is_compensatory: bool,
        notes: Option<String>,
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
            original_team_id,
            is_compensatory,
            notes,
            created_at: now,
            updated_at: now,
        })
    }

    /// Returns true if this pick was traded (team differs from original team)
    pub fn is_traded(&self) -> bool {
        self.original_team_id
            .is_some_and(|orig| orig != self.team_id)
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
        assert_eq!(draft.picks_per_round, Some(32));
        assert_eq!(draft.status, DraftStatus::NotStarted);
        assert_eq!(draft.total_picks(), Some(224));
        assert!(!draft.is_realistic());
    }

    #[test]
    fn test_create_realistic_draft() {
        let draft = Draft::new_realistic(2026, 7).unwrap();
        assert_eq!(draft.year, 2026);
        assert_eq!(draft.rounds, 7);
        assert_eq!(draft.picks_per_round, None);
        assert_eq!(draft.status, DraftStatus::NotStarted);
        assert_eq!(draft.total_picks(), None);
        assert!(draft.is_realistic());
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

    #[test]
    fn test_draft_pick_new_has_no_trade_metadata() {
        let draft_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let pick = DraftPick::new(draft_id, 1, 1, 1, team_id).unwrap();

        assert_eq!(pick.original_team_id, None);
        assert!(!pick.is_compensatory);
        assert_eq!(pick.notes, None);
        assert!(!pick.is_traded());
    }

    #[test]
    fn test_realistic_draft_pick() {
        let draft_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let original_team_id = Uuid::new_v4();

        let pick = DraftPick::new_realistic(
            draft_id,
            1,
            24,
            24,
            team_id,
            Some(original_team_id),
            false,
            Some("From Green Bay".to_string()),
        )
        .unwrap();

        assert_eq!(pick.original_team_id, Some(original_team_id));
        assert!(!pick.is_compensatory);
        assert_eq!(pick.notes, Some("From Green Bay".to_string()));
        assert!(pick.is_traded());
    }

    #[test]
    fn test_compensatory_pick() {
        let draft_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let pick = DraftPick::new_realistic(
            draft_id,
            3,
            33,
            97,
            team_id,
            None,
            true,
            Some("Compensatory pick".to_string()),
        )
        .unwrap();

        assert!(pick.is_compensatory);
        assert!(!pick.is_traded());
    }

    #[test]
    fn test_not_traded_when_same_team() {
        let draft_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let pick = DraftPick::new_realistic(
            draft_id,
            1,
            1,
            1,
            team_id,
            Some(team_id), // same team owns it
            false,
            None,
        )
        .unwrap();

        assert!(!pick.is_traded());
    }
}
