use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use domain::models::RankingSource;

use crate::errors::DbResult;

/// Database model for ranking_sources table
#[derive(Debug, Clone, FromRow)]
pub struct RankingSourceDb {
    pub id: Uuid,
    pub name: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RankingSourceDb {
    pub fn from_domain(source: &RankingSource) -> Self {
        Self {
            id: source.id,
            name: source.name.clone(),
            url: source.url.clone(),
            description: source.description.clone(),
            created_at: source.created_at,
            updated_at: source.updated_at,
        }
    }

    pub fn to_domain(&self) -> DbResult<RankingSource> {
        Ok(RankingSource {
            id: self.id,
            name: self.name.clone(),
            url: self.url.clone(),
            description: self.description.clone(),
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
        let source = RankingSource::new("Tankathon".to_string())
            .unwrap()
            .with_url("https://tankathon.com".to_string())
            .unwrap()
            .with_description("Big board".to_string());

        let db = RankingSourceDb::from_domain(&source);
        assert_eq!(db.name, "Tankathon");
        assert_eq!(db.url, Some("https://tankathon.com".to_string()));
        assert_eq!(db.description, Some("Big board".to_string()));
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let db = RankingSourceDb {
            id: Uuid::new_v4(),
            name: "Tankathon".to_string(),
            url: Some("https://tankathon.com".to_string()),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = db.to_domain();
        assert!(result.is_ok());
        let source = result.unwrap();
        assert_eq!(source.name, "Tankathon");
    }
}
