use std::sync::Arc;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::{DraftStrategy, Player, Position};
use crate::repositories::{DraftStrategyRepository, TeamNeedRepository};

/// Service for managing draft strategies and calculating need-based scores
pub struct DraftStrategyService {
    strategy_repo: Arc<dyn DraftStrategyRepository>,
    need_repo: Arc<dyn TeamNeedRepository>,
}

impl DraftStrategyService {
    pub fn new(
        strategy_repo: Arc<dyn DraftStrategyRepository>,
        need_repo: Arc<dyn TeamNeedRepository>,
    ) -> Self {
        Self {
            strategy_repo,
            need_repo,
        }
    }

    /// Get strategy for team/draft, or return default if not found
    pub async fn get_or_default_strategy(
        &self,
        team_id: Uuid,
        draft_id: Uuid,
    ) -> DomainResult<DraftStrategy> {
        match self
            .strategy_repo
            .find_by_team_and_draft(team_id, draft_id)
            .await?
        {
            Some(strategy) => Ok(strategy),
            None => {
                // Create and save default strategy
                let default_strategy = DraftStrategy::default_strategy(team_id, draft_id);
                self.strategy_repo.create(&default_strategy).await
            }
        }
    }

    /// Create or update a draft strategy
    pub async fn set_strategy(&self, strategy: &DraftStrategy) -> DomainResult<DraftStrategy> {
        // Check if strategy already exists
        match self
            .strategy_repo
            .find_by_team_and_draft(strategy.team_id, strategy.draft_id)
            .await?
        {
            Some(existing) => {
                // Update existing strategy
                let updated = DraftStrategy {
                    id: existing.id,
                    ..strategy.clone()
                };
                self.strategy_repo.update(&updated).await
            }
            None => {
                // Create new strategy
                self.strategy_repo.create(strategy).await
            }
        }
    }

    /// Calculate need score for a player based on team needs
    /// Priority 1 (highest need) = 100, Priority 10 (lowest need) = 10
    /// Formula: (11 - priority) × 10
    pub async fn calculate_need_score(&self, player: &Player, team_id: Uuid) -> DomainResult<f64> {
        let needs = self.need_repo.find_by_team_id(team_id).await?;
        Ok(Self::calculate_need_score_from_needs(player, &needs))
    }

    /// Calculate need score using pre-fetched team needs (avoids repeated DB queries)
    pub fn calculate_need_score_from_needs(
        player: &Player,
        needs: &[crate::models::TeamNeed],
    ) -> f64 {
        let matching_need = needs.iter().find(|need| need.position == player.position);

        match matching_need {
            Some(need) => (11 - need.priority) as f64 * 10.0,
            None => 10.0,
        }
    }

    /// Fetch team needs (for pre-loading in batch operations)
    pub async fn fetch_team_needs(
        &self,
        team_id: Uuid,
    ) -> DomainResult<Vec<crate::models::TeamNeed>> {
        self.need_repo.find_by_team_id(team_id).await
    }

