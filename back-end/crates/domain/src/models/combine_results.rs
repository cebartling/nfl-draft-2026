use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

/// Source of combine/athletic testing data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CombineSource {
    Combine,
    ProDay,
}

impl fmt::Display for CombineSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CombineSource::Combine => write!(f, "combine"),
            CombineSource::ProDay => write!(f, "pro_day"),
        }
    }
}

impl FromStr for CombineSource {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "combine" => Ok(CombineSource::Combine),
            "pro_day" => Ok(CombineSource::ProDay),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid combine source: '{}'. Must be 'combine' or 'pro_day'",
                s
            ))),
        }
    }
}

impl Default for CombineSource {
    fn default() -> Self {
        CombineSource::Combine
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CombineResults {
    pub id: Uuid,
    pub player_id: Uuid,
    pub year: i32,
    pub source: CombineSource,
    pub forty_yard_dash: Option<f64>,
    pub bench_press: Option<i32>,
    pub vertical_jump: Option<f64>,
    pub broad_jump: Option<i32>,
    pub three_cone_drill: Option<f64>,
    pub twenty_yard_shuttle: Option<f64>,
    pub arm_length: Option<f64>,
    pub hand_size: Option<f64>,
    pub wingspan: Option<f64>,
    pub ten_yard_split: Option<f64>,
    pub twenty_yard_split: Option<f64>,
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
            source: CombineSource::default(),
            forty_yard_dash: None,
            bench_press: None,
            vertical_jump: None,
            broad_jump: None,
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            arm_length: None,
            hand_size: None,
            wingspan: None,
            ten_yard_split: None,
            twenty_yard_split: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_source(mut self, source: CombineSource) -> Self {
        self.source = source;
        self
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

    pub fn with_arm_length(mut self, inches: f64) -> DomainResult<Self> {
        Self::validate_arm_length(inches)?;
        self.arm_length = Some(inches);
        Ok(self)
    }

    pub fn with_hand_size(mut self, inches: f64) -> DomainResult<Self> {
        Self::validate_hand_size(inches)?;
        self.hand_size = Some(inches);
        Ok(self)
    }

    pub fn with_wingspan(mut self, inches: f64) -> DomainResult<Self> {
        Self::validate_wingspan(inches)?;
        self.wingspan = Some(inches);
        Ok(self)
    }

    pub fn with_ten_yard_split(mut self, time: f64) -> DomainResult<Self> {
        Self::validate_ten_yard_split(time)?;
        self.ten_yard_split = Some(time);
        Ok(self)
    }

    pub fn with_twenty_yard_split(mut self, time: f64) -> DomainResult<Self> {
        Self::validate_twenty_yard_split(time)?;
        self.twenty_yard_split = Some(time);
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

    pub fn update_arm_length(&mut self, inches: Option<f64>) -> DomainResult<()> {
        if let Some(i) = inches {
            Self::validate_arm_length(i)?;
        }
        self.arm_length = inches;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_hand_size(&mut self, inches: Option<f64>) -> DomainResult<()> {
        if let Some(i) = inches {
            Self::validate_hand_size(i)?;
        }
        self.hand_size = inches;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_wingspan(&mut self, inches: Option<f64>) -> DomainResult<()> {
        if let Some(i) = inches {
            Self::validate_wingspan(i)?;
        }
        self.wingspan = inches;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_ten_yard_split(&mut self, time: Option<f64>) -> DomainResult<()> {
        if let Some(t) = time {
            Self::validate_ten_yard_split(t)?;
        }
        self.ten_yard_split = time;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_twenty_yard_split_time(&mut self, time: Option<f64>) -> DomainResult<()> {
        if let Some(t) = time {
            Self::validate_twenty_yard_split(t)?;
        }
        self.twenty_yard_split = time;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn validate_year(year: i32) -> DomainResult<()> {
        if !(2000..=2100).contains(&year) {
            return Err(DomainError::ValidationError(
                "Combine year must be between 2000 and 2100".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_forty_dash(time: f64) -> DomainResult<()> {
        if !(4.0..=6.0).contains(&time) {
            return Err(DomainError::ValidationError(
                "40-yard dash time must be between 4.0 and 6.0 seconds".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_bench_press(reps: i32) -> DomainResult<()> {
        if !(0..=50).contains(&reps) {
            return Err(DomainError::ValidationError(
                "Bench press reps must be between 0 and 50".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_vertical_jump(inches: f64) -> DomainResult<()> {
        if !(20.0..=50.0).contains(&inches) {
            return Err(DomainError::ValidationError(
                "Vertical jump must be between 20 and 50 inches".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_broad_jump(inches: i32) -> DomainResult<()> {
        if !(80..=150).contains(&inches) {
            return Err(DomainError::ValidationError(
                "Broad jump must be between 80 and 150 inches".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_three_cone_drill(time: f64) -> DomainResult<()> {
        if !(6.0..=9.0).contains(&time) {
            return Err(DomainError::ValidationError(
                "Three cone drill time must be between 6.0 and 9.0 seconds".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_twenty_yard_shuttle(time: f64) -> DomainResult<()> {
        if !(3.5..=6.0).contains(&time) {
            return Err(DomainError::ValidationError(
                "20-yard shuttle time must be between 3.5 and 6.0 seconds".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_arm_length(inches: f64) -> DomainResult<()> {
        if !(28.0..=40.0).contains(&inches) {
            return Err(DomainError::ValidationError(
                "Arm length must be between 28.0 and 40.0 inches".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_hand_size(inches: f64) -> DomainResult<()> {
        if !(7.0..=12.0).contains(&inches) {
            return Err(DomainError::ValidationError(
                "Hand size must be between 7.0 and 12.0 inches".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_wingspan(inches: f64) -> DomainResult<()> {
        if !(70.0..=90.0).contains(&inches) {
            return Err(DomainError::ValidationError(
                "Wingspan must be between 70.0 and 90.0 inches".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_ten_yard_split(time: f64) -> DomainResult<()> {
        if !(1.3..=2.1).contains(&time) {
            return Err(DomainError::ValidationError(
                "10-yard split must be between 1.3 and 2.1 seconds".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_twenty_yard_split(time: f64) -> DomainResult<()> {
        if !(2.3..=3.5).contains(&time) {
            return Err(DomainError::ValidationError(
                "20-yard split must be between 2.3 and 3.5 seconds".to_string(),
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
        assert_eq!(results.source, CombineSource::Combine);
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
    fn test_with_source() {
        let player_id = Uuid::new_v4();
        let results = CombineResults::new(player_id, 2026)
            .unwrap()
            .with_source(CombineSource::ProDay);

        assert_eq!(results.source, CombineSource::ProDay);
    }

    #[test]
    fn test_new_measurables() {
        let player_id = Uuid::new_v4();
        let results = CombineResults::new(player_id, 2026)
            .unwrap()
            .with_arm_length(33.5)
            .unwrap()
            .with_hand_size(9.75)
            .unwrap()
            .with_wingspan(78.5)
            .unwrap()
            .with_ten_yard_split(1.55)
            .unwrap()
            .with_twenty_yard_split(2.65)
            .unwrap();

        assert_eq!(results.arm_length, Some(33.5));
        assert_eq!(results.hand_size, Some(9.75));
        assert_eq!(results.wingspan, Some(78.5));
        assert_eq!(results.ten_yard_split, Some(1.55));
        assert_eq!(results.twenty_yard_split, Some(2.65));
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

    #[test]
    fn test_combine_source_display() {
        assert_eq!(CombineSource::Combine.to_string(), "combine");
        assert_eq!(CombineSource::ProDay.to_string(), "pro_day");
    }

    #[test]
    fn test_combine_source_from_str() {
        assert_eq!(
            CombineSource::from_str("combine").unwrap(),
            CombineSource::Combine
        );
        assert_eq!(
            CombineSource::from_str("pro_day").unwrap(),
            CombineSource::ProDay
        );
        assert!(CombineSource::from_str("invalid").is_err());
    }
}
