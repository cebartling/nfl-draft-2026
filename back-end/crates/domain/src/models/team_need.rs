use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};
use crate::models::Position;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct TeamNeed {
    pub id: Uuid,
    pub team_id: Uuid,
    pub position: Position,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TeamNeed {
    pub fn new(team_id: Uuid, position: Position, priority: i32) -> DomainResult<Self> {
        Self::validate_priority(priority)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            team_id,
            position,
            priority,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_priority(&mut self, priority: i32) -> DomainResult<()> {
        Self::validate_priority(priority)?;
        self.priority = priority;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn validate_priority(priority: i32) -> DomainResult<()> {
        if priority < 1 || priority > 10 {
            return Err(DomainError::ValidationError(
                "Priority must be between 1 and 10".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_team_need() {
        let team_id = Uuid::new_v4();
        let need = TeamNeed::new(team_id, Position::QB, 1).unwrap();

        assert_eq!(need.team_id, team_id);
        assert_eq!(need.position, Position::QB);
        assert_eq!(need.priority, 1);
    }

    #[test]
    fn test_invalid_priority() {
        let team_id = Uuid::new_v4();
        assert!(TeamNeed::new(team_id, Position::QB, 0).is_err());
        assert!(TeamNeed::new(team_id, Position::QB, 11).is_err());
    }

    #[test]
    fn test_update_priority() {
        let team_id = Uuid::new_v4();
        let mut need = TeamNeed::new(team_id, Position::QB, 1).unwrap();

        need.update_priority(5).unwrap();
        assert_eq!(need.priority, 5);

        assert!(need.update_priority(11).is_err());
    }
}
