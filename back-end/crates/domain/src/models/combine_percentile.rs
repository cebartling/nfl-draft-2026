use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

/// Valid measurements for percentile lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measurement {
    FortyYardDash,
    BenchPress,
    VerticalJump,
    BroadJump,
    ThreeConeDrill,
    TwentyYardShuttle,
    ArmLength,
    HandSize,
    Wingspan,
    TenYardSplit,
    TwentyYardSplit,
    Height,
    Weight,
}

impl std::fmt::Display for Measurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Measurement::FortyYardDash => write!(f, "forty_yard_dash"),
            Measurement::BenchPress => write!(f, "bench_press"),
            Measurement::VerticalJump => write!(f, "vertical_jump"),
            Measurement::BroadJump => write!(f, "broad_jump"),
            Measurement::ThreeConeDrill => write!(f, "three_cone_drill"),
            Measurement::TwentyYardShuttle => write!(f, "twenty_yard_shuttle"),
            Measurement::ArmLength => write!(f, "arm_length"),
            Measurement::HandSize => write!(f, "hand_size"),
            Measurement::Wingspan => write!(f, "wingspan"),
            Measurement::TenYardSplit => write!(f, "ten_yard_split"),
            Measurement::TwentyYardSplit => write!(f, "twenty_yard_split"),
            Measurement::Height => write!(f, "height"),
            Measurement::Weight => write!(f, "weight"),
        }
    }
}

impl std::str::FromStr for Measurement {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "forty_yard_dash" => Ok(Measurement::FortyYardDash),
            "bench_press" => Ok(Measurement::BenchPress),
            "vertical_jump" => Ok(Measurement::VerticalJump),
            "broad_jump" => Ok(Measurement::BroadJump),
            "three_cone_drill" => Ok(Measurement::ThreeConeDrill),
            "twenty_yard_shuttle" => Ok(Measurement::TwentyYardShuttle),
            "arm_length" => Ok(Measurement::ArmLength),
            "hand_size" => Ok(Measurement::HandSize),
            "wingspan" => Ok(Measurement::Wingspan),
            "ten_yard_split" => Ok(Measurement::TenYardSplit),
            "twenty_yard_split" => Ok(Measurement::TwentyYardSplit),
            "height" => Ok(Measurement::Height),
            "weight" => Ok(Measurement::Weight),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid measurement: {}",
                s
            ))),
        }
    }
}

/// Historical combine percentile breakpoints for a position/measurement pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinePercentile {
    pub id: Uuid,
    pub position: String,
    pub measurement: Measurement,
    pub sample_size: i32,
    pub min_value: f64,
    pub p10: f64,
    pub p20: f64,
    pub p30: f64,
    pub p40: f64,
    pub p50: f64,
    pub p60: f64,
    pub p70: f64,
    pub p80: f64,
    pub p90: f64,
    pub max_value: f64,
    pub years_start: i32,
    pub years_end: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CombinePercentile {
    pub fn new(position: String, measurement: Measurement) -> DomainResult<Self> {
        let valid_positions = [
            "QB", "RB", "WR", "TE", "OT", "IOL", "EDGE", "DL", "LB", "CB", "S", "K", "P",
        ];
        if !valid_positions.contains(&position.as_str()) {
            return Err(DomainError::ValidationError(format!(
                "Invalid position: {}",
                position
            )));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            position,
            measurement,
            sample_size: 0,
            min_value: 0.0,
            p10: 0.0,
            p20: 0.0,
            p30: 0.0,
            p40: 0.0,
            p50: 0.0,
            p60: 0.0,
            p70: 0.0,
            p80: 0.0,
            p90: 0.0,
            max_value: 0.0,
            years_start: 2000,
            years_end: 2025,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn with_percentiles(
        mut self,
        sample_size: i32,
        min_value: f64,
        p10: f64,
        p20: f64,
        p30: f64,
        p40: f64,
        p50: f64,
        p60: f64,
        p70: f64,
        p80: f64,
        p90: f64,
        max_value: f64,
    ) -> DomainResult<Self> {
        if sample_size < 0 {
            return Err(DomainError::ValidationError(
                "Sample size cannot be negative".to_string(),
            ));
        }
        self.sample_size = sample_size;
        self.min_value = min_value;
        self.p10 = p10;
        self.p20 = p20;
        self.p30 = p30;
        self.p40 = p40;
        self.p50 = p50;
        self.p60 = p60;
        self.p70 = p70;
        self.p80 = p80;
        self.p90 = p90;
        self.max_value = max_value;
        Ok(self)
    }

    pub fn with_years(mut self, start: i32, end: i32) -> DomainResult<Self> {
        if start > end {
            return Err(DomainError::ValidationError(format!(
                "years_start ({}) must be <= years_end ({})",
                start, end
            )));
        }
        self.years_start = start;
        self.years_end = end;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_combine_percentile() {
        let cp = CombinePercentile::new("QB".to_string(), Measurement::FortyYardDash).unwrap();
        assert_eq!(cp.position, "QB");
        assert_eq!(cp.measurement, Measurement::FortyYardDash);
        assert_eq!(cp.sample_size, 0);
    }

    #[test]
    fn test_invalid_position() {
        let result = CombinePercentile::new("XX".to_string(), Measurement::FortyYardDash);
        assert!(result.is_err());
    }

    #[test]
    fn test_with_percentiles() {
        let cp = CombinePercentile::new("WR".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(500, 4.2, 4.3, 4.35, 4.4, 4.42, 4.45, 4.5, 4.55, 4.6, 4.7, 5.0)
            .unwrap();
        assert_eq!(cp.sample_size, 500);
        assert_eq!(cp.p50, 4.45);
        assert_eq!(cp.min_value, 4.2);
        assert_eq!(cp.max_value, 5.0);
    }

    #[test]
    fn test_negative_sample_size() {
        let result = CombinePercentile::new("QB".to_string(), Measurement::BenchPress)
            .unwrap()
            .with_percentiles(-1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_with_years() {
        let cp = CombinePercentile::new("RB".to_string(), Measurement::BenchPress)
            .unwrap()
            .with_years(2010, 2025)
            .unwrap();
        assert_eq!(cp.years_start, 2010);
        assert_eq!(cp.years_end, 2025);
    }

    #[test]
    fn test_invalid_years() {
        let result = CombinePercentile::new("RB".to_string(), Measurement::BenchPress)
            .unwrap()
            .with_years(2025, 2010);
        assert!(result.is_err());
    }

    #[test]
    fn test_measurement_display() {
        assert_eq!(Measurement::FortyYardDash.to_string(), "forty_yard_dash");
        assert_eq!(Measurement::BenchPress.to_string(), "bench_press");
        assert_eq!(Measurement::Height.to_string(), "height");
    }

    #[test]
    fn test_measurement_from_str() {
        assert_eq!(
            "forty_yard_dash".parse::<Measurement>().unwrap(),
            Measurement::FortyYardDash
        );
        assert_eq!(
            "height".parse::<Measurement>().unwrap(),
            Measurement::Height
        );
        assert!("invalid".parse::<Measurement>().is_err());
    }
}
