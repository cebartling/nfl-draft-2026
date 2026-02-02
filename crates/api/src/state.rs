use sqlx::PgPool;
use std::sync::Arc;

use domain::repositories::{TeamRepository, PlayerRepository};
use db::repositories::{SqlxTeamRepository, SqlxPlayerRepository};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub team_repo: Arc<dyn TeamRepository>,
    pub player_repo: Arc<dyn PlayerRepository>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            team_repo: Arc::new(SqlxTeamRepository::new(pool.clone())),
            player_repo: Arc::new(SqlxPlayerRepository::new(pool)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft".to_string());

        let pool = db::create_pool(&database_url).await.expect("Failed to create pool");
        let state = AppState::new(pool);

        // Just verify state was created successfully
        assert!(Arc::strong_count(&state.team_repo) >= 1);
        assert!(Arc::strong_count(&state.player_repo) >= 1);
    }
}
