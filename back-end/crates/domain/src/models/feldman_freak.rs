use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeldmanFreak {
    pub id: Uuid,
    pub player_id: Uuid,
    pub year: i32,
    pub rank: i32,
    pub description: String,
    pub article_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl FeldmanFreak {
    pub fn new(player_id: Uuid, year: i32, rank: i32, description: String) -> DomainResult<Self> {
        Self::validate_year(year)?;
        Self::validate_rank(rank)?;
        Self::validate_description(&description)?;

        Ok(Self {
            id: Uuid::new_v4(),
            player_id,
            year,
            rank,
            description,
            article_url: None,
            created_at: Utc::now(),
        })
    }

    pub fn with_article_url(mut self, url: String) -> DomainResult<Self> {
        if url.len() > 500 {
            return Err(DomainError::ValidationError(
                "Article URL cannot exceed 500 characters".to_string(),
            ));
        }
        self.article_url = Some(url);
        Ok(self)
    }

    fn validate_year(year: i32) -> DomainResult<()> {
        if !(2020..=2030).contains(&year) {
            return Err(DomainError::ValidationError(format!(
                "Year must be between 2020 and 2030, got {}",
                year
            )));
        }
        Ok(())
    }

    fn validate_rank(rank: i32) -> DomainResult<()> {
        if rank <= 0 {
            return Err(DomainError::ValidationError(format!(
                "Rank must be positive, got {}",
                rank
            )));
        }
        Ok(())
    }

    fn validate_description(description: &str) -> DomainResult<()> {
        if description.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Description cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feldman_freak() {
        let player_id = Uuid::new_v4();
        let freak =
            FeldmanFreak::new(player_id, 2026, 1, "Vertical jumped 41.5 inches".to_string())
                .unwrap();

        assert_eq!(freak.player_id, player_id);
        assert_eq!(freak.year, 2026);
        assert_eq!(freak.rank, 1);
        assert_eq!(freak.description, "Vertical jumped 41.5 inches");
        assert!(freak.article_url.is_none());
    }

    #[test]
    fn test_with_article_url() {
        let player_id = Uuid::new_v4();
        let freak =
            FeldmanFreak::new(player_id, 2026, 1, "Vertical jumped 41.5 inches".to_string())
                .unwrap()
                .with_article_url("https://example.com/freaks".to_string())
                .unwrap();

        assert_eq!(
            freak.article_url,
            Some("https://example.com/freaks".to_string())
        );
    }

    #[test]
    fn test_invalid_year() {
        let player_id = Uuid::new_v4();
        assert!(FeldmanFreak::new(player_id, 2019, 1, "desc".to_string()).is_err());
        assert!(FeldmanFreak::new(player_id, 2031, 1, "desc".to_string()).is_err());
    }

    #[test]
    fn test_invalid_rank() {
        let player_id = Uuid::new_v4();
        assert!(FeldmanFreak::new(player_id, 2026, 0, "desc".to_string()).is_err());
        assert!(FeldmanFreak::new(player_id, 2026, -1, "desc".to_string()).is_err());
    }

    #[test]
    fn test_empty_description() {
        let player_id = Uuid::new_v4();
        assert!(FeldmanFreak::new(player_id, 2026, 1, "".to_string()).is_err());
        assert!(FeldmanFreak::new(player_id, 2026, 1, "   ".to_string()).is_err());
    }

    #[test]
    fn test_article_url_too_long() {
        let player_id = Uuid::new_v4();
        let freak = FeldmanFreak::new(player_id, 2026, 1, "desc".to_string()).unwrap();
        let long_url = "a".repeat(501);
        assert!(freak.with_article_url(long_url).is_err());
    }
}
