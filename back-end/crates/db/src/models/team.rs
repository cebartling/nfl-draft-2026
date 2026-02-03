use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::{Conference, Division, Team};

use crate::errors::{DbError, DbResult};

/// Database model for teams table
#[derive(Debug, Clone, FromRow)]
pub struct TeamDb {
    pub id: Uuid,
    pub name: String,
    pub abbreviation: String,
    pub city: String,
    pub conference: String,
    pub division: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TeamDb {
    /// Convert from domain Team to database TeamDb
    pub fn from_domain(team: &Team) -> Self {
        Self {
            id: team.id,
            name: team.name.clone(),
            abbreviation: team.abbreviation.clone(),
            city: team.city.clone(),
            conference: conference_to_string(&team.conference),
            division: division_to_string(&team.division),
            created_at: team.created_at,
            updated_at: team.updated_at,
        }
    }

    /// Convert from database TeamDb to domain Team
    pub fn to_domain(&self) -> DbResult<Team> {
        Ok(Team {
            id: self.id,
            name: self.name.clone(),
            abbreviation: self.abbreviation.clone(),
            city: self.city.clone(),
            conference: string_to_conference(&self.conference)?,
            division: string_to_division(&self.division)?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

fn conference_to_string(conference: &Conference) -> String {
    match conference {
        Conference::AFC => "AFC".to_string(),
        Conference::NFC => "NFC".to_string(),
    }
}

fn string_to_conference(s: &str) -> DbResult<Conference> {
    match s {
        "AFC" => Ok(Conference::AFC),
        "NFC" => Ok(Conference::NFC),
        _ => Err(DbError::MappingError(format!("Invalid conference: {}", s))),
    }
}

fn division_to_string(division: &Division) -> String {
    match division {
        Division::AFCEast => "AFC East".to_string(),
        Division::AFCNorth => "AFC North".to_string(),
        Division::AFCSouth => "AFC South".to_string(),
        Division::AFCWest => "AFC West".to_string(),
        Division::NFCEast => "NFC East".to_string(),
        Division::NFCNorth => "NFC North".to_string(),
        Division::NFCSouth => "NFC South".to_string(),
        Division::NFCWest => "NFC West".to_string(),
    }
}

fn string_to_division(s: &str) -> DbResult<Division> {
    match s {
        "AFC East" => Ok(Division::AFCEast),
        "AFC North" => Ok(Division::AFCNorth),
        "AFC South" => Ok(Division::AFCSouth),
        "AFC West" => Ok(Division::AFCWest),
        "NFC East" => Ok(Division::NFCEast),
        "NFC North" => Ok(Division::NFCNorth),
        "NFC South" => Ok(Division::NFCSouth),
        "NFC West" => Ok(Division::NFCWest),
        _ => Err(DbError::MappingError(format!("Invalid division: {}", s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::{Conference, Division};

    #[test]
    fn test_conference_mapping() {
        assert_eq!(conference_to_string(&Conference::AFC), "AFC");
        assert_eq!(conference_to_string(&Conference::NFC), "NFC");

        assert!(matches!(string_to_conference("AFC"), Ok(Conference::AFC)));
        assert!(matches!(string_to_conference("NFC"), Ok(Conference::NFC)));
        assert!(string_to_conference("INVALID").is_err());
    }

    #[test]
    fn test_division_mapping() {
        assert_eq!(division_to_string(&Division::AFCEast), "AFC East");
        assert_eq!(division_to_string(&Division::NFCWest), "NFC West");

        assert!(matches!(
            string_to_division("AFC East"),
            Ok(Division::AFCEast)
        ));
        assert!(matches!(
            string_to_division("NFC West"),
            Ok(Division::NFCWest)
        ));
        assert!(string_to_division("INVALID").is_err());
    }

    #[test]
    fn test_domain_to_db_conversion() {
        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();

        let team_db = TeamDb::from_domain(&team);
        assert_eq!(team_db.name, "Dallas Cowboys");
        assert_eq!(team_db.conference, "NFC");
        assert_eq!(team_db.division, "NFC East");
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let team_db = TeamDb {
            id: Uuid::new_v4(),
            name: "Dallas Cowboys".to_string(),
            abbreviation: "DAL".to_string(),
            city: "Dallas".to_string(),
            conference: "NFC".to_string(),
            division: "NFC East".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = team_db.to_domain();
        assert!(result.is_ok());

        let team = result.unwrap();
        assert_eq!(team.name, "Dallas Cowboys");
        assert_eq!(team.conference, Conference::NFC);
        assert_eq!(team.division, Division::NFCEast);
    }
}
