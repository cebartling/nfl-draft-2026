use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProspectRanking {
    pub id: Uuid,
    pub ranking_source_id: Uuid,
    pub player_id: Uuid,
    pub rank: i32,
    pub scraped_at: NaiveDate,
    pub created_at: DateTime<Utc>,
}

impl ProspectRanking {
    pub fn new(
        ranking_source_id: Uuid,
        player_id: Uuid,
        rank: i32,
        scraped_at: NaiveDate,
    ) -> DomainResult<Self> {
        if rank <= 0 {
            return Err(DomainError::ValidationError(
                "Rank must be positive".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            ranking_source_id,
            player_id,
            rank,
            scraped_at,
            created_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_prospect_ranking() {
        let source_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        let ranking = ProspectRanking::new(source_id, player_id, 1, date).unwrap();
        assert_eq!(ranking.ranking_source_id, source_id);
        assert_eq!(ranking.player_id, player_id);
        assert_eq!(ranking.rank, 1);
        assert_eq!(ranking.scraped_at, date);
    }

    #[test]
    fn test_zero_rank_rejected() {
        let result = ProspectRanking::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            0,
            NaiveDate::from_ymd_opt(2026, 2, 11).unwrap(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_rank_rejected() {
        let result = ProspectRanking::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            -1,
            NaiveDate::from_ymd_opt(2026, 2, 11).unwrap(),
        );
        assert!(result.is_err());
    }
}
