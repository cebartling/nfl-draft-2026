use std::sync::Arc;

use domain::models::{
    CombineResults, Conference, Division, Draft, DraftStrategy, FitGrade, Player, Position,
    ScoutingReport, Team, TeamNeed,
};
use domain::repositories::{
    CombinePercentileRepository, CombineResultsRepository, DraftPickRepository, DraftRepository,
    DraftStrategyRepository, PlayerRepository, ScoutingReportRepository, TeamNeedRepository,
    TeamRepository,
};
use domain::services::{
    AutoPickService, DraftEngine, DraftStrategyService, PlayerEvaluationService, RasScoringService,
};

use db::repositories::{
    SqlxCombinePercentileRepository, SqlxCombineResultsRepository, SqlxDraftPickRepository,
    SqlxDraftRepository, SqlxDraftStrategyRepository, SqlxPlayerRepository,
    SqlxScoutingReportRepository, SqlxTeamNeedRepository, SqlxTeamRepository,
};

/// Integration tests for auto-pick functionality with real database
///
/// These tests verify the complete auto-pick flow:
/// 1. Database setup with teams, players, scouting reports, needs
/// 2. Auto-pick decision execution
/// 3. Database state verification
///
/// Run with: cargo test -p api --test auto_pick_integration -- --test-threads=1
mod common;

struct TestContext {
    pool: sqlx::PgPool,
    draft_engine: DraftEngine,
    team_repo: Arc<SqlxTeamRepository>,
    player_repo: Arc<SqlxPlayerRepository>,
    draft_repo: Arc<SqlxDraftRepository>,
    pick_repo: Arc<SqlxDraftPickRepository>,
    scouting_repo: Arc<SqlxScoutingReportRepository>,
    combine_repo: Arc<SqlxCombineResultsRepository>,
    percentile_repo: Arc<SqlxCombinePercentileRepository>,
    need_repo: Arc<SqlxTeamNeedRepository>,
    strategy_repo: Arc<SqlxDraftStrategyRepository>,
}

impl TestContext {
    async fn new() -> Self {
        let pool = common::setup_test_pool().await;
        common::cleanup_database(&pool).await;

        let team_repo = Arc::new(SqlxTeamRepository::new(pool.clone()));
        let player_repo = Arc::new(SqlxPlayerRepository::new(pool.clone()));
        let draft_repo = Arc::new(SqlxDraftRepository::new(pool.clone()));
        let pick_repo = Arc::new(SqlxDraftPickRepository::new(pool.clone()));
        let scouting_repo = Arc::new(SqlxScoutingReportRepository::new(pool.clone()));
        let combine_repo = Arc::new(SqlxCombineResultsRepository::new(pool.clone()));
        let percentile_repo = Arc::new(SqlxCombinePercentileRepository::new(pool.clone()));
        let need_repo = Arc::new(SqlxTeamNeedRepository::new(pool.clone()));
        let strategy_repo = Arc::new(SqlxDraftStrategyRepository::new(pool.clone()));

        // Create services with RAS integration
        let ras_service = Arc::new(RasScoringService::new(
            percentile_repo.clone() as Arc<dyn CombinePercentileRepository>,
        ));

        let player_eval_service = Arc::new(
            PlayerEvaluationService::new(
                scouting_repo.clone() as Arc<dyn ScoutingReportRepository>,
                combine_repo.clone() as Arc<dyn CombineResultsRepository>,
            )
            .with_ras_service(ras_service),
        );

        let strategy_service = Arc::new(DraftStrategyService::new(
            strategy_repo.clone() as Arc<dyn DraftStrategyRepository>,
            need_repo.clone() as Arc<dyn TeamNeedRepository>,
        ));

        let auto_pick_service =
            Arc::new(AutoPickService::new(player_eval_service, strategy_service));

        let draft_engine = DraftEngine::new(
            draft_repo.clone() as Arc<dyn DraftRepository>,
            pick_repo.clone() as Arc<dyn DraftPickRepository>,
            team_repo.clone() as Arc<dyn TeamRepository>,
            player_repo.clone() as Arc<dyn PlayerRepository>,
        )
        .with_auto_pick(auto_pick_service);

        Self {
            pool,
            draft_engine,
            team_repo,
            player_repo,
            draft_repo,
            pick_repo,
            scouting_repo,
            combine_repo,
            percentile_repo,
            need_repo,
            strategy_repo,
        }
    }

