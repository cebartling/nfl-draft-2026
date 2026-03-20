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
    pub position_value: f64,
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
    /// Pre-fetches team needs, scouting reports, combine results, percentile data,
    /// prospect rankings, and Feldman Freaks to avoid N+1 query patterns.
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

        // Pre-fetch prospect rankings (1 query) → normalize to 0-100 consensus score per player
        // Normalization: rank 1 → 100, rank 300+ → 0; average across sources when multiple exist.
        let ranking_scores: HashMap<Uuid, f64> =
            if let Some(ranking_repo) = &self.ranking_repo {
                match ranking_repo.find_all_with_source().await {
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
                                let avg_rank =
                                    ranks.iter().sum::<f64>() / ranks.len() as f64;
                                let score = (100.0
                                    - ((avg_rank - 1.0) * 100.0 / 300.0))
                                    .clamp(0.0, 100.0);
                                (player_id, score)
                            })
                            .collect()
                    }
                    Err(_) => HashMap::new(),
                }
            } else {
                HashMap::new()
            };

        // Pre-fetch Feldman Freaks for the current draft year (1 query) → HashSet for O(1) lookup
        let feldman_freak_ids: HashSet<Uuid> =
            if let Some(freak_repo) = &self.feldman_freak_repo {
                match freak_repo.find_by_year(2026).await {
                    Ok(freaks) => freaks.into_iter().map(|f| f.player_id).collect(),
                    Err(_) => HashSet::new(),
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

            // Get position value multiplier (pure computation)
            let position_value = self
                .strategy_service
                .get_position_value(strategy, player.position);

            // Calculate final score
            let weighted_bpa = bpa_score * (strategy.bpa_weight as f64 / 100.0);
            let weighted_need = need_score * (strategy.need_weight as f64 / 100.0);
            let final_score = (weighted_bpa + weighted_need) * position_value;

            let ranking_score = consensus_ranking_score.unwrap_or(50.0);
            let rationale = Self::build_rationale(
                player,
                bpa_score,
                need_score,
                position_value,
                ranking_score,
                is_feldman_freak,
                final_score,
                strategy,
            );

            scores.push(PlayerScore {
                player_id: player.id,
                bpa_score,
                need_score,
                position_value,
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
        position_value: f64,
        ranking_score: f64,
        is_feldman_freak: bool,
        final_score: f64,
        strategy: &crate::models::DraftStrategy,
    ) -> String {
        let freak_tag = if is_feldman_freak { " [Freak]" } else { "" };
        format!(
            "{} {} ({:?}){}: BPA={:.1}, Need={:.1}, Rank={:.1}, PosValue={:.2}, Final={:.1} ({}% BPA / {}% Need)",
            player.first_name,
            player.last_name,
            player.position,
            freak_tag,
            bpa_score,
            need_score,
            ranking_score,
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
    use crate::models::PlayerRankingWithSource;
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
            .expect_find_all_with_source()
            .returning(move || Ok(rankings.clone()));

        freak_mock
            .expect_find_by_year()
            .returning(|_| Ok(vec![]));

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
            .decide_pick(team_id, draft_id, &players)
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
}
