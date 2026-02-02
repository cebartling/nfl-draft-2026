use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum Conference {
    AFC,
    NFC,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Division {
    #[serde(rename = "AFC East")]
    AFCEast,
    #[serde(rename = "AFC North")]
    AFCNorth,
    #[serde(rename = "AFC South")]
    AFCSouth,
    #[serde(rename = "AFC West")]
    AFCWest,
    #[serde(rename = "NFC East")]
    NFCEast,
    #[serde(rename = "NFC North")]
    NFCNorth,
    #[serde(rename = "NFC South")]
    NFCSouth,
    #[serde(rename = "NFC West")]
    NFCWest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub abbreviation: String,
    pub city: String,
    pub conference: Conference,
    pub division: Division,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Team {
    pub fn new(
        name: String,
        abbreviation: String,
        city: String,
        conference: Conference,
        division: Division,
    ) -> DomainResult<Self> {
        Self::validate_name(&name)?;
        Self::validate_abbreviation(&abbreviation)?;
        Self::validate_city(&city)?;
        Self::validate_conference_division(&conference, &division)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            abbreviation,
            city,
            conference,
            division,
            created_at: now,
            updated_at: now,
        })
    }

    fn validate_name(name: &str) -> DomainResult<()> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Team name cannot be empty".to_string(),
            ));
        }
        if name.len() > 100 {
            return Err(DomainError::ValidationError(
                "Team name cannot exceed 100 characters".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_abbreviation(abbreviation: &str) -> DomainResult<()> {
        if abbreviation.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Team abbreviation cannot be empty".to_string(),
            ));
        }
        if abbreviation.len() > 5 {
            return Err(DomainError::ValidationError(
                "Team abbreviation cannot exceed 5 characters".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_city(city: &str) -> DomainResult<()> {
        if city.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Team city cannot be empty".to_string(),
            ));
        }
        if city.len() > 100 {
            return Err(DomainError::ValidationError(
                "Team city cannot exceed 100 characters".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_conference_division(
        conference: &Conference,
        division: &Division,
    ) -> DomainResult<()> {
        let valid = match (conference, division) {
            (Conference::AFC, Division::AFCEast)
            | (Conference::AFC, Division::AFCNorth)
            | (Conference::AFC, Division::AFCSouth)
            | (Conference::AFC, Division::AFCWest)
            | (Conference::NFC, Division::NFCEast)
            | (Conference::NFC, Division::NFCNorth)
            | (Conference::NFC, Division::NFCSouth)
            | (Conference::NFC, Division::NFCWest) => true,
            _ => false,
        };

        if !valid {
            return Err(DomainError::ValidationError(
                "Division must match conference".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_team() {
        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        );

        assert!(team.is_ok());
        let team = team.unwrap();
        assert_eq!(team.name, "Dallas Cowboys");
        assert_eq!(team.abbreviation, "DAL");
        assert_eq!(team.city, "Dallas");
        assert_eq!(team.conference, Conference::NFC);
        assert_eq!(team.division, Division::NFCEast);
    }

    #[test]
    fn test_team_name_cannot_be_empty() {
        let result = Team::new(
            "".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_team_name_cannot_exceed_100_chars() {
        let long_name = "a".repeat(101);
        let result = Team::new(
            long_name,
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_team_abbreviation_cannot_be_empty() {
        let result = Team::new(
            "Dallas Cowboys".to_string(),
            "".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_team_abbreviation_cannot_exceed_5_chars() {
        let result = Team::new(
            "Dallas Cowboys".to_string(),
            "DALLAS".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_team_city_cannot_be_empty() {
        let result = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "".to_string(),
            Conference::NFC,
            Division::NFCEast,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_division_must_match_conference() {
        // NFC conference with AFC division should fail
        let result = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::AFCEast,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[test]
    fn test_afc_teams_valid() {
        let divisions = vec![
            Division::AFCEast,
            Division::AFCNorth,
            Division::AFCSouth,
            Division::AFCWest,
        ];

        for division in divisions {
            let result = Team::new(
                "Test Team".to_string(),
                "TST".to_string(),
                "Test City".to_string(),
                Conference::AFC,
                division,
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_nfc_teams_valid() {
        let divisions = vec![
            Division::NFCEast,
            Division::NFCNorth,
            Division::NFCSouth,
            Division::NFCWest,
        ];

        for division in divisions {
            let result = Team::new(
                "Test Team".to_string(),
                "TST".to_string(),
                "Test City".to_string(),
                Conference::NFC,
                division,
            );
            assert!(result.is_ok());
        }
    }
}