    async fn cleanup(&self) {
        common::cleanup_database(&self.pool).await;
    }

    async fn create_team(&self, abbr: &str) -> Team {
        let team = Team::new(
            format!("Team {}", abbr),
            abbr.to_string(),
            "Test City".to_string(),
            Conference::AFC,
            Division::AFCEast,
        )
        .unwrap();
        self.team_repo.create(&team).await.unwrap()
    }

    async fn create_player(&self, name: &str, position: Position) -> Player {
        let player = Player::new(name.to_string(), "Test".to_string(), position, 2026).unwrap();
        self.player_repo.create(&player).await.unwrap()
    }

    async fn create_scouting_report(
        &self,
        player_id: uuid::Uuid,
        team_id: uuid::Uuid,
        grade: f64,
        fit_grade: Option<FitGrade>,
        injury_concern: bool,
        character_concern: bool,
    ) -> ScoutingReport {
        let mut report = ScoutingReport::new(player_id, team_id, grade).unwrap();
        report.fit_grade = fit_grade;
        report.injury_concern = injury_concern;
        report.character_concern = character_concern;
        self.scouting_repo.create(&report).await.unwrap()
    }

    async fn create_combine_results(&self, player_id: uuid::Uuid) -> CombineResults {
        let results = CombineResults::new(player_id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.5)
            .unwrap()
            .with_vertical_jump(36.0)
            .unwrap()
            .with_bench_press(20)
            .unwrap();
        self.combine_repo.create(&results).await.unwrap()
    }

    /// Seed WR percentile data for RAS scoring
    async fn seed_wr_percentiles(&self) {
        use domain::models::{CombinePercentile, Measurement};

        let measurements = vec![
            // All breakpoints in ascending order
            ("forty_yard_dash", 4.24, 4.33, 4.37, 4.40, 4.42, 4.45, 4.48, 4.50, 4.54, 4.58, 4.80),
            ("bench_press", 5.0, 9.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 18.0, 20.0, 28.0),
            ("vertical_jump", 28.0, 31.0, 33.0, 34.0, 35.0, 37.0, 38.0, 39.0, 40.0, 42.0, 46.0),
            ("broad_jump", 108.0, 114.0, 117.0, 119.0, 121.0, 122.0, 124.0, 125.0, 127.0, 130.0, 138.0),
            ("three_cone_drill", 6.40, 6.55, 6.63, 6.70, 6.75, 6.80, 6.87, 6.92, 6.98, 7.05, 7.40),
            ("twenty_yard_shuttle", 3.86, 3.96, 4.00, 4.04, 4.07, 4.10, 4.14, 4.17, 4.20, 4.25, 4.50),
            ("height", 67.0, 69.0, 70.0, 70.5, 71.0, 71.5, 72.0, 72.5, 73.0, 74.0, 77.0),
            ("weight", 165.0, 175.0, 180.0, 183.0, 185.0, 188.0, 191.0, 194.0, 198.0, 202.0, 215.0),
            ("ten_yard_split", 1.42, 1.47, 1.49, 1.50, 1.51, 1.52, 1.54, 1.55, 1.56, 1.58, 1.70),
            ("twenty_yard_split", 2.40, 2.47, 2.50, 2.52, 2.54, 2.56, 2.58, 2.60, 2.63, 2.66, 2.80),
        ];

        for (m, min, p10, p20, p30, p40, p50, p60, p70, p80, p90, max) in measurements {
            let measurement: Measurement = m.parse().unwrap();
            let p = CombinePercentile::new("WR".to_string(), measurement)
                .unwrap()
                .with_percentiles(300, min, p10, p20, p30, p40, p50, p60, p70, p80, p90, max)
                .unwrap();
            self.percentile_repo.upsert(&p).await.unwrap();
        }
    }

