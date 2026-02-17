use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::DbError;
use domain::models::CombinePercentile;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CombinePercentileDb {
    pub id: Uuid,
    pub position: String,
    pub measurement: String,
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

impl CombinePercentileDb {
    pub fn from_domain(p: &CombinePercentile) -> Self {
        Self {
            id: p.id,
            position: p.position.clone(),
            measurement: p.measurement.to_string(),
            sample_size: p.sample_size,
            min_value: p.min_value,
            p10: p.p10,
            p20: p.p20,
            p30: p.p30,
            p40: p.p40,
            p50: p.p50,
            p60: p.p60,
            p70: p.p70,
            p80: p.p80,
            p90: p.p90,
            max_value: p.max_value,
            years_start: p.years_start,
            years_end: p.years_end,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }

    pub fn to_domain(self) -> Result<CombinePercentile, DbError> {
        let measurement = self
            .measurement
            .parse()
            .map_err(|_| DbError::MappingError(format!("Invalid measurement: {}", self.measurement)))?;

        Ok(CombinePercentile {
            id: self.id,
            position: self.position,
            measurement,
            sample_size: self.sample_size,
            min_value: self.min_value,
            p10: self.p10,
            p20: self.p20,
            p30: self.p30,
            p40: self.p40,
            p50: self.p50,
            p60: self.p60,
            p70: self.p70,
            p80: self.p80,
            p90: self.p90,
            max_value: self.max_value,
            years_start: self.years_start,
            years_end: self.years_end,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::Measurement;

    #[test]
    fn test_domain_to_db_conversion() {
        let domain = CombinePercentile::new("QB".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(100, 4.2, 4.3, 4.35, 4.4, 4.42, 4.45, 4.5, 4.55, 4.6, 4.7, 5.0)
            .unwrap();

        let db = CombinePercentileDb::from_domain(&domain);
        assert_eq!(db.position, "QB");
        assert_eq!(db.measurement, "forty_yard_dash");
        assert_eq!(db.sample_size, 100);
        assert_eq!(db.p50, 4.45);
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let db = CombinePercentileDb {
            id: Uuid::new_v4(),
            position: "WR".to_string(),
            measurement: "vertical_jump".to_string(),
            sample_size: 200,
            min_value: 25.0,
            p10: 28.0,
            p20: 30.0,
            p30: 31.0,
            p40: 32.0,
            p50: 33.5,
            p60: 35.0,
            p70: 36.5,
            p80: 38.0,
            p90: 40.0,
            max_value: 46.0,
            years_start: 2000,
            years_end: 2025,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let domain = db.to_domain().unwrap();
        assert_eq!(domain.position, "WR");
        assert_eq!(domain.measurement, Measurement::VerticalJump);
        assert_eq!(domain.p50, 33.5);
    }
}
