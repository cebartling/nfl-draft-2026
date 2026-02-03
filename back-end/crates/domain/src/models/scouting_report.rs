use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum FitGrade {
    A,
    B,
    C,
    D,
    F,
}

impl FitGrade {
    pub fn as_str(&self) -> &'static str {
        match self {
            FitGrade::A => "A",
            FitGrade::B => "B",
            FitGrade::C => "C",
            FitGrade::D => "D",
            FitGrade::F => "F",
        }
    }

    pub fn from_str(s: &str) -> DomainResult<Self> {
        match s {
            "A" => Ok(FitGrade::A),
            "B" => Ok(FitGrade::B),
            "C" => Ok(FitGrade::C),
            "D" => Ok(FitGrade::D),
            "F" => Ok(FitGrade::F),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid fit grade: {}. Must be A, B, C, D, or F",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ScoutingReport {
    pub id: Uuid,
    pub player_id: Uuid,
    pub team_id: Uuid,
    pub grade: f64,
    pub notes: Option<String>,
    pub fit_grade: Option<FitGrade>,
    pub injury_concern: bool,
    pub character_concern: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ScoutingReport {
    pub fn new(player_id: Uuid, team_id: Uuid, grade: f64) -> DomainResult<Self> {
        Self::validate_grade(grade)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            player_id,
            team_id,
            grade,
            notes: None,
            fit_grade: None,
            injury_concern: false,
            character_concern: false,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_notes(mut self, notes: String) -> DomainResult<Self> {
        if notes.len() > 5000 {
            return Err(DomainError::ValidationError(
                "Notes cannot exceed 5000 characters".to_string(),
            ));
        }
        self.notes = Some(notes);
        Ok(self)
    }

    pub fn with_fit_grade(mut self, grade: FitGrade) -> Self {
        self.fit_grade = Some(grade);
        self
    }

    pub fn with_injury_concern(mut self, concern: bool) -> Self {
        self.injury_concern = concern;
        self
    }

    pub fn with_character_concern(mut self, concern: bool) -> Self {
        self.character_concern = concern;
        self
    }

    pub fn update_grade(&mut self, grade: f64) -> DomainResult<()> {
        Self::validate_grade(grade)?;
        self.grade = grade;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_notes(&mut self, notes: String) -> DomainResult<()> {
        if notes.len() > 5000 {
            return Err(DomainError::ValidationError(
                "Notes cannot exceed 5000 characters".to_string(),
            ));
        }
        self.notes = Some(notes);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_fit_grade(&mut self, fit_grade: FitGrade) -> DomainResult<()> {
        self.fit_grade = Some(fit_grade);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_injury_concern(&mut self, concern: bool) -> DomainResult<()> {
        self.injury_concern = concern;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_character_concern(&mut self, concern: bool) -> DomainResult<()> {
        self.character_concern = concern;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn validate_grade(grade: f64) -> DomainResult<()> {
        if grade < 0.0 || grade > 10.0 {
            return Err(DomainError::ValidationError(
                "Grade must be between 0.0 and 10.0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scouting_report() {
        let player_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let report = ScoutingReport::new(player_id, team_id, 8.5).unwrap();

        assert_eq!(report.player_id, player_id);
        assert_eq!(report.team_id, team_id);
        assert_eq!(report.grade, 8.5);
        assert!(report.notes.is_none());
        assert!(report.fit_grade.is_none());
        assert!(!report.injury_concern);
        assert!(!report.character_concern);
    }

    #[test]
    fn test_invalid_grade() {
        let player_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        assert!(ScoutingReport::new(player_id, team_id, -0.1).is_err());
        assert!(ScoutingReport::new(player_id, team_id, 10.1).is_err());
    }

    #[test]
    fn test_builder_methods() {
        let player_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let report = ScoutingReport::new(player_id, team_id, 8.5)
            .unwrap()
            .with_notes("Excellent arm strength".to_string())
            .unwrap()
            .with_fit_grade(FitGrade::A)
            .with_injury_concern(true);

        assert_eq!(report.notes, Some("Excellent arm strength".to_string()));
        assert_eq!(report.fit_grade, Some(FitGrade::A));
        assert!(report.injury_concern);
    }

    #[test]
    fn test_notes_too_long() {
        let player_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let long_notes = "a".repeat(5001);
        let report = ScoutingReport::new(player_id, team_id, 8.5).unwrap();
        assert!(report.with_notes(long_notes).is_err());
    }

    #[test]
    fn test_fit_grade_conversion() {
        assert_eq!(FitGrade::A.as_str(), "A");
        assert_eq!(FitGrade::from_str("A").unwrap(), FitGrade::A);
        assert_eq!(FitGrade::from_str("F").unwrap(), FitGrade::F);
        assert!(FitGrade::from_str("X").is_err());
    }
}
