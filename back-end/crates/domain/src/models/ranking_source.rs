use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingSource {
    pub id: Uuid,
    pub name: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RankingSource {
    pub fn new(name: String) -> DomainResult<Self> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Ranking source name cannot be empty".to_string(),
            ));
        }
        if name.len() > 100 {
            return Err(DomainError::ValidationError(
                "Ranking source name cannot exceed 100 characters".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            url: None,
            description: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn with_url(mut self, url: String) -> DomainResult<Self> {
        if url.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "URL cannot be empty".to_string(),
            ));
        }
        if url.len() > 500 {
            return Err(DomainError::ValidationError(
                "URL cannot exceed 500 characters".to_string(),
            ));
        }
        self.url = Some(url);
        Ok(self)
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ranking_source() {
        let source = RankingSource::new("Tankathon".to_string()).unwrap();
        assert_eq!(source.name, "Tankathon");
        assert!(source.url.is_none());
        assert!(source.description.is_none());
    }

    #[test]
    fn test_with_url() {
        let source = RankingSource::new("Tankathon".to_string())
            .unwrap()
            .with_url("https://tankathon.com".to_string())
            .unwrap();
        assert_eq!(source.url, Some("https://tankathon.com".to_string()));
    }

    #[test]
    fn test_with_description() {
        let source = RankingSource::new("Tankathon".to_string())
            .unwrap()
            .with_description("Big board rankings".to_string());
        assert_eq!(
            source.description,
            Some("Big board rankings".to_string())
        );
    }

    #[test]
    fn test_empty_name_rejected() {
        let result = RankingSource::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_name_too_long_rejected() {
        let long_name = "x".repeat(101);
        let result = RankingSource::new(long_name);
        assert!(result.is_err());
    }
}
