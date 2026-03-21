use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};
use crate::models::Player;
use crate::repositories::{FeldmanFreakRepository, ProspectRankingRepository};
use crate::services::{DraftStrategyService, PlayerEvaluationService, RasScoringService};

/// Result of player scoring with detailed breakdown
#[derive(Debug, Clone)]
pub struct PlayerScore {
    pub player_id: Uuid,
    pub bpa_score: f64,
    pub need_score: f64,
    /// Raw position factor from team strategy (e.g. 1.5 for QB, 0.85 for RB).
    /// Used to compute `pos_bonus = (position_factor - 1.0) * 5.0` which is added
    /// to the final score as a small preference signal, not a multiplier.
    pub position_factor: f64,
    pub ranking_score: f64,
    pub final_score: f64,
    pub rationale: String,
}

/// Service for making auto-pick decisions
pub struct AutoPickService {
    player_eval_service: Arc<PlayerEvaluationService>,
    strategy_service: Arc<DraftStrategyService>,
    ranking_repo: Option<Arc<dyn ProspectRankingRepository>>,
    feldman_freak_repo: Option<Arc<dyn FeldmanFreakRepository>>,
}

impl AutoPickService {
    pub fn new(
        player_eval_service: Arc<PlayerEvaluationService>,
        strategy_service: Arc<DraftStrategyService>,
    ) -> Self {
        Self {
            player_eval_service,
            strategy_service,
            ranking_repo: None,
            feldman_freak_repo: None,
        }
    }

    /// Wire in the prospect ranking repository for consensus ranking signal
    pub fn with_ranking_repo(mut self, repo: Arc<dyn ProspectRankingRepository>) -> Self {
        self.ranking_repo = Some(repo);
        self
    }

    /// Wire in the Feldman Freak repository for athleticism bonus
    pub fn with_feldman_freak_repo(mut self, repo: Arc<dyn FeldmanFreakRepository>) -> Self {
        self.feldman_freak_repo = Some(repo);
        self
    }

    /// Decide which player to pick based on team strategy
    /// Returns the selected player ID and the scoring breakdown
    /// Compute effective BPA/need weights for a given round.
    ///
    /// Early rounds are BPA-dominant (round 1 = ~90% BPA); later rounds shift
    /// to team needs (round 7 = ~10% BPA). The team's strategic preference
    /// provides a small additional offset so philosophy still matters.
    ///
    /// Formula: base = clamp(90 - (round-1) × 13, 10, 90)
    ///          offset = (strategy.bpa_weight - 60) × 0.15   (range: -9 to +6 for bpa_weight in [0,100])
    ///          effective_bpa = clamp(base + offset, 5, 95)
    fn effective_weights(round: i32, strategy: &crate::models::DraftStrategy) -> (f64, f64) {
        let base_bpa = (90.0 - (round as f64 - 1.0) * 13.0).clamp(10.0, 90.0);
        let strategy_offset = (strategy.bpa_weight as f64 - 60.0) * 0.15;
        let effective_bpa = (base_bpa + strategy_offset).clamp(5.0, 95.0);
        let effective_need = 100.0 - effective_bpa;
        (effective_bpa / 100.0, effective_need / 100.0)
    }

