use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::CombineResults;

use crate::errors::DbResult;

/// Database model for combine_results table
#[derive(Debug, Clone, FromRow)]
pub struct CombineResultsDb {
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

impl CombineResultsDb {
    /// Convert from domain CombineResults to database CombineResultsDb
    pub fn from_domain(results: &CombineResults) -> Self {
        Self {
            id: results.id,
            player_id: results.player_id,
            year: results.year,
            forty_yard_dash: results.forty_yard_dash,
            bench_press: results.bench_press,
            vertical_jump: results.vertical_jump,
            broad_jump: results.broad_jump,
            three_cone_drill: results.three_cone_drill,
            twenty_yard_shuttle: results.twenty_yard_shuttle,
            created_at: results.created_at,
            updated_at: results.updated_at,
        }
    }

    /// Convert from database CombineResultsDb to domain CombineResults
    pub fn to_domain(&self) -> DbResult<CombineResults> {
        Ok(CombineResults {
            id: self.id,
            player_id: self.player_id,
            year: self.year,
            forty_yard_dash: self.forty_yard_dash,
            bench_press: self.bench_press,
            vertical_jump: self.vertical_jump,
            broad_jump: self.broad_jump,
            three_cone_drill: self.three_cone_drill,
            twenty_yard_shuttle: self.twenty_yard_shuttle,
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
        let results = CombineResults::new(player_id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.52)
            .unwrap()
            .with_bench_press(20)
            .unwrap();

        let results_db = CombineResultsDb::from_domain(&results);
        assert_eq!(results_db.player_id, player_id);
        assert_eq!(results_db.year, 2026);
        assert_eq!(results_db.forty_yard_dash, Some(4.52));
        assert_eq!(results_db.bench_press, Some(20));
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let results_db = CombineResultsDb {
            id: Uuid::new_v4(),
            player_id: Uuid::new_v4(),
            year: 2026,
            forty_yard_dash: Some(4.52),
            bench_press: Some(20),
            vertical_jump: Some(35.5),
            broad_jump: None,
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = results_db.to_domain();
        assert!(result.is_ok());

        let results = result.unwrap();
        assert_eq!(results.year, 2026);
        assert_eq!(results.forty_yard_dash, Some(4.52));
        assert_eq!(results.vertical_jump, Some(35.5));
    }
}
