use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::TeamNeed;

use crate::errors::DbResult;
use crate::models::player::{position_to_string, string_to_position};

/// Database model for team_needs table
#[derive(Debug, Clone, FromRow)]
pub struct TeamNeedDb {
    pub id: Uuid,
    pub team_id: Uuid,
    pub position: String,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TeamNeedDb {
    /// Convert from domain TeamNeed to database TeamNeedDb
    pub fn from_domain(need: &TeamNeed) -> Self {
        Self {
            id: need.id,
            team_id: need.team_id,
            position: position_to_string(&need.position),
            priority: need.priority,
            created_at: need.created_at,
            updated_at: need.updated_at,
        }
    }

    /// Convert from database TeamNeedDb to domain TeamNeed
    pub fn to_domain(&self) -> DbResult<TeamNeed> {
        Ok(TeamNeed {
            id: self.id,
            team_id: self.team_id,
            position: string_to_position(&self.position)?,
            priority: self.priority,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::Position;

    #[test]
    fn test_domain_to_db_conversion() {
        let team_id = Uuid::new_v4();
        let need = TeamNeed::new(team_id, Position::QB, 1).unwrap();

        let need_db = TeamNeedDb::from_domain(&need);
        assert_eq!(need_db.team_id, team_id);
        assert_eq!(need_db.position, "QB");
        assert_eq!(need_db.priority, 1);
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let need_db = TeamNeedDb {
            id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            position: "QB".to_string(),
            priority: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = need_db.to_domain();
        assert!(result.is_ok());

        let need = result.unwrap();
        assert_eq!(need.position, Position::QB);
        assert_eq!(need.priority, 1);
    }
}