    pub async fn decide_pick(
        &self,
        team_id: Uuid,
        draft_id: Uuid,
        draft_year: i32,
        round: i32,
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
            .score_all_players(team_id, draft_year, round, available_players, &strategy)
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
    /// Pre-fetches team needs, scouting reports, combine results, percentile data,
    /// prospect rankings, and Feldman Freaks to avoid N+1 query patterns.
    async fn score_all_players(
        &self,
        team_id: Uuid,
        draft_year: i32,
        round: i32,
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

        // Pre-fetch prospect rankings for available players (1 query) → normalize to 0-100
        // Normalization: rank 1 → 100, rank 300 → 0 (exactly); average across sources when multiple exist.
        // Denominator 299 = (300 - 1) ensures rank 300 maps to exactly 0.0.
        let player_ids: Vec<Uuid> = players.iter().map(|p| p.id).collect();
        let ranking_scores: HashMap<Uuid, f64> = if let Some(ranking_repo) = &self.ranking_repo {
            match ranking_repo.find_for_players_with_source(&player_ids).await {
                Ok(all_rankings) => {
                    // Group ranks by player_id
                    let mut ranks_by_player: HashMap<Uuid, Vec<f64>> = HashMap::new();
                    for r in all_rankings {
                        ranks_by_player
                            .entry(r.player_id)
                            .or_default()
                            .push(r.rank as f64);
                    }
                    ranks_by_player
                        .into_iter()
                        .map(|(player_id, ranks)| {
                            let avg_rank = ranks.iter().sum::<f64>() / ranks.len() as f64;
                            let score =
                                (100.0 - ((avg_rank - 1.0) * 100.0 / 299.0)).clamp(0.0, 100.0);
                            (player_id, score)
                        })
                        .collect()
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch prospect rankings for BPA scoring: {}. All players will receive neutral ranking score (50.0).", e);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        // Pre-fetch Feldman Freaks for the current draft year (1 query) → HashSet for O(1) lookup
        let feldman_freak_ids: HashSet<Uuid> = if let Some(freak_repo) = &self.feldman_freak_repo {
            match freak_repo.find_by_year(draft_year).await {
                Ok(freaks) => freaks.into_iter().map(|f| f.player_id).collect(),
                Err(e) => {
                    tracing::warn!("Failed to fetch Feldman Freaks for BPA scoring: {}. No athleticism bonuses will be applied.", e);
                    HashSet::new()
                }
            }
        } else {
            HashSet::new()
        };

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

            let consensus_ranking_score = ranking_scores.get(&player.id).copied();
            let is_feldman_freak = feldman_freak_ids.contains(&player.id);

            // Calculate BPA score with pre-loaded data (0 additional queries)
            let bpa_score = self.player_eval_service.calculate_bpa_score_preloaded(
                player,
                scouting_report,
                combine,
                percentiles,
                consensus_ranking_score,
                is_feldman_freak,
            );

            // Calculate need score from pre-fetched needs (0 additional queries)
            let need_score =
                DraftStrategyService::calculate_need_score_from_needs(player, &team_needs);

            // Get position factor from team strategy (pure computation).
            // Used as additive bonus: pos_bonus = (position_factor - 1.0) * 5.0.
            // QB gets +2.5, RB gets -0.75. Additive (not multiplicative) so elite
            // non-QB prospects can still be selected early over mediocre QBs.
            let position_factor = self
                .strategy_service
                .get_position_value(strategy, player.position);

            let (bpa_w, need_w) = Self::effective_weights(round, strategy);
            let weighted_bpa = bpa_score * bpa_w;
            let weighted_need = need_score * need_w;
            let pos_bonus = (position_factor - 1.0) * 5.0;
            let final_score = weighted_bpa + weighted_need + pos_bonus;

            let ranking_score = consensus_ranking_score.unwrap_or(50.0);
            let rationale = Self::build_rationale(
                player,
                bpa_score,
                need_score,
                position_factor,
                ranking_score,
                is_feldman_freak,
                final_score,
                round,
                bpa_w,
            );

            scores.push(PlayerScore {
                player_id: player.id,
                bpa_score,
                need_score,
                position_factor,
                ranking_score,
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
        position_factor: f64,
        ranking_score: f64,
        is_feldman_freak: bool,
        final_score: f64,
        round: i32,
        bpa_w: f64,
    ) -> String {
        let freak_tag = if is_feldman_freak { " [Freak]" } else { "" };
        format!(
            "{} {} ({:?}){}: BPA={:.1}, Need={:.1}, Rank={:.1}, PosFactor={:.2}, Final={:.1} (R{}: {:.0}% BPA / {:.0}% Need)",
            player.first_name,
            player.last_name,
            player.position,
            freak_tag,
            bpa_score,
            need_score,
            ranking_score,
            position_factor,
            final_score,
            round,
            bpa_w * 100.0,
            (1.0 - bpa_w) * 100.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PlayerRankingWithSource;
    use crate::models::{CombineResults, DraftStrategy, Position, ScoutingReport, TeamNeed};
    use crate::repositories::{
        CombineResultsRepository, DraftStrategyRepository, FeldmanFreakRepository,
        ProspectRankingRepository, ScoutingReportRepository, TeamNeedRepository,
    };
    use chrono::NaiveDate;
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
            async fn find_all(&self) -> DomainResult<Vec<CombineResults>>;
            async fn count_by_year(&self, year: i32) -> DomainResult<i64>;
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

    mock! {
        ProspectRankingRepo {}

        #[async_trait::async_trait]
        impl ProspectRankingRepository for ProspectRankingRepo {
            async fn create_batch(&self, rankings: &[crate::models::ProspectRanking]) -> DomainResult<usize>;
            async fn find_by_player_with_source(&self, player_id: Uuid) -> DomainResult<Vec<PlayerRankingWithSource>>;
            async fn find_all_with_source(&self) -> DomainResult<Vec<PlayerRankingWithSource>>;
            async fn find_for_players_with_source(&self, player_ids: &[Uuid]) -> DomainResult<Vec<PlayerRankingWithSource>>;
            async fn find_by_player(&self, player_id: Uuid) -> DomainResult<Vec<crate::models::ProspectRanking>>;
            async fn find_by_source(&self, source_id: Uuid) -> DomainResult<Vec<crate::models::ProspectRanking>>;
            async fn delete_by_source(&self, source_id: Uuid) -> DomainResult<u64>;
        }
    }

    mock! {
        FeldmanFreakRepo {}

        #[async_trait::async_trait]
        impl FeldmanFreakRepository for FeldmanFreakRepo {
            async fn create(&self, freak: &crate::models::FeldmanFreak) -> DomainResult<crate::models::FeldmanFreak>;
            async fn find_by_player(&self, player_id: Uuid) -> DomainResult<Option<crate::models::FeldmanFreak>>;
            async fn find_by_year(&self, year: i32) -> DomainResult<Vec<crate::models::FeldmanFreak>>;
            async fn find_all(&self) -> DomainResult<Vec<crate::models::FeldmanFreak>>;
            async fn delete_by_year(&self, year: i32) -> DomainResult<u64>;
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
            .decide_pick(team_id, draft_id, 2026, 1, &players)
            .await
            .unwrap();

        // QB should be selected (higher BPA score dominates)
        assert_eq!(selected_id, qb_id);
        assert_eq!(scores.len(), 2);
    }

    #[tokio::test]
    async fn test_need_heavy_picks_team_need() {
        // Given: need-heavy strategy (30/70) in round 5
        // And: QB grade 9.5 available (team has no QB need)
        // And: RB grade 7.0 available (team has priority-1 RB need)
        // Then: RB should be selected — in round 5 effective weights are ~34% BPA / ~66% Need
        // (round-based formula dominates; the team's need-heavy preference reinforces it)

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
            .decide_pick(team_id, draft_id, 2026, 5, &players)
            .await
            .unwrap();

        // RB should be selected (need dominates in round 5)
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
            .decide_pick(team_id, draft_id, 2026, 1, &players)
            .await
            .unwrap();

        // QB should be selected (higher position value: 1.5 vs 0.85)
        assert_eq!(selected_id, qb_id);

        // Verify position factors affected final scores
        let qb_score = scores.iter().find(|s| s.player_id == qb_id).unwrap();
        let rb_score = scores.iter().find(|s| s.player_id == rb_id).unwrap();
        assert!(qb_score.final_score > rb_score.final_score);
        assert_eq!(qb_score.position_factor, 1.5);
        assert_eq!(rb_score.position_factor, 0.85);
    }

    #[test]
    fn test_effective_weights_round_progression() {
        use crate::models::DraftStrategy;

        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let strategy = DraftStrategy::default_strategy(team_id, draft_id); // 60/40

        let (bpa_r1, need_r1) = AutoPickService::effective_weights(1, &strategy);
        let (bpa_r4, _) = AutoPickService::effective_weights(4, &strategy);
        let (bpa_r7, need_r7) = AutoPickService::effective_weights(7, &strategy);

        // Round 1: BPA-dominant (≥ 85%)
        assert!(
            bpa_r1 >= 0.85,
            "R1 bpa_w should be ≥ 85%, got {:.1}%",
            bpa_r1 * 100.0
        );
        // Round 7: Need-dominant (≥ 85%)
        assert!(
            need_r7 >= 0.85,
            "R7 need_w should be ≥ 85%, got {:.1}%",
            need_r7 * 100.0
        );
        // Monotonically decreasing BPA weight
        assert!(bpa_r1 > bpa_r4, "BPA weight should decrease R1→R4");
        assert!(bpa_r4 > bpa_r7, "BPA weight should decrease R4→R7");
        // All weights sum to 1.0
        assert!((bpa_r1 + need_r1 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_effective_weights_bpa_heavy_team_stays_higher() {
        use crate::models::DraftStrategy;
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let mut bpa_strat = DraftStrategy::default_strategy(team_id, draft_id);
        bpa_strat.bpa_weight = 80;
        bpa_strat.need_weight = 20;
        let mut need_strat = DraftStrategy::default_strategy(team_id, draft_id);
        need_strat.bpa_weight = 40;
        need_strat.need_weight = 60;

        for round in 1..=7 {
            let (bpa_bpa, _) = AutoPickService::effective_weights(round, &bpa_strat);
            let (bpa_need, _) = AutoPickService::effective_weights(round, &need_strat);
            assert!(
                bpa_bpa > bpa_need,
                "BPA-focused team should have higher BPA weight at round {}: {:.1}% vs {:.1}%",
                round,
                bpa_bpa * 100.0,
                bpa_need * 100.0
            );
        }
    }

    #[tokio::test]
    async fn test_rank1_player_beats_rank300_with_equal_scouting() {
        // Given: two QBs with identical scouting grades
        // And: player A is ranked #1 overall, player B is ranked #300
        // Then: player A should be selected (consensus ranking signal distinguishes them)

        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        let top_id = Uuid::new_v4();
        let late_id = Uuid::new_v4();

        let top_qb = create_test_player(top_id, Position::QB);
        let late_qb = create_test_player(late_id, Position::QB);

        let players = vec![top_qb.clone(), late_qb.clone()];

        let mut scouting_mock = MockScoutingReportRepo::new();
        let mut combine_mock = MockCombineResultsRepo::new();
        let mut strategy_mock = MockDraftStrategyRepo::new();
        let mut need_mock = MockTeamNeedRepo::new();
        let mut ranking_mock = MockProspectRankingRepo::new();
        let mut freak_mock = MockFeldmanFreakRepo::new();

        let strategy = DraftStrategy::default_strategy(team_id, draft_id);
        strategy_mock
            .expect_find_by_team_and_draft()
            .returning(move |_, _| Ok(Some(strategy.clone())));

        // Both players have identical scouting grade 8.0
        let top_report = ScoutingReport::new(top_id, team_id, 8.0).unwrap();
        let late_report = ScoutingReport::new(late_id, team_id, 8.0).unwrap();
        scouting_mock
            .expect_find_by_team_id()
            .returning(move |_| Ok(vec![top_report.clone(), late_report.clone()]));

        combine_mock
            .expect_find_by_player_id()
            .returning(|_| Ok(vec![]));

        need_mock.expect_find_by_team_id().returning(|_| Ok(vec![]));

        // top_id ranked #1, late_id ranked #300
        let scraped = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let source_id = Uuid::new_v4();
        let rankings = vec![
            PlayerRankingWithSource {
                player_id: top_id,
                source_name: "TestSource".to_string(),
                source_id,
                rank: 1,
                scraped_at: scraped,
            },
            PlayerRankingWithSource {
                player_id: late_id,
                source_name: "TestSource".to_string(),
                source_id,
                rank: 300,
                scraped_at: scraped,
            },
        ];
        ranking_mock
            .expect_find_for_players_with_source()
            .returning(move |_| Ok(rankings.clone()));

        freak_mock.expect_find_by_year().returning(|_| Ok(vec![]));

        let player_eval = Arc::new(PlayerEvaluationService::new(
            Arc::new(scouting_mock),
            Arc::new(combine_mock),
        ));
        let strategy_svc = Arc::new(DraftStrategyService::new(
            Arc::new(strategy_mock),
            Arc::new(need_mock),
        ));
        let auto_pick = AutoPickService::new(player_eval, strategy_svc)
            .with_ranking_repo(Arc::new(ranking_mock))
            .with_feldman_freak_repo(Arc::new(freak_mock));

        let (selected_id, scores) = auto_pick
            .decide_pick(team_id, draft_id, 2026, 1, &players)
            .await
            .unwrap();

        // Rank-1 player should win despite identical scouting grades
        assert_eq!(selected_id, top_id);

        let top_score = scores.iter().find(|s| s.player_id == top_id).unwrap();
        let late_score = scores.iter().find(|s| s.player_id == late_id).unwrap();
        assert!(
            top_score.ranking_score > late_score.ranking_score,
            "rank-1 should have higher ranking_score than rank-300: {} vs {}",
            top_score.ranking_score,
            late_score.ranking_score
        );
        assert!(
            top_score.bpa_score > late_score.bpa_score,
            "rank-1 should have higher BPA than rank-300 with equal scouting: {} vs {}",
            top_score.bpa_score,
            late_score.bpa_score
        );
    }

    #[tokio::test]
    async fn test_pos_bonus_does_not_override_elite_bpa() {
        // Given: rank-1 LB (elite, BPA ~80) vs rank-100 QB (mediocre, BPA ~60)
        // Both have no team needs.
        // Then: LB should be selected — position bonus (+2.5 for QB) must NOT override
        // a 20+ point BPA advantage. This verifies the additive formula is correct.

        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let lb_id = Uuid::new_v4();
        let qb_id = Uuid::new_v4();

        let lb = create_test_player(lb_id, Position::LB);
        let qb = create_test_player(qb_id, Position::QB);
        let players = vec![lb.clone(), qb.clone()];

        let mut scouting_mock = MockScoutingReportRepo::new();
        let mut combine_mock = MockCombineResultsRepo::new();
        let mut strategy_mock = MockDraftStrategyRepo::new();
        let mut need_mock = MockTeamNeedRepo::new();
        let mut ranking_mock = MockProspectRankingRepo::new();
        let mut freak_mock = MockFeldmanFreakRepo::new();

        let strategy = DraftStrategy::default_strategy(team_id, draft_id);
        strategy_mock
            .expect_find_by_team_and_draft()
            .returning(move |_, _| Ok(Some(strategy.clone())));

        // LB grade 9.5 (elite), QB grade 7.0 (mediocre)
        let lb_report = ScoutingReport::new(lb_id, team_id, 9.5).unwrap();
        let qb_report = ScoutingReport::new(qb_id, team_id, 7.0).unwrap();
        scouting_mock
            .expect_find_by_team_id()
            .returning(move |_| Ok(vec![lb_report.clone(), qb_report.clone()]));

        combine_mock
            .expect_find_by_player_id()
            .returning(|_| Ok(vec![]));

        need_mock.expect_find_by_team_id().returning(|_| Ok(vec![]));

        let scraped = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let source_id = Uuid::new_v4();
        let rankings = vec![
            PlayerRankingWithSource {
                player_id: lb_id,
                source_name: "TestSource".to_string(),
                source_id,
                rank: 1,
                scraped_at: scraped,
            },
            PlayerRankingWithSource {
                player_id: qb_id,
                source_name: "TestSource".to_string(),
                source_id,
                rank: 100,
                scraped_at: scraped,
            },
        ];
        ranking_mock
            .expect_find_for_players_with_source()
            .returning(move |_| Ok(rankings.clone()));

        freak_mock.expect_find_by_year().returning(|_| Ok(vec![]));

        let player_eval = Arc::new(PlayerEvaluationService::new(
            Arc::new(scouting_mock),
            Arc::new(combine_mock),
        ));
        let strategy_svc = Arc::new(DraftStrategyService::new(
            Arc::new(strategy_mock),
            Arc::new(need_mock),
        ));
        let auto_pick = AutoPickService::new(player_eval, strategy_svc)
            .with_ranking_repo(Arc::new(ranking_mock))
            .with_feldman_freak_repo(Arc::new(freak_mock));

        let (selected_id, scores) = auto_pick
            .decide_pick(team_id, draft_id, 2026, 1, &players)
            .await
            .unwrap();

        // Elite LB (rank 1, grade 9.5) must beat mediocre QB (rank 100, grade 7.0)
        // even though QB has a position bonus. BPA signal must dominate.
        assert_eq!(
            selected_id, lb_id,
            "rank-1 elite LB should beat rank-100 mediocre QB despite QB position bonus"
        );

        let lb_score = scores.iter().find(|s| s.player_id == lb_id).unwrap();
        let qb_score = scores.iter().find(|s| s.player_id == qb_id).unwrap();
        assert!(
            lb_score.final_score > qb_score.final_score,
            "LB final_score {} should exceed QB final_score {}",
            lb_score.final_score,
            qb_score.final_score
        );
    }

    #[tokio::test]
    async fn test_auto_pick_works_gracefully_when_ranking_repo_fails() {
        // When the ranking repository returns an error, auto-pick should still
        // complete successfully using neutral ranking scores (50.0) for all players.

        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let qb_id = Uuid::new_v4();
        let rb_id = Uuid::new_v4();

        let qb = create_test_player(qb_id, Position::QB);
        let rb = create_test_player(rb_id, Position::RB);
        let players = vec![qb.clone(), rb.clone()];

        let mut scouting_mock = MockScoutingReportRepo::new();
        let mut combine_mock = MockCombineResultsRepo::new();
        let mut strategy_mock = MockDraftStrategyRepo::new();
        let mut need_mock = MockTeamNeedRepo::new();
        let mut ranking_mock = MockProspectRankingRepo::new();
        let mut freak_mock = MockFeldmanFreakRepo::new();

        let strategy = DraftStrategy::default_strategy(team_id, draft_id);
        strategy_mock
            .expect_find_by_team_and_draft()
            .returning(move |_, _| Ok(Some(strategy.clone())));

        let qb_report = ScoutingReport::new(qb_id, team_id, 9.0).unwrap();
        let rb_report = ScoutingReport::new(rb_id, team_id, 7.0).unwrap();
        scouting_mock
            .expect_find_by_team_id()
            .returning(move |_| Ok(vec![qb_report.clone(), rb_report.clone()]));

        combine_mock
            .expect_find_by_player_id()
            .returning(|_| Ok(vec![]));

        need_mock.expect_find_by_team_id().returning(|_| Ok(vec![]));

        // Ranking repo fails — should be handled gracefully
        ranking_mock
            .expect_find_for_players_with_source()
            .returning(|_| Err(DomainError::ValidationError("DB error".to_string())));

        freak_mock.expect_find_by_year().returning(|_| Ok(vec![]));

        let player_eval = Arc::new(PlayerEvaluationService::new(
            Arc::new(scouting_mock),
            Arc::new(combine_mock),
        ));
        let strategy_svc = Arc::new(DraftStrategyService::new(
            Arc::new(strategy_mock),
            Arc::new(need_mock),
        ));
        let auto_pick = AutoPickService::new(player_eval, strategy_svc)
            .with_ranking_repo(Arc::new(ranking_mock))
            .with_feldman_freak_repo(Arc::new(freak_mock));

        // Should not panic or return an error — graceful degradation
        let result = auto_pick
            .decide_pick(team_id, draft_id, 2026, 1, &players)
            .await;

        assert!(
            result.is_ok(),
            "auto-pick should succeed even when ranking repo fails"
        );
        let (selected_id, _) = result.unwrap();
        // QB has higher scouting grade so should still win (neutral ranking score = 50 for both)
        assert_eq!(selected_id, qb_id);
    }
}
