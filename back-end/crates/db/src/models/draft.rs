use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::{Draft, DraftPick, DraftStatus};

use crate::errors::{DbError, DbResult};

/// Database model for drafts table
#[derive(Debug, Clone, FromRow)]
pub struct DraftDb {
    pub id: Uuid,
    pub name: String,
    pub year: i32,
    pub status: String,
    pub rounds: i32,
    pub picks_per_round: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DraftDb {
    /// Convert from domain Draft to database DraftDb
    pub fn from_domain(draft: &Draft) -> Self {
        Self {
            id: draft.id,
            name: draft.name.clone(),
            year: draft.year,
            status: status_to_string(&draft.status),
            rounds: draft.rounds,
            picks_per_round: draft.picks_per_round,
            created_at: draft.created_at,
            updated_at: draft.updated_at,
        }
    }

    /// Convert from database DraftDb to domain Draft
    pub fn to_domain(&self) -> DbResult<Draft> {
        Ok(Draft {
            id: self.id,
            name: self.name.clone(),
            year: self.year,
            status: string_to_status(&self.status)?,
            rounds: self.rounds,
            picks_per_round: self.picks_per_round,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// Database model for draft_picks table
#[derive(Debug, Clone, FromRow)]
pub struct DraftPickDb {
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

impl DraftPickDb {
    /// Convert from domain DraftPick to database DraftPickDb
    pub fn from_domain(pick: &DraftPick) -> Self {
        Self {
            id: pick.id,
            draft_id: pick.draft_id,
            round: pick.round,
            pick_number: pick.pick_number,
            overall_pick: pick.overall_pick,
            team_id: pick.team_id,
            player_id: pick.player_id,
            picked_at: pick.picked_at,
            original_team_id: pick.original_team_id,
            is_compensatory: pick.is_compensatory,
            notes: pick.notes.clone(),
            created_at: pick.created_at,
            updated_at: pick.updated_at,
        }
    }

    /// Convert from database DraftPickDb to domain DraftPick
    pub fn to_domain(&self) -> DbResult<DraftPick> {
        Ok(DraftPick {
            id: self.id,
            draft_id: self.draft_id,
            round: self.round,
            pick_number: self.pick_number,
            overall_pick: self.overall_pick,
            team_id: self.team_id,
            player_id: self.player_id,
            picked_at: self.picked_at,
            original_team_id: self.original_team_id,
            is_compensatory: self.is_compensatory,
            notes: self.notes.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

fn status_to_string(status: &DraftStatus) -> String {
    status.to_string()
}

fn string_to_status(s: &str) -> DbResult<DraftStatus> {
    match s {
        "NotStarted" => Ok(DraftStatus::NotStarted),
        "InProgress" => Ok(DraftStatus::InProgress),
        "Paused" => Ok(DraftStatus::Paused),
        "Completed" => Ok(DraftStatus::Completed),
        _ => Err(DbError::MappingError(format!(
            "Invalid draft status: {}",
            s
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_mapping() {
        assert_eq!(status_to_string(&DraftStatus::NotStarted), "NotStarted");
        assert_eq!(status_to_string(&DraftStatus::InProgress), "InProgress");
        assert_eq!(status_to_string(&DraftStatus::Paused), "Paused");
        assert_eq!(status_to_string(&DraftStatus::Completed), "Completed");

        assert!(matches!(
            string_to_status("NotStarted"),
            Ok(DraftStatus::NotStarted)
        ));
        assert!(matches!(
            string_to_status("InProgress"),
            Ok(DraftStatus::InProgress)
        ));
        assert!(matches!(
            string_to_status("Paused"),
            Ok(DraftStatus::Paused)
        ));
        assert!(matches!(
            string_to_status("Completed"),
            Ok(DraftStatus::Completed)
        ));
        assert!(string_to_status("INVALID").is_err());
    }

    #[test]
    fn test_draft_domain_to_db_conversion() {
        let draft = Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap();
        let draft_db = DraftDb::from_domain(&draft);

        assert_eq!(draft_db.name, "Test Draft");
        assert_eq!(draft_db.year, 2026);
        assert_eq!(draft_db.status, "NotStarted");
        assert_eq!(draft_db.rounds, 7);
        assert_eq!(draft_db.picks_per_round, Some(32));
    }

    #[test]
    fn test_draft_db_to_domain_conversion() {
        let draft_db = DraftDb {
            id: Uuid::new_v4(),
            name: "Test Draft".to_string(),
            year: 2026,
            status: "NotStarted".to_string(),
            rounds: 7,
            picks_per_round: Some(32),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = draft_db.to_domain();
        assert!(result.is_ok());

        let draft = result.unwrap();
        assert_eq!(draft.name, "Test Draft");
        assert_eq!(draft.year, 2026);
        assert_eq!(draft.status, DraftStatus::NotStarted);
        assert_eq!(draft.rounds, 7);
        assert_eq!(draft.picks_per_round, Some(32));
    }

    #[test]
    fn test_realistic_draft_db_conversion() {
        let draft = Draft::new_realistic("Realistic Draft".to_string(), 2026, 7).unwrap();
        let draft_db = DraftDb::from_domain(&draft);

        assert_eq!(draft_db.picks_per_round, None);

        let domain = draft_db.to_domain().unwrap();
        assert_eq!(domain.picks_per_round, None);
        assert!(domain.is_realistic());
    }

    #[test]
    fn test_draft_pick_conversions() {
        let draft_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let pick = DraftPick::new(draft_id, 1, 1, 1, team_id).unwrap();

        // Domain to DB
        let pick_db = DraftPickDb::from_domain(&pick);
        assert_eq!(pick_db.draft_id, draft_id);
        assert_eq!(pick_db.round, 1);
        assert_eq!(pick_db.pick_number, 1);
        assert_eq!(pick_db.overall_pick, 1);
        assert_eq!(pick_db.team_id, team_id);
        assert!(pick_db.player_id.is_none());
        assert!(pick_db.picked_at.is_none());

        // DB to Domain
        let result = pick_db.to_domain();
        assert!(result.is_ok());

        let pick_result = result.unwrap();
        assert_eq!(pick_result.draft_id, draft_id);
        assert_eq!(pick_result.round, 1);
        assert!(pick_result.player_id.is_none());
    }
}
