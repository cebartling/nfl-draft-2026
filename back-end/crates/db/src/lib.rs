pub mod errors;
pub mod models;
pub mod pool;
pub mod repositories;

pub use errors::{DbError, DbResult};
pub use pool::create_pool;

#[cfg(test)]
pub async fn get_test_pool() -> sqlx::PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
    });

    create_pool(&database_url)
        .await
        .expect("Failed to create test pool")
}
