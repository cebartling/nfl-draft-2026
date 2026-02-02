use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::{Player, Position};

use crate::errors::{DbError, DbResult};

/// Database model for players table
#[derive(Debug, Clone, FromRow)]
pub struct PlayerDb {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub position: String,
    pub college: Option<String>,
    pub height_inches: Option<i32>,
    pub weight_pounds: Option<i32>,
    pub draft_year: i32,
    pub draft_eligible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PlayerDb {
    /// Convert from domain Player to database PlayerDb
    pub fn from_domain(player: &Player) -> Self {
        Self {
            id: player.id,
            first_name: player.first_name.clone(),
            last_name: player.last_name.clone(),
            position: position_to_string(&player.position),
            college: player.college.clone(),
            height_inches: player.height_inches,
            weight_pounds: player.weight_pounds,
            draft_year: player.draft_year,
            draft_eligible: player.draft_eligible,
            created_at: player.created_at,
            updated_at: player.updated_at,
        }
    }

    /// Convert from database PlayerDb to domain Player
    pub fn to_domain(&self) -> DbResult<Player> {
        Ok(Player {
            id: self.id,
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            position: string_to_position(&self.position)?,
            college: self.college.clone(),
            height_inches: self.height_inches,
            weight_pounds: self.weight_pounds,
            draft_year: self.draft_year,
            draft_eligible: self.draft_eligible,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

pub(crate) fn position_to_string(position: &Position) -> String {
    match position {
        Position::QB => "QB",
        Position::RB => "RB",
        Position::WR => "WR",
        Position::TE => "TE",
        Position::OT => "OT",
        Position::OG => "OG",
        Position::C => "C",
        Position::DE => "DE",
        Position::DT => "DT",
        Position::LB => "LB",
        Position::CB => "CB",
        Position::S => "S",
        Position::K => "K",
        Position::P => "P",
    }.to_string()
}

fn string_to_position(s: &str) -> DbResult<Position> {
    match s {
        "QB" => Ok(Position::QB),
        "RB" => Ok(Position::RB),
        "WR" => Ok(Position::WR),
        "TE" => Ok(Position::TE),
        "OT" => Ok(Position::OT),
        "OG" => Ok(Position::OG),
        "C" => Ok(Position::C),
        "DE" => Ok(Position::DE),
        "DT" => Ok(Position::DT),
        "LB" => Ok(Position::LB),
        "CB" => Ok(Position::CB),
        "S" => Ok(Position::S),
        "K" => Ok(Position::K),
        "P" => Ok(Position::P),
        _ => Err(DbError::MappingError(format!("Invalid position: {}", s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_mapping() {
        assert_eq!(position_to_string(&Position::QB), "QB");
        assert_eq!(position_to_string(&Position::WR), "WR");

        assert!(matches!(string_to_position("QB"), Ok(Position::QB)));
        assert!(matches!(string_to_position("WR"), Ok(Position::WR)));
        assert!(string_to_position("INVALID").is_err());
    }

    #[test]
    fn test_all_positions_map_correctly() {
        let positions = vec![
            Position::QB, Position::RB, Position::WR, Position::TE,
            Position::OT, Position::OG, Position::C,
            Position::DE, Position::DT, Position::LB, Position::CB, Position::S,
            Position::K, Position::P,
        ];

        for pos in positions {
            let s = position_to_string(&pos);
            let mapped = string_to_position(&s);
            assert!(mapped.is_ok());
            assert_eq!(mapped.unwrap(), pos);
        }
    }

    #[test]
    fn test_domain_to_db_conversion() {
        let player = Player::new(
            "John".to_string(),
            "Doe".to_string(),
            Position::QB,
            2026,
        ).unwrap();

        let player_db = PlayerDb::from_domain(&player);
        assert_eq!(player_db.first_name, "John");
        assert_eq!(player_db.last_name, "Doe");
        assert_eq!(player_db.position, "QB");
        assert_eq!(player_db.draft_year, 2026);
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let player_db = PlayerDb {
            id: Uuid::new_v4(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            position: "QB".to_string(),
            college: Some("Texas".to_string()),
            height_inches: Some(75),
            weight_pounds: Some(220),
            draft_year: 2026,
            draft_eligible: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = player_db.to_domain();
        assert!(result.is_ok());

        let player = result.unwrap();
        assert_eq!(player.first_name, "John");
        assert_eq!(player.position, Position::QB);
        assert_eq!(player.college, Some("Texas".to_string()));
    }
}
