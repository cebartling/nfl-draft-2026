use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use db::repositories::{
    EventRepo, SessionRepo, SqlxCombineResultsRepository, SqlxDraftPickRepository,
    SqlxDraftRepository, SqlxDraftStrategyRepository, SqlxPlayerRepository,
    SqlxScoutingReportRepository, SqlxTeamNeedRepository, SqlxTeamRepository,
    SqlxTeamSeasonRepository, SqlxTradeRepository,
};
use domain::repositories::{
    CombineResultsRepository, DraftPickRepository, DraftRepository, DraftStrategyRepository,
    EventRepository, PlayerRepository, ScoutingReportRepository, SessionRepository,
    TeamNeedRepository, TeamRepository, TeamSeasonRepository, TradeRepository,
};
use domain::services::{
    AutoPickService, DraftEngine, DraftStrategyService, PlayerEvaluationService, TradeEngine,
};
use websocket::ConnectionManager;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub team_repo: Arc<dyn TeamRepository>,
    pub player_repo: Arc<dyn PlayerRepository>,
    pub draft_repo: Arc<dyn DraftRepository>,
    pub draft_pick_repo: Arc<dyn DraftPickRepository>,
    pub combine_results_repo: Arc<dyn CombineResultsRepository>,
    pub scouting_report_repo: Arc<dyn ScoutingReportRepository>,
    pub team_need_repo: Arc<dyn TeamNeedRepository>,
    pub team_season_repo: Arc<dyn TeamSeasonRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub event_repo: Arc<dyn EventRepository>,
    pub trade_repo: Arc<dyn TradeRepository>,
    pub draft_engine: Arc<DraftEngine>,
    pub trade_engine: Arc<TradeEngine>,
    pub ws_manager: ConnectionManager,
    pub seed_api_key: Option<String>,
    /// Per-session mutex to prevent concurrent auto-pick-run requests
    pub session_locks: Arc<DashMap<Uuid, Arc<Mutex<()>>>>,
}

impl AppState {
    pub fn new(pool: PgPool, seed_api_key: Option<String>) -> Self {
        let team_repo: Arc<dyn TeamRepository> = Arc::new(SqlxTeamRepository::new(pool.clone()));
        let player_repo: Arc<dyn PlayerRepository> =
            Arc::new(SqlxPlayerRepository::new(pool.clone()));
        let draft_repo: Arc<dyn DraftRepository> = Arc::new(SqlxDraftRepository::new(pool.clone()));
        let draft_pick_repo: Arc<dyn DraftPickRepository> =
            Arc::new(SqlxDraftPickRepository::new(pool.clone()));
        let combine_results_repo: Arc<dyn CombineResultsRepository> =
            Arc::new(SqlxCombineResultsRepository::new(pool.clone()));
        let scouting_report_repo: Arc<dyn ScoutingReportRepository> =
            Arc::new(SqlxScoutingReportRepository::new(pool.clone()));
        let team_need_repo: Arc<dyn TeamNeedRepository> =
            Arc::new(SqlxTeamNeedRepository::new(pool.clone()));
        let team_season_repo: Arc<dyn TeamSeasonRepository> =
            Arc::new(SqlxTeamSeasonRepository::new(pool.clone()));
        let session_repo: Arc<dyn SessionRepository> = Arc::new(SessionRepo::new(pool.clone()));
        let event_repo: Arc<dyn EventRepository> = Arc::new(EventRepo::new(pool.clone()));
        let trade_repo: Arc<dyn TradeRepository> = Arc::new(SqlxTradeRepository::new(pool.clone()));
        let draft_strategy_repo: Arc<dyn DraftStrategyRepository> =
            Arc::new(SqlxDraftStrategyRepository::new(pool.clone()));

        let player_eval_service = Arc::new(PlayerEvaluationService::new(
            scouting_report_repo.clone(),
            combine_results_repo.clone(),
        ));

        let strategy_service = Arc::new(DraftStrategyService::new(
            draft_strategy_repo,
            team_need_repo.clone(),
        ));

        let auto_pick_service =
            Arc::new(AutoPickService::new(player_eval_service, strategy_service));

        let draft_engine = Arc::new(
            DraftEngine::new(
                draft_repo.clone(),
                draft_pick_repo.clone(),
                team_repo.clone(),
                player_repo.clone(),
            )
            .with_team_season_repo(team_season_repo.clone())
            .with_auto_pick(auto_pick_service),
        );

        let trade_engine = Arc::new(TradeEngine::with_default_chart(
            trade_repo.clone(),
            draft_pick_repo.clone(),
            team_repo.clone(),
        ));

        let ws_manager = ConnectionManager::new();
        let session_locks = Arc::new(DashMap::new());

        Self {
            team_repo,
            player_repo,
            draft_repo,
            draft_pick_repo,
            combine_results_repo,
            scouting_report_repo,
            team_need_repo,
            team_season_repo,
            session_repo,
            event_repo,
            trade_repo,
            draft_engine,
            trade_engine,
            ws_manager,
            seed_api_key,
            session_locks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        let pool = db::create_pool(&database_url)
            .await
            .expect("Failed to create pool");
        let state = AppState::new(pool, None);

        // Just verify state was created successfully
        assert!(Arc::strong_count(&state.team_repo) >= 1);
        assert!(Arc::strong_count(&state.player_repo) >= 1);
    }
}