    /// Get position value multiplier from strategy
    pub fn get_position_value(&self, strategy: &DraftStrategy, position: Position) -> f64 {
        strategy.get_position_value(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TeamNeed;
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        DraftStrategyRepo {}

        #[async_trait::async_trait]
        impl DraftStrategyRepository for DraftStrategyRepo {
            async fn create(&self, strategy: &DraftStrategy) -> DomainResult<DraftStrategy>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftStrategy>>;
            async fn find_by_team_and_draft(&self, team_id: Uuid, draft_id: Uuid) -> DomainResult<Option<DraftStrategy>>;
            async fn find_by_draft_id(&self, draft_id: Uuid) -> DomainResult<Vec<DraftStrategy>>;
            async fn update(&self, strategy: &DraftStrategy) -> DomainResult<DraftStrategy>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
        }
    }

    mock! {
        TeamNeedRepo {}

        #[async_trait::async_trait]
        impl TeamNeedRepository for TeamNeedRepo {
            async fn create(&self, need: &TeamNeed) -> DomainResult<TeamNeed>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<TeamNeed>>;
            async fn find_by_team_id(&self, team_id: Uuid) -> DomainResult<Vec<TeamNeed>>;
            async fn update(&self, need: &TeamNeed) -> DomainResult<TeamNeed>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
            async fn delete_by_team_id(&self, team_id: Uuid) -> DomainResult<()>;
        }
    }

    fn create_test_player(position: Position) -> Player {
        Player::new("John".to_string(), "Doe".to_string(), position, 2026).unwrap()
    }

    #[tokio::test]
    async fn test_get_or_default_strategy_creates_default() {
        let mut strategy_mock = MockDraftStrategyRepo::new();
        let need_mock = MockTeamNeedRepo::new();

        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        // First call returns None (no existing strategy)
        strategy_mock
            .expect_find_by_team_and_draft()
            .with(eq(team_id), eq(draft_id))
            .times(1)
            .returning(|_, _| Ok(None));

        // Expect create to be called with default values
        strategy_mock
            .expect_create()
            .times(1)
            .returning(move |strategy| {
                assert_eq!(strategy.team_id, team_id);
                assert_eq!(strategy.draft_id, draft_id);
                assert_eq!(strategy.bpa_weight, 60);
                assert_eq!(strategy.need_weight, 40);
                Ok(strategy.clone())
            });

        let service = DraftStrategyService::new(Arc::new(strategy_mock), Arc::new(need_mock));

        let strategy = service
            .get_or_default_strategy(team_id, draft_id)
            .await
            .unwrap();

        assert_eq!(strategy.bpa_weight, 60);
        assert_eq!(strategy.need_weight, 40);
        assert_eq!(strategy.risk_tolerance, 5);
    }

    #[tokio::test]
    async fn test_get_or_default_strategy_returns_existing() {
        let mut strategy_mock = MockDraftStrategyRepo::new();
        let need_mock = MockTeamNeedRepo::new();

        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        let mut existing_strategy = DraftStrategy::default_strategy(team_id, draft_id);
        existing_strategy.bpa_weight = 70;
        existing_strategy.need_weight = 30;

        strategy_mock
            .expect_find_by_team_and_draft()
            .with(eq(team_id), eq(draft_id))
            .times(1)
            .returning(move |_, _| Ok(Some(existing_strategy.clone())));

        let service = DraftStrategyService::new(Arc::new(strategy_mock), Arc::new(need_mock));

        let strategy = service
            .get_or_default_strategy(team_id, draft_id)
            .await
            .unwrap();

        assert_eq!(strategy.bpa_weight, 70);
        assert_eq!(strategy.need_weight, 30);
    }

    #[tokio::test]
    async fn test_calculate_need_score_high_priority() {
        let strategy_mock = MockDraftStrategyRepo::new();
        let mut need_mock = MockTeamNeedRepo::new();

        let team_id = Uuid::new_v4();
        let player = create_test_player(Position::QB);

        // Priority 1 need for QB
        let need = TeamNeed::new(team_id, Position::QB, 1).unwrap();

        need_mock
            .expect_find_by_team_id()
            .with(eq(team_id))
            .times(1)
            .returning(move |_| Ok(vec![need.clone()]));

        let service = DraftStrategyService::new(Arc::new(strategy_mock), Arc::new(need_mock));

        let score = service
            .calculate_need_score(&player, team_id)
            .await
            .unwrap();

        // Priority 1: (11 - 1) * 10 = 100
        assert_eq!(score, 100.0);
    }

    #[tokio::test]
    async fn test_calculate_need_score_low_priority() {
        let strategy_mock = MockDraftStrategyRepo::new();
        let mut need_mock = MockTeamNeedRepo::new();

        let team_id = Uuid::new_v4();
        let player = create_test_player(Position::QB);

        // Priority 10 need for QB
        let need = TeamNeed::new(team_id, Position::QB, 10).unwrap();

        need_mock
            .expect_find_by_team_id()
            .with(eq(team_id))
            .times(1)
            .returning(move |_| Ok(vec![need.clone()]));

        let service = DraftStrategyService::new(Arc::new(strategy_mock), Arc::new(need_mock));

        let score = service
            .calculate_need_score(&player, team_id)
            .await
            .unwrap();

        // Priority 10: (11 - 10) * 10 = 10
        assert_eq!(score, 10.0);
    }

    #[tokio::test]
    async fn test_calculate_need_score_no_need() {
        let strategy_mock = MockDraftStrategyRepo::new();
        let mut need_mock = MockTeamNeedRepo::new();

        let team_id = Uuid::new_v4();
        let player = create_test_player(Position::QB);

        // Team has needs, but not for QB
        let needs = vec![
            TeamNeed::new(team_id, Position::WR, 1).unwrap(),
            TeamNeed::new(team_id, Position::DE, 2).unwrap(),
        ];

        need_mock
            .expect_find_by_team_id()
            .with(eq(team_id))
            .times(1)
            .returning(move |_| Ok(needs.clone()));

        let service = DraftStrategyService::new(Arc::new(strategy_mock), Arc::new(need_mock));

        let score = service
            .calculate_need_score(&player, team_id)
            .await
            .unwrap();

        // No matching need = low score
        assert_eq!(score, 10.0);
    }

    #[test]
    fn test_get_position_value() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let strategy = DraftStrategy::default_strategy(team_id, draft_id);

        let service = DraftStrategyService::new(
            Arc::new(MockDraftStrategyRepo::new()),
            Arc::new(MockTeamNeedRepo::new()),
        );

        assert_eq!(service.get_position_value(&strategy, Position::QB), 1.5);
        assert_eq!(service.get_position_value(&strategy, Position::DE), 1.3);
        assert_eq!(service.get_position_value(&strategy, Position::RB), 0.85);
        assert_eq!(service.get_position_value(&strategy, Position::K), 0.5);
    }
}
