use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::FeldmanFreak;

use crate::errors::DbResult;

/// Database model for feldman_freaks table
#[derive(Debug, Clone, FromRow)]
pub struct FeldmanFreakDb {
    pub id: Uuid,
    pub player_id: Uuid,
    pub year: i32,
    pub rank: i32,
    pub description: String,
    pub article_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl FeldmanFreakDb {
    /// Convert from domain FeldmanFreak to database FeldmanFreakDb
    pub fn from_domain(freak: &FeldmanFreak) -> Self {
        Self {
            id: freak.id,
            player_id: freak.player_id,
            year: freak.year,
            rank: freak.rank,
            description: freak.description.clone(),
            article_url: freak.article_url.clone(),
            created_at: freak.created_at,
        }
    }

    /// Convert from database FeldmanFreakDb to domain FeldmanFreak
    pub fn to_domain(&self) -> DbResult<FeldmanFreak> {
        Ok(FeldmanFreak {
            id: self.id,
            player_id: self.player_id,
            year: self.year,
            rank: self.rank,
            description: self.description.clone(),
            article_url: self.article_url.clone(),
            created_at: self.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_to_db_conversion() {
        let player_id = Uuid::new_v4();
        let freak = FeldmanFreak::new(
            player_id,
            2026,
            1,
            "Vertical jumped 41.5 inches".to_string(),
        )
        .unwrap()
        .with_article_url("https://example.com/freaks".to_string())
        .unwrap();

        let freak_db = FeldmanFreakDb::from_domain(&freak);
        assert_eq!(freak_db.player_id, player_id);
        assert_eq!(freak_db.year, 2026);
        assert_eq!(freak_db.rank, 1);
        assert_eq!(freak_db.description, "Vertical jumped 41.5 inches");
        assert_eq!(
            freak_db.article_url,
            Some("https://example.com/freaks".to_string())
        );
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let freak_db = FeldmanFreakDb {
            id: Uuid::new_v4(),
            player_id: Uuid::new_v4(),
            year: 2026,
            rank: 5,
            description: "Bench pressed 425 lbs".to_string(),
            article_url: None,
            created_at: Utc::now(),
        };

        let result = freak_db.to_domain();
        assert!(result.is_ok());

        let freak = result.unwrap();
        assert_eq!(freak.year, 2026);
        assert_eq!(freak.rank, 5);
        assert_eq!(freak.description, "Bench pressed 425 lbs");
        assert!(freak.article_url.is_none());
    }
}
