use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};
use crate::models::Player;
use crate::services::{DraftStrategyService, PlayerEvaluationService, RasScoringService};

/// Result of player scoring with detailed breakdown
#[derive(Debug, Clone)]
pub struct PlayerScore {
    pub player_id: Uuid,
    pub bpa_score: f64,
    pub need_score: f64,
    pub position_value: f64,
    pub final_score: f64,
    pub rationale: String,
}

/// Service for making auto-pick decisions
pub struct AutoPickService {
    player_eval_service: Arc<PlayerEvaluationService>,
    strategy_service: Arc<DraftStrategyService>,
}

impl AutoPickService {
    pub fn new(
        player_eval_service: Arc<PlayerEvaluationService>,
        strategy_service: Arc<DraftStrategyService>,
    ) -> Self {
        Self {
            player_eval_service,
            strategy_service,
        }
    }

    /// Decide which player to pick based on team strategy
    /// Returns the selected player ID and the scoring breakdown
    pub async fn decide_pick(
        &self,
        team_id: Uuid,
        draft_id: Uuid,
        available_players: &[Player],
    ) -> DomainResult<(Uuid, Vec<PlayerScore>)> {
        if available_players.is_empty() {
            return Err(DomainError::ValidationError(
                "No available players to choose from".to_string(),
            ));
        }

        // Get team's draft strategy
        let strategy = self
            .strategy_service
            .get_or_default_strategy(team_id, draft_id)
            .await?;

        // Score all available players
        let scored_players = self
            .score_all_players(team_id, available_players, &strategy)
            .await?;

        if scored_players.is_empty() {
            return Err(DomainError::NotFound(
                "No players could be scored (missing scouting reports)".to_string(),
            ));
        }

        // Select player with highest final score
        let selected = scored_players
            .iter()
            .max_by(|a, b| {
                a.final_score
                    .partial_cmp(&b.final_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        Ok((selected.player_id, scored_players))
    }

    /// Score all players and return sorted by final score (descending).
    /// Pre-fetches team needs, scouting reports, combine results, and percentile data
    /// to avoid N+1 query patterns.
    async fn score_all_players(
        &self,
        team_id: Uuid,
        players: &[Player],
        strategy: &crate::models::DraftStrategy,
    ) -> DomainResult<Vec<PlayerScore>> {
        // Pre-fetch team needs (1 query instead of N)
        let team_needs = self.strategy_service.fetch_team_needs(team_id).await?;

        // Pre-fetch scouting reports for this team (1 query instead of N)
        let scouting_reports = self
            .player_eval_service
            .fetch_team_scouting_reports(team_id)
            .await?;
        let scouting_by_player: HashMap<Uuid, _> = scouting_reports
            .into_iter()
            .map(|r| (r.player_id, r))
            .collect();

        // Pre-fetch combine results for all players (N queries, but eliminates 10N RAS sub-queries)
        let mut combine_by_player: HashMap<Uuid, crate::models::CombineResults> = HashMap::new();
        for player in players {
            if let Ok(results) = self
                .player_eval_service
                .fetch_player_combine_results(player.id)
                .await
            {
                if let Some(first) = results.into_iter().next() {
                    combine_by_player.insert(player.id, first);
                }
            }
        }

        // Pre-fetch percentiles for all relevant position groups (~13 queries instead of 10*N)
        let position_groups: HashSet<String> = players
            .iter()
            .map(|p| RasScoringService::map_position(&p.position))
            .collect();
        let mut percentiles_by_position: HashMap<String, Vec<crate::models::CombinePercentile>> =
            HashMap::new();
        if let Some(ras) = self.player_eval_service.ras_service() {
            for pos in &position_groups {
                let percentiles = ras.fetch_percentiles_for_position(pos).await;
                percentiles_by_position.insert(pos.clone(), percentiles);
            }
        }

        let mut scores = Vec::new();

        for player in players {
            // Look up pre-fetched scouting report
            let scouting_report = match scouting_by_player.get(&player.id) {
                Some(report) => report,
                None => continue, // Skip players without scouting reports
            };

            let combine = combine_by_player.get(&player.id);
            let position_key = RasScoringService::map_position(&player.position);
            let percentiles = percentiles_by_position
                .get(&position_key)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            // Calculate BPA score with pre-loaded data (0 additional queries)
            let bpa_score = self.player_eval_service.calculate_bpa_score_preloaded(
                player,
                scouting_report,
                combine,
                percentiles,
            );

            // Calculate need score from pre-fetched needs (0 additional queries)
            let need_score =
                DraftStrategyService::calculate_need_score_from_needs(player, &team_needs);

            // Get position value multiplier (pure computation)
            let position_value = self
                .strategy_service
                .get_position_value(strategy, player.position);

            // Calculate final score
            let weighted_bpa = bpa_score * (strategy.bpa_weight as f64 / 100.0);
            let weighted_need = need_score * (strategy.need_weight as f64 / 100.0);
            let final_score = (weighted_bpa + weighted_need) * position_value;

            let rationale = Self::build_rationale(
                player,
                bpa_score,
                need_score,
                position_value,
                final_score,
                strategy,
            );

            scores.push(PlayerScore {
                player_id: player.id,
                bpa_score,
                need_score,
                position_value,
                final_score,
                rationale,
            });
        }

        // Sort by final score descending
        scores.sort_by(|a, b| {
            b.final_score
                .partial_cmp(&a.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(scores)
    }

    fn build_rationale(
        player: &Player,
        bpa_score: f64,
        need_score: f64,
        position_value: f64,
        final_score: f64,
        strategy: &crate::models::DraftStrategy,
    ) -> String {
        format!(
            "{} {} ({:?}): BPA={:.1}, Need={:.1}, PosValue={:.2}, Final={:.1} ({}% BPA / {}% Need)",
            player.first_name,
            player.last_name,
            player.position,
            bpa_score,
            need_score,
            position_value,
            final_score,
            strategy.bpa_weight,
            strategy.need_weight
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CombineResults, DraftStrategy, Position, ScoutingReport, TeamNeed};
    use crate::repositories::{
        CombineResultsRepository, DraftStrategyRepository, ScoutingReportRepository,
        TeamNeedRepository,
    };
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        ScoutingReportRepo {}

        #[async_trait::async_trait]
        impl ScoutingReportRepository for ScoutingReportRepo {
            async fn create(&self, report: &ScoutingReport) -> DomainResult<ScoutingReport>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<ScoutingReport>>;
            async fn find_by_player_id(&self, player_id: Uuid) -> DomainResult<Vec<ScoutingReport>>;
            async fn find_by_team_id(&self, team_id: Uuid) -> DomainResult<Vec<ScoutingReport>>;
            async fn find_by_team_and_player(&self, team_id: Uuid, player_id: Uuid) -> DomainResult<Option<ScoutingReport>>;
            async fn update(&self, report: &ScoutingReport) -> DomainResult<ScoutingReport>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
        }
    }

    mock! {
        CombineResultsRepo {}

        #[async_trait::async_trait]
        impl CombineResultsRepository for CombineResultsRepo {
            async fn create(&self, results: &CombineResults) -> DomainResult<CombineResults>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<CombineResults>>;
            async fn find_by_player_id(&self, player_id: Uuid) -> DomainResult<Vec<CombineResults>>;
            async fn find_by_player_and_year(&self, player_id: Uuid, year: i32) -> DomainResult<Option<CombineResults>>;
            async fn find_by_player_year_source(&self, player_id: Uuid, year: i32, source: &str) -> DomainResult<Option<CombineResults>>;
            async fn update(&self, results: &CombineResults) -> DomainResult<CombineResults>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
        }
    }

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

    fn create_test_player(id: Uuid, position: Position) -> Player {
        let mut player =
            Player::new("John".to_string(), "Doe".to_string(), position, 2026).unwrap();
        player.id = id;
        player
    }

    #[tokio::test]
    async fn test_bpa_heavy_picks_highest_grade() {
        // Given: 90% BPA strategy
        // And: QB grade 9.5 available (team doesn't need QB)
        // And: RB grade 7.0 available (team needs RB priority 1)
        // Then: QB should be selected (BPA dominates)

        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        let qb_id = Uuid::new_v4();
        let rb_id = Uuid::new_v4();

        let qb = create_test_player(qb_id, Position::QB);
        let rb = create_test_player(rb_id, Position::RB);

        let players = vec![qb.clone(), rb.clone()];

        // Mock repositories
        let mut scouting_mock = MockScoutingReportRepo::new();
        let mut combine_mock = MockCombineResultsRepo::new();
        let mut strategy_mock = MockDraftStrategyRepo::new();
        let mut need_mock = MockTeamNeedRepo::new();

        // Setup BPA-heavy strategy (90/10)
        let mut strategy = DraftStrategy::default_strategy(team_id, draft_id);
        strategy.bpa_weight = 90;
        strategy.need_weight = 10;

        strategy_mock
            .expect_find_by_team_and_draft()
            .returning(move |_, _| Ok(Some(strategy.clone())));

        // Scouting reports: now fetched by team (batch), not per-player
        let qb_report = ScoutingReport::new(qb_id, team_id, 9.5).unwrap();
        let rb_report = ScoutingReport::new(rb_id, team_id, 7.0).unwrap();
        scouting_mock
            .expect_find_by_team_id()
            .returning(move |_| Ok(vec![qb_report.clone(), rb_report.clone()]));

        // Combine results: fetched per player
        combine_mock
            .expect_find_by_player_id()
            .returning(|_| Ok(vec![]));

        // Team needs: fetched once
        let rb_need = TeamNeed::new(team_id, Position::RB, 1).unwrap();
        need_mock
            .expect_find_by_team_id()
            .returning(move |_| Ok(vec![rb_need.clone()]));

        // Setup services
        let player_eval = Arc::new(PlayerEvaluationService::new(
            Arc::new(scouting_mock),
            Arc::new(combine_mock),
        ));
        let strategy_svc = Arc::new(DraftStrategyService::new(
            Arc::new(strategy_mock),
            Arc::new(need_mock),
        ));
        let auto_pick = AutoPickService::new(player_eval, strategy_svc);

        let (selected_id, scores) = auto_pick
            .decide_pick(team_id, draft_id, &players)
            .await
            .unwrap();

        // QB should be selected (higher BPA score dominates)
        assert_eq!(selected_id, qb_id);
        assert_eq!(scores.len(), 2);
    }

    #[tokio::test]
    async fn test_need_heavy_picks_team_need() {
        // Given: 30% BPA, 70% need strategy
        // And: QB grade 9.5 available (no need)
        // And: RB grade 7.0 available (priority 1 need)
        // Then: RB should be selected (need dominates)

        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        let qb_id = Uuid::new_v4();
        let rb_id = Uuid::new_v4();

        let qb = create_test_player(qb_id, Position::QB);
        let rb = create_test_player(rb_id, Position::RB);

        let players = vec![qb.clone(), rb.clone()];

        // Mock repositories
        let mut scouting_mock = MockScoutingReportRepo::new();
        let mut combine_mock = MockCombineResultsRepo::new();
        let mut strategy_mock = MockDraftStrategyRepo::new();
        let mut need_mock = MockTeamNeedRepo::new();

        // Setup need-heavy strategy (30/70)
        let mut strategy = DraftStrategy::default_strategy(team_id, draft_id);
        strategy.bpa_weight = 30;
        strategy.need_weight = 70;

        strategy_mock
            .expect_find_by_team_and_draft()
            .returning(move |_, _| Ok(Some(strategy.clone())));

        // Scouting reports: batch fetched by team
        let qb_report = ScoutingReport::new(qb_id, team_id, 9.5).unwrap();
        let rb_report = ScoutingReport::new(rb_id, team_id, 7.0).unwrap();
        scouting_mock
            .expect_find_by_team_id()
            .returning(move |_| Ok(vec![qb_report.clone(), rb_report.clone()]));

        // Combine results: per player
        combine_mock
            .expect_find_by_player_id()
            .returning(|_| Ok(vec![]));

        // Team needs: fetched once
        let rb_need = TeamNeed::new(team_id, Position::RB, 1).unwrap();
        need_mock
            .expect_find_by_team_id()
            .returning(move |_| Ok(vec![rb_need.clone()]));

        // Setup services
        let player_eval = Arc::new(PlayerEvaluationService::new(
            Arc::new(scouting_mock),
            Arc::new(combine_mock),
        ));
        let strategy_svc = Arc::new(DraftStrategyService::new(
            Arc::new(strategy_mock),
            Arc::new(need_mock),
        ));
        let auto_pick = AutoPickService::new(player_eval, strategy_svc);

        let (selected_id, scores) = auto_pick
            .decide_pick(team_id, draft_id, &players)
            .await
            .unwrap();

        // RB should be selected (higher need score dominates)
        assert_eq!(selected_id, rb_id);
        assert_eq!(scores.len(), 2);
    }

    #[tokio::test]
    async fn test_position_value_affects_ranking() {
        // Given: Two players same BPA score
        // And: QB (value 1.5) vs RB (value 0.85)
        // Then: QB should rank higher due to position value

        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        let qb_id = Uuid::new_v4();
        let rb_id = Uuid::new_v4();

        let qb = create_test_player(qb_id, Position::QB);
        let rb = create_test_player(rb_id, Position::RB);

        let players = vec![qb.clone(), rb.clone()];

        // Mock repositories
        let mut scouting_mock = MockScoutingReportRepo::new();
        let mut combine_mock = MockCombineResultsRepo::new();
        let mut strategy_mock = MockDraftStrategyRepo::new();
        let mut need_mock = MockTeamNeedRepo::new();

        // Setup balanced strategy
        let strategy = DraftStrategy::default_strategy(team_id, draft_id);

        strategy_mock
            .expect_find_by_team_and_draft()
            .returning(move |_, _| Ok(Some(strategy.clone())));

        // Scouting reports: batch fetched by team
        let qb_report = ScoutingReport::new(qb_id, team_id, 8.0).unwrap();
        let rb_report = ScoutingReport::new(rb_id, team_id, 8.0).unwrap();
        scouting_mock
            .expect_find_by_team_id()
            .returning(move |_| Ok(vec![qb_report.clone(), rb_report.clone()]));

        // Combine results: per player
        combine_mock
            .expect_find_by_player_id()
            .returning(|_| Ok(vec![]));

        // No specific needs
        need_mock.expect_find_by_team_id().returning(|_| Ok(vec![]));

        // Setup services
        let player_eval = Arc::new(PlayerEvaluationService::new(
            Arc::new(scouting_mock),
            Arc::new(combine_mock),
        ));
        let strategy_svc = Arc::new(DraftStrategyService::new(
            Arc::new(strategy_mock),
            Arc::new(need_mock),
        ));
        let auto_pick = AutoPickService::new(player_eval, strategy_svc);

        let (selected_id, scores) = auto_pick
            .decide_pick(team_id, draft_id, &players)
            .await
            .unwrap();

        // QB should be selected (higher position value: 1.5 vs 0.85)
        assert_eq!(selected_id, qb_id);

        // Verify position values affected final scores
        let qb_score = scores.iter().find(|s| s.player_id == qb_id).unwrap();
        let rb_score = scores.iter().find(|s| s.player_id == rb_id).unwrap();
        assert!(qb_score.final_score > rb_score.final_score);
        assert_eq!(qb_score.position_value, 1.5);
        assert_eq!(rb_score.position_value, 0.85);
    }
}
