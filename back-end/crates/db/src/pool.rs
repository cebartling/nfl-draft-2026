use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Create a PostgreSQL connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pool_success() {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        let result = create_pool(&database_url).await;
        assert!(result.is_ok());

        let pool = result.unwrap();
        // Pool initializes with min_connections (1)
        assert!(pool.size() >= 1);
    }

    #[tokio::test]
    async fn test_create_pool_invalid_url() {
        let result = create_pool("postgresql://invalid:invalid@localhost:9999/invalid").await;
        assert!(result.is_err());
    }
}
