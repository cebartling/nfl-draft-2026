use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::{FitGrade, ScoutingReport};

use crate::errors::{DbError, DbResult};

/// Database model for scouting_reports table
#[derive(Debug, Clone, FromRow)]
pub struct ScoutingReportDb {
    pub id: Uuid,
    pub player_id: Uuid,
    pub team_id: Uuid,
    pub grade: f64,
    pub notes: Option<String>,
    pub fit_grade: Option<String>,
    pub injury_concern: bool,
    pub character_concern: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ScoutingReportDb {
    /// Convert from domain ScoutingReport to database ScoutingReportDb
    pub fn from_domain(report: &ScoutingReport) -> Self {
        Self {
            id: report.id,
            player_id: report.player_id,
            team_id: report.team_id,
            grade: report.grade,
            notes: report.notes.clone(),
            fit_grade: report.fit_grade.map(|g| g.as_str().to_string()),
            injury_concern: report.injury_concern,
            character_concern: report.character_concern,
            created_at: report.created_at,
            updated_at: report.updated_at,
        }
    }

    /// Convert from database ScoutingReportDb to domain ScoutingReport
    pub fn to_domain(&self) -> DbResult<ScoutingReport> {
        let fit_grade = match &self.fit_grade {
            Some(s) => Some(
                FitGrade::parse_grade(s)
                    .map_err(|_| DbError::MappingError(format!("Invalid fit grade: {}", s)))?,
            ),
            None => None,
        };

        Ok(ScoutingReport {
            id: self.id,
            player_id: self.player_id,
            team_id: self.team_id,
            grade: self.grade,
            notes: self.notes.clone(),
            fit_grade,
            injury_concern: self.injury_concern,
            character_concern: self.character_concern,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_to_db_conversion() {
        let player_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let report = ScoutingReport::new(player_id, team_id, 8.5)
            .unwrap()
            .with_fit_grade(FitGrade::A)
            .with_injury_concern(true);

        let report_db = ScoutingReportDb::from_domain(&report);
        assert_eq!(report_db.player_id, player_id);
        assert_eq!(report_db.team_id, team_id);
        assert_eq!(report_db.grade, 8.5);
        assert_eq!(report_db.fit_grade, Some("A".to_string()));
        assert!(report_db.injury_concern);
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let report_db = ScoutingReportDb {
            id: Uuid::new_v4(),
            player_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            grade: 8.5,
            notes: Some("Excellent prospect".to_string()),
            fit_grade: Some("A".to_string()),
            injury_concern: false,
            character_concern: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = report_db.to_domain();
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.grade, 8.5);
        assert_eq!(report.fit_grade, Some(FitGrade::A));
        assert_eq!(report.notes, Some("Excellent prospect".to_string()));
    }

    #[test]
    fn test_invalid_fit_grade() {
        let report_db = ScoutingReportDb {
            id: Uuid::new_v4(),
            player_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            grade: 8.5,
            notes: None,
            fit_grade: Some("X".to_string()),
            injury_concern: false,
            character_concern: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = report_db.to_domain();
        assert!(result.is_err());
    }
}
