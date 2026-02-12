use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::ProspectRanking;

use crate::errors::DbResult;

/// Database model for prospect_rankings table
#[derive(Debug, Clone, FromRow)]
pub struct ProspectRankingDb {
    pub id: Uuid,
    pub ranking_source_id: Uuid,
    pub player_id: Uuid,
    pub rank: i32,
    pub scraped_at: NaiveDate,
    pub created_at: DateTime<Utc>,
}

impl ProspectRankingDb {
    pub fn from_domain(ranking: &ProspectRanking) -> Self {
        Self {
            id: ranking.id,
            ranking_source_id: ranking.ranking_source_id,
            player_id: ranking.player_id,
            rank: ranking.rank,
            scraped_at: ranking.scraped_at,
            created_at: ranking.created_at,
        }
    }

    pub fn to_domain(&self) -> DbResult<ProspectRanking> {
        Ok(ProspectRanking {
            id: self.id,
            ranking_source_id: self.ranking_source_id,
            player_id: self.player_id,
            rank: self.rank,
            scraped_at: self.scraped_at,
            created_at: self.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_to_db_conversion() {
        let source_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        let ranking = ProspectRanking::new(source_id, player_id, 1, date).unwrap();
        let db = ProspectRankingDb::from_domain(&ranking);

        assert_eq!(db.ranking_source_id, source_id);
        assert_eq!(db.player_id, player_id);
        assert_eq!(db.rank, 1);
        assert_eq!(db.scraped_at, date);
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let db = ProspectRankingDb {
            id: Uuid::new_v4(),
            ranking_source_id: Uuid::new_v4(),
            player_id: Uuid::new_v4(),
            rank: 5,
            scraped_at: NaiveDate::from_ymd_opt(2026, 2, 11).unwrap(),
            created_at: Utc::now(),
        };

        let result = db.to_domain();
        assert!(result.is_ok());
        let ranking = result.unwrap();
        assert_eq!(ranking.rank, 5);
    }
}
