use sqlx::PgPool;
use std::sync::Arc;

use domain::repositories::{TeamRepository, PlayerRepository, DraftRepository, DraftPickRepository};
use domain::services::DraftEngine;
use db::repositories::{SqlxTeamRepository, SqlxPlayerRepository, SqlxDraftRepository, SqlxDraftPickRepository};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub team_repo: Arc<dyn TeamRepository>,
    pub player_repo: Arc<dyn PlayerRepository>,
    pub draft_repo: Arc<dyn DraftRepository>,
    pub draft_pick_repo: Arc<dyn DraftPickRepository>,
    pub draft_engine: Arc<DraftEngine>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let team_repo: Arc<dyn TeamRepository> = Arc::new(SqlxTeamRepository::new(pool.clone()));
        let player_repo: Arc<dyn PlayerRepository> = Arc::new(SqlxPlayerRepository::new(pool.clone()));
        let draft_repo: Arc<dyn DraftRepository> = Arc::new(SqlxDraftRepository::new(pool.clone()));
        let draft_pick_repo: Arc<dyn DraftPickRepository> = Arc::new(SqlxDraftPickRepository::new(pool));

        let draft_engine = Arc::new(DraftEngine::new(
            draft_repo.clone(),
            draft_pick_repo.clone(),
            team_repo.clone(),
            player_repo.clone(),
        ));

        Self {
            team_repo,
            player_repo,
            draft_repo,
            draft_pick_repo,
            draft_engine,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string());

        let pool = db::create_pool(&database_url).await.expect("Failed to create pool");
        let state = AppState::new(pool);

        // Just verify state was created successfully
        assert!(Arc::strong_count(&state.team_repo) >= 1);
        assert!(Arc::strong_count(&state.player_repo) >= 1);
    }
}
