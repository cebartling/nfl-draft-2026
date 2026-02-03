use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Position {
    // Offense
    QB,
    RB,
    WR,
    TE,
    OT,
    OG,
    C,
    // Defense
    DE,
    DT,
    LB,
    CB,
    S,
    // Special Teams
    K,
    P,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub position: Position,
    pub college: Option<String>,
    pub height_inches: Option<i32>,
    pub weight_pounds: Option<i32>,
    pub draft_year: i32,
    pub draft_eligible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Player {
    pub fn new(
        first_name: String,
        last_name: String,
        position: Position,
        draft_year: i32,
    ) -> DomainResult<Self> {
        Self::validate_name(&first_name, "First name")?;
        Self::validate_name(&last_name, "Last name")?;
        Self::validate_draft_year(draft_year)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            first_name,
            last_name,
            position,
            college: None,
            height_inches: None,
            weight_pounds: None,
            draft_year,
            draft_eligible: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_college(mut self, college: String) -> DomainResult<Self> {
        if college.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "College name cannot be empty".to_string(),
            ));
        }
        if college.len() > 100 {
            return Err(DomainError::ValidationError(
                "College name cannot exceed 100 characters".to_string(),
            ));
        }
        self.college = Some(college);
        Ok(self)
    }

    pub fn with_physical_stats(
        mut self,
        height_inches: i32,
        weight_pounds: i32,
    ) -> DomainResult<Self> {
        Self::validate_height(height_inches)?;
        Self::validate_weight(weight_pounds)?;
        self.height_inches = Some(height_inches);
        self.weight_pounds = Some(weight_pounds);
        Ok(self)
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    fn validate_name(name: &str, field: &str) -> DomainResult<()> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(format!(
                "{} cannot be empty",
                field
            )));
        }
        if name.len() > 100 {
            return Err(DomainError::ValidationError(format!(
                "{} cannot exceed 100 characters",
                field
            )));
        }
        Ok(())
    }

    fn validate_draft_year(year: i32) -> DomainResult<()> {
        if year < 1936 || year > 2100 {
            return Err(DomainError::ValidationError(
                "Draft year must be between 1936 and 2100".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_height(height_inches: i32) -> DomainResult<()> {
        if height_inches < 60 || height_inches > 90 {
            return Err(DomainError::ValidationError(
                "Height must be between 60 and 90 inches".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_weight(weight_pounds: i32) -> DomainResult<()> {
        if weight_pounds < 150 || weight_pounds > 400 {
            return Err(DomainError::ValidationError(
                "Weight must be between 150 and 400 pounds".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_player() {
        let player = Player::new(
            "John".to_string(),
            "Doe".to_string(),
            Position::QB,
            2026,
        );

        assert!(player.is_ok());
        let player = player.unwrap();
        assert_eq!(player.first_name, "John");
        assert_eq!(player.last_name, "Doe");
        assert_eq!(player.position, Position::QB);
        assert_eq!(player.draft_year, 2026);
        assert_eq!(player.draft_eligible, true);
        assert_eq!(player.full_name(), "John Doe");
    }

    #[test]
    fn test_player_first_name_cannot_be_empty() {
        let result = Player::new(
            "".to_string(),
            "Doe".to_string(),
            Position::QB,
            2026,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_player_last_name_cannot_be_empty() {
        let result = Player::new(
            "John".to_string(),
            "".to_string(),
            Position::QB,
            2026,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_player_name_cannot_exceed_100_chars() {
        let long_name = "a".repeat(101);
        let result = Player::new(
            long_name,
            "Doe".to_string(),
            Position::QB,
            2026,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_draft_year() {
        let result = Player::new(
            "John".to_string(),
            "Doe".to_string(),
            Position::QB,
            1935,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_player_with_college() {
        let player = Player::new(
            "John".to_string(),
            "Doe".to_string(),
            Position::QB,
            2026,
        )
        .unwrap()
        .with_college("University of Texas".to_string());

        assert!(player.is_ok());
        let player = player.unwrap();
        assert_eq!(player.college, Some("University of Texas".to_string()));
    }

    #[test]
    fn test_player_with_physical_stats() {
        let player = Player::new(
            "John".to_string(),
            "Doe".to_string(),
            Position::QB,
            2026,
        )
        .unwrap()
        .with_physical_stats(75, 220);

        assert!(player.is_ok());
        let player = player.unwrap();
        assert_eq!(player.height_inches, Some(75));
        assert_eq!(player.weight_pounds, Some(220));
    }

    #[test]
    fn test_invalid_height() {
        let player = Player::new(
            "John".to_string(),
            "Doe".to_string(),
            Position::QB,
            2026,
        )
        .unwrap()
        .with_physical_stats(50, 220);

        assert!(player.is_err());
        assert!(matches!(player.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_invalid_weight() {
        let player = Player::new(
            "John".to_string(),
            "Doe".to_string(),
            Position::QB,
            2026,
        )
        .unwrap()
        .with_physical_stats(75, 100);

        assert!(player.is_err());
        assert!(matches!(player.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_all_positions_valid() {
        let positions = vec![
            Position::QB, Position::RB, Position::WR, Position::TE,
            Position::OT, Position::OG, Position::C,
            Position::DE, Position::DT, Position::LB, Position::CB, Position::S,
            Position::K, Position::P,
        ];

        for pos in positions {
            let result = Player::new(
                "Test".to_string(),
                "Player".to_string(),
                pos,
                2026,
            );
            assert!(result.is_ok());
        }
    }
}
