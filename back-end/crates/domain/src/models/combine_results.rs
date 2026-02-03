use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CombineResults {
    pub id: Uuid,
    pub player_id: Uuid,
    pub year: i32,
    pub forty_yard_dash: Option<f64>,
    pub bench_press: Option<i32>,
    pub vertical_jump: Option<f64>,
    pub broad_jump: Option<i32>,
    pub three_cone_drill: Option<f64>,
    pub twenty_yard_shuttle: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CombineResults {
    pub fn new(player_id: Uuid, year: i32) -> DomainResult<Self> {
        Self::validate_year(year)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            player_id,
            year,
            forty_yard_dash: None,
            bench_press: None,
            vertical_jump: None,
            broad_jump: None,
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_forty_yard_dash(mut self, time: f64) -> DomainResult<Self> {
        Self::validate_forty_dash(time)?;
        self.forty_yard_dash = Some(time);
        Ok(self)
    }

    pub fn with_bench_press(mut self, reps: i32) -> DomainResult<Self> {
        Self::validate_bench_press(reps)?;
        self.bench_press = Some(reps);
        Ok(self)
    }

    pub fn with_vertical_jump(mut self, inches: f64) -> DomainResult<Self> {
        Self::validate_vertical_jump(inches)?;
        self.vertical_jump = Some(inches);
        Ok(self)
    }

    pub fn with_broad_jump(mut self, inches: i32) -> DomainResult<Self> {
        Self::validate_broad_jump(inches)?;
        self.broad_jump = Some(inches);
        Ok(self)
    }

    pub fn with_three_cone_drill(mut self, time: f64) -> DomainResult<Self> {
        Self::validate_three_cone_drill(time)?;
        self.three_cone_drill = Some(time);
        Ok(self)
    }

    pub fn with_twenty_yard_shuttle(mut self, time: f64) -> DomainResult<Self> {
        Self::validate_twenty_yard_shuttle(time)?;
        self.twenty_yard_shuttle = Some(time);
        Ok(self)
    }

    // Update methods for modifying existing results
    pub fn update_forty_yard_dash(&mut self, time: Option<f64>) -> DomainResult<()> {
        if let Some(t) = time {
            Self::validate_forty_dash(t)?;
        }
        self.forty_yard_dash = time;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_bench_press(&mut self, reps: Option<i32>) -> DomainResult<()> {
        if let Some(r) = reps {
            Self::validate_bench_press(r)?;
        }
        self.bench_press = reps;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_vertical_jump(&mut self, inches: Option<f64>) -> DomainResult<()> {
        if let Some(i) = inches {
            Self::validate_vertical_jump(i)?;
        }
        self.vertical_jump = inches;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_broad_jump(&mut self, inches: Option<i32>) -> DomainResult<()> {
        if let Some(i) = inches {
            Self::validate_broad_jump(i)?;
        }
        self.broad_jump = inches;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_three_cone_drill(&mut self, time: Option<f64>) -> DomainResult<()> {
        if let Some(t) = time {
            Self::validate_three_cone_drill(t)?;
        }
        self.three_cone_drill = time;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_twenty_yard_shuttle(&mut self, time: Option<f64>) -> DomainResult<()> {
        if let Some(t) = time {
            Self::validate_twenty_yard_shuttle(t)?;
        }
        self.twenty_yard_shuttle = time;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn validate_year(year: i32) -> DomainResult<()> {
        if year < 2000 || year > 2100 {
            return Err(DomainError::ValidationError(
                "Combine year must be between 2000 and 2100".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_forty_dash(time: f64) -> DomainResult<()> {
        if time < 4.0 || time > 6.0 {
            return Err(DomainError::ValidationError(
                "40-yard dash time must be between 4.0 and 6.0 seconds".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_bench_press(reps: i32) -> DomainResult<()> {
        if reps < 0 || reps > 50 {
            return Err(DomainError::ValidationError(
                "Bench press reps must be between 0 and 50".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_vertical_jump(inches: f64) -> DomainResult<()> {
        if inches < 20.0 || inches > 50.0 {
            return Err(DomainError::ValidationError(
                "Vertical jump must be between 20 and 50 inches".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_broad_jump(inches: i32) -> DomainResult<()> {
        if inches < 80 || inches > 150 {
            return Err(DomainError::ValidationError(
                "Broad jump must be between 80 and 150 inches".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_three_cone_drill(time: f64) -> DomainResult<()> {
        if time < 6.0 || time > 9.0 {
            return Err(DomainError::ValidationError(
                "Three cone drill time must be between 6.0 and 9.0 seconds".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_twenty_yard_shuttle(time: f64) -> DomainResult<()> {
        if time < 3.5 || time > 6.0 {
            return Err(DomainError::ValidationError(
                "20-yard shuttle time must be between 3.5 and 6.0 seconds".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_combine_results() {
        let player_id = Uuid::new_v4();
        let results = CombineResults::new(player_id, 2026).unwrap();

        assert_eq!(results.player_id, player_id);
        assert_eq!(results.year, 2026);
        assert!(results.forty_yard_dash.is_none());
    }

    #[test]
    fn test_invalid_year() {
        let player_id = Uuid::new_v4();
        assert!(CombineResults::new(player_id, 1999).is_err());
        assert!(CombineResults::new(player_id, 2101).is_err());
    }

    #[test]
    fn test_builder_methods() {
        let player_id = Uuid::new_v4();
        let results = CombineResults::new(player_id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.52)
            .unwrap()
            .with_bench_press(20)
            .unwrap()
            .with_vertical_jump(35.5)
            .unwrap();

        assert_eq!(results.forty_yard_dash, Some(4.52));
        assert_eq!(results.bench_press, Some(20));
        assert_eq!(results.vertical_jump, Some(35.5));
    }

    #[test]
    fn test_invalid_forty_dash() {
        let player_id = Uuid::new_v4();
        let results1 = CombineResults::new(player_id, 2026).unwrap();
        assert!(results1.with_forty_yard_dash(3.9).is_err());

        let results2 = CombineResults::new(player_id, 2026).unwrap();
        assert!(results2.with_forty_yard_dash(6.1).is_err());
    }

    #[test]
    fn test_invalid_bench_press() {
        let player_id = Uuid::new_v4();
        let results1 = CombineResults::new(player_id, 2026).unwrap();
        assert!(results1.with_bench_press(-1).is_err());

        let results2 = CombineResults::new(player_id, 2026).unwrap();
        assert!(results2.with_bench_press(51).is_err());
    }

    #[test]
    fn test_invalid_vertical_jump() {
        let player_id = Uuid::new_v4();
        let results1 = CombineResults::new(player_id, 2026).unwrap();
        assert!(results1.with_vertical_jump(19.9).is_err());

        let results2 = CombineResults::new(player_id, 2026).unwrap();
        assert!(results2.with_vertical_jump(50.1).is_err());
    }
}