    async fn create_team_need(
        &self,
        team_id: uuid::Uuid,
        position: Position,
        priority: i32,
    ) -> TeamNeed {
        let need = TeamNeed::new(team_id, position, priority).unwrap();
        self.need_repo.create(&need).await.unwrap()
    }

    async fn create_draft_strategy(
        &self,
        team_id: uuid::Uuid,
        draft_id: uuid::Uuid,
        bpa_weight: i32,
        need_weight: i32,
    ) -> DraftStrategy {
        let strategy =
            DraftStrategy::new(team_id, draft_id, bpa_weight, need_weight, None, 5).unwrap();
        self.strategy_repo.create(&strategy).await.unwrap()
    }
}

#[tokio::test]
async fn test_auto_pick_bpa_heavy_strategy() {
    // Given: Team with 90% BPA strategy
    // And: High-grade QB available (not a team need)
    // And: Lower-grade RB available (priority 1 need)
    // Then: QB should be selected (BPA dominates)

    let ctx = TestContext::new().await;

    // Create team and draft
    let team = ctx.create_team("BPA").await;
    let draft = Draft::new("Test Draft".to_string(), 2026, 7, 1).unwrap();
    let draft = ctx.draft_repo.create(&draft).await.unwrap();

    // Initialize picks
    ctx.draft_engine.initialize_picks(draft.id).await.unwrap();

    // Get first pick (belongs to first team)
    let pick = ctx
        .pick_repo
        .find_next_pick(draft.id)
        .await
        .unwrap()
        .unwrap();

    // Create players
    let qb = ctx.create_player("Elite QB", Position::QB).await;
    let rb = ctx.create_player("Good RB", Position::RB).await;

    // QB has excellent grade (9.5)
    ctx.create_scouting_report(qb.id, team.id, 9.5, Some(FitGrade::A), false, false)
        .await;
    ctx.create_combine_results(qb.id).await;

    // RB has good grade (7.0) but lower
    ctx.create_scouting_report(rb.id, team.id, 7.0, Some(FitGrade::B), false, false)
        .await;
    ctx.create_combine_results(rb.id).await;

    // Team needs RB (priority 1), not QB
    ctx.create_team_need(team.id, Position::RB, 1).await;

    // Set BPA-heavy strategy (90/10)
    ctx.create_draft_strategy(team.id, draft.id, 90, 10).await;

    // Execute auto-pick
    let updated_pick = ctx.draft_engine.execute_auto_pick(pick.id).await.unwrap();

    // Verify QB was selected (BPA wins)
    assert_eq!(updated_pick.player_id, Some(qb.id));
    assert!(updated_pick.picked_at.is_some());

    // Verify in database
    let db_pick = ctx.pick_repo.find_by_id(pick.id).await.unwrap().unwrap();
    assert_eq!(db_pick.player_id, Some(qb.id));

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_auto_pick_need_heavy_strategy() {
    // Given: Team with 30% BPA / 70% Need strategy
    // And: High-grade QB available (not a need)
    // And: Lower-grade RB available (priority 1 need)
    // Then: RB should be selected (need dominates)

    let ctx = TestContext::new().await;

    // Create team and draft
    let team = ctx.create_team("NED").await;
    let draft = Draft::new("Test Draft".to_string(), 2026, 7, 1).unwrap();
    let draft = ctx.draft_repo.create(&draft).await.unwrap();

    // Initialize picks
    ctx.draft_engine.initialize_picks(draft.id).await.unwrap();

    let pick = ctx
        .pick_repo
        .find_next_pick(draft.id)
        .await
        .unwrap()
        .unwrap();

    // Create players
    let qb = ctx.create_player("Elite QB", Position::QB).await;
    let rb = ctx.create_player("Good RB", Position::RB).await;

    // QB has excellent grade (9.5)
    ctx.create_scouting_report(qb.id, team.id, 9.5, Some(FitGrade::A), false, false)
        .await;

    // RB has good grade (7.5)
    ctx.create_scouting_report(rb.id, team.id, 7.5, Some(FitGrade::A), false, false)
        .await;

    // Team needs RB (priority 1)
    ctx.create_team_need(team.id, Position::RB, 1).await;

    // Set need-heavy strategy (30/70)
    ctx.create_draft_strategy(team.id, draft.id, 30, 70).await;

    // Execute auto-pick
    let updated_pick = ctx.draft_engine.execute_auto_pick(pick.id).await.unwrap();

    // Verify RB was selected (need wins)
    assert_eq!(updated_pick.player_id, Some(rb.id));

    // Verify in database
    let db_pick = ctx.pick_repo.find_by_id(pick.id).await.unwrap().unwrap();
    assert_eq!(db_pick.player_id, Some(rb.id));

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_auto_pick_position_value_matters() {
    // Given: Two players with similar BPA scores
    // And: QB (position value 1.5) vs WR (position value 1.0)
    // And: Balanced strategy (60/40)
    // And: No specific team needs
    // Then: QB should be selected due to higher position value

    let ctx = TestContext::new().await;

    let team = ctx.create_team("POS").await;
    let draft = Draft::new("Test Draft".to_string(), 2026, 7, 1).unwrap();
    let draft = ctx.draft_repo.create(&draft).await.unwrap();

    ctx.draft_engine.initialize_picks(draft.id).await.unwrap();

    let pick = ctx
        .pick_repo
        .find_next_pick(draft.id)
        .await
        .unwrap()
        .unwrap();

    // Create players with similar grades
    let qb = ctx.create_player("Good QB", Position::QB).await;
    let wr = ctx.create_player("Good WR", Position::WR).await;

    // Both have same grade (8.0)
    ctx.create_scouting_report(qb.id, team.id, 8.0, Some(FitGrade::B), false, false)
        .await;
    ctx.create_scouting_report(wr.id, team.id, 8.0, Some(FitGrade::B), false, false)
        .await;

    // No specific needs
    // Default strategy (60/40) will be used

    // Execute auto-pick
    let updated_pick = ctx.draft_engine.execute_auto_pick(pick.id).await.unwrap();

    // Verify QB was selected (higher position value: 1.5 vs 1.0)
    assert_eq!(updated_pick.player_id, Some(qb.id));

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_auto_pick_concern_penalties() {
    // Given: Two players with same base grade
    // And: One has injury and character concerns
    // And: Other has no concerns
    // Then: Player without concerns should be selected

    let ctx = TestContext::new().await;

    let team = ctx.create_team("CLN").await;
    let draft = Draft::new("Test Draft".to_string(), 2026, 7, 1).unwrap();
    let draft = ctx.draft_repo.create(&draft).await.unwrap();

    ctx.draft_engine.initialize_picks(draft.id).await.unwrap();

    let pick = ctx
        .pick_repo
        .find_next_pick(draft.id)
        .await
        .unwrap()
        .unwrap();

    // Create two QBs with same grade
    let clean_qb = ctx.create_player("Clean QB", Position::QB).await;
    let risky_qb = ctx.create_player("Risky QB", Position::QB).await;

    // Both have same base grade (8.5)
    ctx.create_scouting_report(clean_qb.id, team.id, 8.5, Some(FitGrade::A), false, false)
        .await;

    // Risky QB has injury and character concerns (10 point penalty total)
    ctx.create_scouting_report(risky_qb.id, team.id, 8.5, Some(FitGrade::A), true, true)
        .await;

    // Execute auto-pick
    let updated_pick = ctx.draft_engine.execute_auto_pick(pick.id).await.unwrap();

    // Verify clean QB was selected
    assert_eq!(updated_pick.player_id, Some(clean_qb.id));

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_auto_pick_with_combine_data() {
    // Given: Two players with same scouting grade
    // And: One has excellent combine results
    // And: Other has no combine data
    // Then: Player with better combine should score higher

    let ctx = TestContext::new().await;

    let team = ctx.create_team("CMB").await;
    let draft = Draft::new("Test Draft".to_string(), 2026, 7, 1).unwrap();
    let draft = ctx.draft_repo.create(&draft).await.unwrap();

    ctx.draft_engine.initialize_picks(draft.id).await.unwrap();

    let pick = ctx
        .pick_repo
        .find_next_pick(draft.id)
        .await
        .unwrap()
        .unwrap();

    // Create two WRs
    let athletic_wr = ctx.create_player("Athletic WR", Position::WR).await;
    let unknown_wr = ctx.create_player("Unknown WR", Position::WR).await;

    // Both have same scouting grade
    ctx.create_scouting_report(
        athletic_wr.id,
        team.id,
        7.5,
        Some(FitGrade::B),
        false,
        false,
    )
    .await;
    ctx.create_scouting_report(unknown_wr.id, team.id, 7.5, Some(FitGrade::B), false, false)
        .await;

    // Athletic WR has excellent combine results
    let combine = CombineResults::new(athletic_wr.id, 2026)
        .unwrap()
        .with_forty_yard_dash(4.3)
        .unwrap() // Excellent
        .with_vertical_jump(42.0)
        .unwrap() // Excellent
        .with_three_cone_drill(6.5)
        .unwrap(); // Excellent
    ctx.combine_repo.create(&combine).await.unwrap();

    // Unknown WR has no combine data (will use default 50.0)

    // Execute auto-pick
    let updated_pick = ctx.draft_engine.execute_auto_pick(pick.id).await.unwrap();

    // Verify athletic WR was selected (combine boosts BPA score)
    assert_eq!(updated_pick.player_id, Some(athletic_wr.id));

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_auto_pick_no_available_players() {
    // Given: A pick to be made
    // And: No eligible players in the draft year
    // Then: Should return error

    let ctx = TestContext::new().await;

    let _team = ctx.create_team("NOP").await;
    let draft = Draft::new("Test Draft".to_string(), 2026, 7, 1).unwrap();
    let draft = ctx.draft_repo.create(&draft).await.unwrap();

    ctx.draft_engine.initialize_picks(draft.id).await.unwrap();

    let pick = ctx
        .pick_repo
        .find_next_pick(draft.id)
        .await
        .unwrap()
        .unwrap();

    // No players created for 2026

    // Execute auto-pick should fail
    let result = ctx.draft_engine.execute_auto_pick(pick.id).await;
    assert!(result.is_err());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_auto_pick_without_scouting_reports() {
    // Given: Players exist
    // And: No scouting reports for the team
    // Then: Should return error (can't evaluate without scouting)

    let ctx = TestContext::new().await;

    let _team = ctx.create_team("NSC").await;
    let draft = Draft::new("Test Draft".to_string(), 2026, 7, 1).unwrap();
    let draft = ctx.draft_repo.create(&draft).await.unwrap();

    ctx.draft_engine.initialize_picks(draft.id).await.unwrap();

    let pick = ctx
        .pick_repo
        .find_next_pick(draft.id)
        .await
        .unwrap()
        .unwrap();

    // Create player but no scouting report
    ctx.create_player("Unscouted QB", Position::QB).await;

    // Execute auto-pick should fail (no scouting reports)
    let result = ctx.draft_engine.execute_auto_pick(pick.id).await;
    assert!(result.is_err());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_auto_pick_uses_default_strategy_if_none_exists() {
    // Given: No explicit strategy configured for team
    // Then: Should use default strategy (60/40 BPA/Need)

    let ctx = TestContext::new().await;

    let team = ctx.create_team("DEF").await;
    let draft = Draft::new("Test Draft".to_string(), 2026, 7, 1).unwrap();
    let draft = ctx.draft_repo.create(&draft).await.unwrap();

    ctx.draft_engine.initialize_picks(draft.id).await.unwrap();

    let pick = ctx
        .pick_repo
        .find_next_pick(draft.id)
        .await
        .unwrap()
        .unwrap();

    // Create players
    let qb = ctx.create_player("QB", Position::QB).await;
    ctx.create_scouting_report(qb.id, team.id, 8.0, Some(FitGrade::A), false, false)
        .await;

    // No strategy explicitly set - should use default

    // Execute auto-pick should succeed with default strategy
    let result = ctx.draft_engine.execute_auto_pick(pick.id).await;
    assert!(result.is_ok());

    // Verify default strategy was created in database
    let strategy = ctx
        .strategy_repo
        .find_by_team_and_draft(team.id, draft.id)
        .await
        .unwrap();
    assert!(strategy.is_some());
    let strategy = strategy.unwrap();
    assert_eq!(strategy.bpa_weight, 60);
    assert_eq!(strategy.need_weight, 40);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_auto_pick_considers_ras_with_percentiles() {
    // Given: Two WRs with same scouting grade
    // And: Percentile data is seeded for WR position
    // And: One WR has elite combine numbers, the other has average
    // Then: Elite WR should be selected (RAS-based combine score is higher)

    let ctx = TestContext::new().await;

    // Seed percentile data so RAS can compute scores
    ctx.seed_wr_percentiles().await;

    let team = ctx.create_team("RAS").await;
    let draft = Draft::new("Test Draft".to_string(), 2026, 7, 1).unwrap();
    let draft = ctx.draft_repo.create(&draft).await.unwrap();

    ctx.draft_engine.initialize_picks(draft.id).await.unwrap();

    let pick = ctx
        .pick_repo
        .find_next_pick(draft.id)
        .await
        .unwrap()
        .unwrap();

    // Create two WRs with same scouting grade
    let mut elite_wr = Player::new("Elite".to_string(), "WR".to_string(), Position::WR, 2026).unwrap();
    elite_wr.height_inches = Some(74);  // 6'2" (p90 for WR)
    elite_wr.weight_pounds = Some(202); // p90
    let elite_wr = ctx.player_repo.create(&elite_wr).await.unwrap();

    let mut avg_wr = Player::new("Average".to_string(), "WR".to_string(), Position::WR, 2026).unwrap();
    avg_wr.height_inches = Some(71);    // p40
    avg_wr.weight_pounds = Some(185);   // p40
    let avg_wr = ctx.player_repo.create(&avg_wr).await.unwrap();

    // Both have same scouting grade
    ctx.create_scouting_report(elite_wr.id, team.id, 8.0, Some(FitGrade::B), false, false).await;
    ctx.create_scouting_report(avg_wr.id, team.id, 8.0, Some(FitGrade::B), false, false).await;

    // Elite WR: all measurements near p90+ (fast times = low numbers)
    let elite_combine = CombineResults::new(elite_wr.id, 2026)
        .unwrap()
        .with_forty_yard_dash(4.30).unwrap()       // faster than p10 (elite)
        .with_bench_press(22).unwrap()               // > p90
        .with_vertical_jump(43.0).unwrap()           // > p90
        .with_broad_jump(132).unwrap()               // > p90
        .with_three_cone_drill(6.50).unwrap()        // faster than p10 (elite)
        .with_twenty_yard_shuttle(3.93).unwrap()     // faster than p10 (elite)
        .with_ten_yard_split(1.45).unwrap()          // faster than p10 (elite)
        .with_twenty_yard_split(2.44).unwrap();      // faster than p10 (elite)
    ctx.combine_repo.create(&elite_combine).await.unwrap();

    // Average WR: measurements near p50
    let avg_combine = CombineResults::new(avg_wr.id, 2026)
        .unwrap()
        .with_forty_yard_dash(4.45).unwrap()         // p50
        .with_bench_press(14).unwrap()                // p50
        .with_vertical_jump(37.0).unwrap()            // p50
        .with_broad_jump(122).unwrap()                // p50
        .with_three_cone_drill(6.80).unwrap()         // p50
        .with_twenty_yard_shuttle(4.10).unwrap()      // p50
        .with_ten_yard_split(1.52).unwrap()           // p50
        .with_twenty_yard_split(2.56).unwrap();       // p50
    ctx.combine_repo.create(&avg_combine).await.unwrap();

    // Execute auto-pick — RAS should give elite WR a much higher combine component
    let updated_pick = ctx.draft_engine.execute_auto_pick(pick.id).await.unwrap();

    // Verify elite WR was selected
    assert_eq!(
        updated_pick.player_id,
        Some(elite_wr.id),
        "Elite WR should be picked over average WR when RAS percentiles are available"
    );

    ctx.cleanup().await;
}
