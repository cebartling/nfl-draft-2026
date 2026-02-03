use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};
use super::ChartType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum SessionStatus {
    NotStarted,
    InProgress,
    Paused,
    Completed,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::NotStarted => write!(f, "NotStarted"),
            SessionStatus::InProgress => write!(f, "InProgress"),
            SessionStatus::Paused => write!(f, "Paused"),
            SessionStatus::Completed => write!(f, "Completed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftSession {
    pub id: Uuid,
    pub draft_id: Uuid,
    pub status: SessionStatus,
    pub current_pick_number: i32,
    pub time_per_pick_seconds: i32,
    pub auto_pick_enabled: bool,
    pub chart_type: ChartType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl DraftSession {
    pub fn new(
        draft_id: Uuid,
        time_per_pick_seconds: i32,
        auto_pick_enabled: bool,
        chart_type: ChartType,
    ) -> DomainResult<Self> {
        Self::validate_time_per_pick(time_per_pick_seconds)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            draft_id,
            status: SessionStatus::NotStarted,
            current_pick_number: 1,
            time_per_pick_seconds,
            auto_pick_enabled,
            chart_type,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
        })
    }

    /// Convenience constructor with default chart type
    pub fn new_with_default_chart(
        draft_id: Uuid,
        time_per_pick_seconds: i32,
        auto_pick_enabled: bool,
    ) -> DomainResult<Self> {
        Self::new(
            draft_id,
            time_per_pick_seconds,
            auto_pick_enabled,
            ChartType::JimmyJohnson,
        )
    }

    /// Set the session status directly, bypassing state validation.
    ///
    /// This method is intended for internal use by the repository layer
    /// when loading sessions from the database.
    pub fn with_status(mut self, status: SessionStatus) -> Self {
        self.status = status;
        self.updated_at = Utc::now();
        self
    }

    pub fn start(&mut self) -> DomainResult<()> {
        match self.status {
            SessionStatus::NotStarted | SessionStatus::Paused => {
                self.status = SessionStatus::InProgress;
                self.updated_at = Utc::now();
                if self.started_at.is_none() {
                    self.started_at = Some(Utc::now());
                }
                Ok(())
            }
            SessionStatus::InProgress => Err(DomainError::InvalidState(
                "Session is already in progress".to_string(),
            )),
            SessionStatus::Completed => Err(DomainError::InvalidState(
                "Session is already completed".to_string(),
            )),
        }
    }

    pub fn pause(&mut self) -> DomainResult<()> {
        match self.status {
            SessionStatus::InProgress => {
                self.status = SessionStatus::Paused;
                self.updated_at = Utc::now();
                Ok(())
            }
            SessionStatus::NotStarted => Err(DomainError::InvalidState(
                "Cannot pause a session that hasn't started".to_string(),
            )),
            SessionStatus::Paused => Err(DomainError::InvalidState(
                "Session is already paused".to_string(),
            )),
            SessionStatus::Completed => Err(DomainError::InvalidState(
                "Cannot pause a completed session".to_string(),
            )),
        }
    }

    pub fn complete(&mut self) -> DomainResult<()> {
        match self.status {
            SessionStatus::InProgress | SessionStatus::Paused => {
                self.status = SessionStatus::Completed;
                self.updated_at = Utc::now();
                self.completed_at = Some(Utc::now());
                Ok(())
            }
            SessionStatus::NotStarted => Err(DomainError::InvalidState(
                "Cannot complete a session that hasn't started".to_string(),
            )),
            SessionStatus::Completed => Err(DomainError::InvalidState(
                "Session is already completed".to_string(),
            )),
        }
    }

    pub fn advance_pick(&mut self) -> DomainResult<()> {
        if self.status != SessionStatus::InProgress {
            return Err(DomainError::InvalidState(
                "Can only advance pick during an active session".to_string(),
            ));
        }
        self.current_pick_number += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::InProgress
    }

    fn validate_time_per_pick(time_per_pick_seconds: i32) -> DomainResult<()> {
        if !(10..=3600).contains(&time_per_pick_seconds) {
            return Err(DomainError::ValidationError(
                "Time per pick must be between 10 and 3600 seconds".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let draft_id = Uuid::new_v4();
        let session = DraftSession::new_with_default_chart(draft_id, 300, false).unwrap();

        assert_eq!(session.draft_id, draft_id);
        assert_eq!(session.status, SessionStatus::NotStarted);
        assert_eq!(session.current_pick_number, 1);
        assert_eq!(session.time_per_pick_seconds, 300);
        assert!(!session.auto_pick_enabled);
        assert!(session.started_at.is_none());
        assert!(session.completed_at.is_none());
    }

    #[test]
    fn test_time_per_pick_validation() {
        let draft_id = Uuid::new_v4();

        // Too short
        let result = DraftSession::new_with_default_chart(draft_id, 5, false);
        assert!(result.is_err());

        // Too long
        let result = DraftSession::new_with_default_chart(draft_id, 3601, false);
        assert!(result.is_err());

        // Valid range
        let result = DraftSession::new_with_default_chart(draft_id, 10, false);
        assert!(result.is_ok());

        let result = DraftSession::new_with_default_chart(draft_id, 3600, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_lifecycle() {
        let draft_id = Uuid::new_v4();
        let mut session = DraftSession::new_with_default_chart(draft_id, 300, false).unwrap();

        // Start session
        assert!(session.start().is_ok());
        assert_eq!(session.status, SessionStatus::InProgress);
        assert!(session.started_at.is_some());
        assert!(session.is_active());

        // Cannot start again
        assert!(session.start().is_err());

        // Pause session
        assert!(session.pause().is_ok());
        assert_eq!(session.status, SessionStatus::Paused);
        assert!(!session.is_active());

        // Cannot pause again
        assert!(session.pause().is_err());

        // Resume session
        assert!(session.start().is_ok());
        assert_eq!(session.status, SessionStatus::InProgress);
        assert!(session.is_active());

        // Complete session
        assert!(session.complete().is_ok());
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.completed_at.is_some());
        assert!(!session.is_active());

        // Cannot modify completed session
        assert!(session.start().is_err());
        assert!(session.pause().is_err());
        assert!(session.complete().is_err());
    }

    #[test]
    fn test_advance_pick() {
        let draft_id = Uuid::new_v4();
        let mut session = DraftSession::new_with_default_chart(draft_id, 300, false).unwrap();

        // Cannot advance before starting
        assert!(session.advance_pick().is_err());

        // Start session
        session.start().unwrap();

        // Advance pick
        assert_eq!(session.current_pick_number, 1);
        assert!(session.advance_pick().is_ok());
        assert_eq!(session.current_pick_number, 2);
        assert!(session.advance_pick().is_ok());
        assert_eq!(session.current_pick_number, 3);

        // Pause and try to advance
        session.pause().unwrap();
        assert!(session.advance_pick().is_err());
    }
}
