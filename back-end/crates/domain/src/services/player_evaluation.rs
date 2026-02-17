use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};
use crate::models::{CombineResults, Player, Position, ScoutingReport};
use crate::repositories::{CombineResultsRepository, ScoutingReportRepository};
use crate::services::RasScoringService;

/// Service for evaluating players and calculating BPA (Best Player Available) scores
pub struct PlayerEvaluationService {
    scouting_repo: Arc<dyn ScoutingReportRepository>,
    combine_repo: Arc<dyn CombineResultsRepository>,
    ras_service: Option<Arc<RasScoringService>>,
}

impl PlayerEvaluationService {
    pub fn new(
        scouting_repo: Arc<dyn ScoutingReportRepository>,
        combine_repo: Arc<dyn CombineResultsRepository>,
    ) -> Self {
        Self {
            scouting_repo,
            combine_repo,
            ras_service: None,
        }
    }

    /// Add RAS scoring service for percentile-based combine evaluation
    pub fn with_ras_service(mut self, ras_service: Arc<RasScoringService>) -> Self {
        self.ras_service = Some(ras_service);
        self
    }

    /// Calculate BPA score for a player from a specific team's perspective
    /// Formula: BPA = (scouting_grade × 0.60) + (combine_score × 0.20) + (fit_score × 0.15) - concern_penalty
    pub async fn calculate_bpa_score(&self, player: &Player, team_id: Uuid) -> DomainResult<f64> {
        // Get scouting report for this team
        let scouting_report = self
            .scouting_repo
            .find_by_team_and_player(team_id, player.id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!(
                    "No scouting report found for player {} and team {}",
                    player.id, team_id
                ))
            })?;

        // Get combine results (get first one if multiple)
        let combine_results_list = self.combine_repo.find_by_player_id(player.id).await?;
        let combine_results = combine_results_list.first();

        // Calculate combine component: prefer RAS if available, fall back to hardcoded normalization
        let combine_score = match (&self.ras_service, combine_results) {
            (Some(ras), Some(combine)) => {
                let ras_score = ras.calculate_ras(player, combine).await;
                // RAS overall_score is 0-10; convert to 0-100 scale
                ras_score.overall_score.map(|s| s * 10.0)
            }
            _ => {
                tracing::debug!(
                    player_id = %player.id,
                    position = ?player.position,
                    has_ras_service = self.ras_service.is_some(),
                    has_combine_results = combine_results.is_some(),
                    "RAS scoring unavailable, falling back to hardcoded combine score normalization"
                );
                None
            }
        };
        let combine_component = combine_score
            .or_else(|| combine_results.map(|c| self.calculate_combine_score(c, &player.position)))
            .unwrap_or(50.0)
            * 0.20;

        // Calculate components
        let scouting_component = Self::normalize_scouting_grade(scouting_report.grade) * 0.60;
        let fit_component = Self::calculate_fit_score(&scouting_report) * 0.15;
        let concern_penalty = Self::calculate_concern_penalty(&scouting_report);

        let bpa_score = scouting_component + combine_component + fit_component - concern_penalty;

        Ok(bpa_score.clamp(0.0, 100.0))
    }

    /// Access the RAS service (for pre-fetching percentiles)
    pub fn ras_service(&self) -> Option<&Arc<RasScoringService>> {
        self.ras_service.as_ref()
    }

    /// Fetch all scouting reports for a team (for pre-loading in batch operations)
    pub async fn fetch_team_scouting_reports(
        &self,
        team_id: Uuid,
    ) -> DomainResult<Vec<ScoutingReport>> {
        self.scouting_repo.find_by_team_id(team_id).await
    }

    /// Fetch combine results for a player (for pre-loading in batch operations)
    pub async fn fetch_player_combine_results(
        &self,
        player_id: Uuid,
    ) -> DomainResult<Vec<CombineResults>> {
        self.combine_repo.find_by_player_id(player_id).await
    }

    /// Calculate BPA score using pre-fetched data (avoids N+1 queries in batch scoring).
    /// `scouting_report`: the team's scouting report for this player (None → skip player)
    /// `combine_results`: the player's combine results (may be empty)
    /// `percentiles`: pre-fetched percentile data for RAS scoring
    pub fn calculate_bpa_score_preloaded(
        &self,
        player: &Player,
        scouting_report: &ScoutingReport,
        combine_results: Option<&CombineResults>,
        percentiles: &[crate::models::CombinePercentile],
    ) -> f64 {
        // Calculate combine component: prefer RAS if available
        let combine_score = match (&self.ras_service, combine_results) {
            (Some(_), Some(combine)) if !percentiles.is_empty() => {
                let ras_score =
                    RasScoringService::calculate_ras_with_percentiles(player, combine, percentiles);
                ras_score.overall_score.map(|s| s * 10.0)
            }
            _ => None,
        };
        let combine_component = combine_score
            .or_else(|| combine_results.map(|c| self.calculate_combine_score(c, &player.position)))
            .unwrap_or(50.0)
            * 0.20;

        let scouting_component = Self::normalize_scouting_grade(scouting_report.grade) * 0.60;
        let fit_component = Self::calculate_fit_score(scouting_report) * 0.15;
        let concern_penalty = Self::calculate_concern_penalty(scouting_report);

        let bpa_score = scouting_component + combine_component + fit_component - concern_penalty;

        bpa_score.clamp(0.0, 100.0)
    }

    /// Rank multiple players by BPA score (highest to lowest)
    pub async fn rank_players_bpa(
        &self,
        players: &[Player],
        team_id: Uuid,
    ) -> DomainResult<Vec<(Player, f64)>> {
        let mut scored_players = Vec::new();

        for player in players {
            match self.calculate_bpa_score(player, team_id).await {
                Ok(score) => scored_players.push((player.clone(), score)),
                Err(_) => {
                    // Skip players without scouting reports
                    continue;
                }
            }
        }

        // Sort by score descending
        scored_players.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored_players)
    }

    /// Calculate position-weighted combine score (0-100 scale).
    ///
    /// This is a fallback scoring method used when RAS (Relative Athletic Score)
    /// percentile data is unavailable. It uses hardcoded normalization ranges to
    /// convert raw combine measurements into a weighted score based on the
    /// player's position.
    pub fn calculate_combine_score(&self, combine: &CombineResults, position: &Position) -> f64 {
        match position {
            // QB: Agility > Speed
            Position::QB => {
                let cone = combine
                    .three_cone_drill
                    .map(|t| Self::normalize_cone_drill(t) * 0.35)
                    .unwrap_or(17.5);
                let shuttle = combine
                    .twenty_yard_shuttle
                    .map(|t| Self::normalize_shuttle(t) * 0.35)
                    .unwrap_or(17.5);
                let forty = combine
                    .forty_yard_dash
                    .map(|t| Self::normalize_forty_dash(t) * 0.30)
                    .unwrap_or(15.0);
                cone + shuttle + forty
            }
            // WR/RB: Speed > Strength
            Position::WR | Position::RB => {
                let forty = combine
                    .forty_yard_dash
                    .map(|t| Self::normalize_forty_dash(t) * 0.40)
                    .unwrap_or(20.0);
                let vertical = combine
                    .vertical_jump
                    .map(|j| Self::normalize_vertical_jump(j) * 0.30)
                    .unwrap_or(15.0);
                let cone = combine
                    .three_cone_drill
                    .map(|t| Self::normalize_cone_drill(t) * 0.20)
                    .unwrap_or(10.0);
                let shuttle = combine
                    .twenty_yard_shuttle
                    .map(|t| Self::normalize_shuttle(t) * 0.10)
                    .unwrap_or(5.0);
                forty + vertical + cone + shuttle
            }
            // OL: Strength > Speed
            Position::OT | Position::OG | Position::C => {
                let bench = combine
                    .bench_press
                    .map(|r| Self::normalize_bench_press(r) * 0.40)
                    .unwrap_or(20.0);
                let broad = combine
                    .broad_jump
                    .map(|j| Self::normalize_broad_jump(j) * 0.30)
                    .unwrap_or(15.0);
                let forty = combine
                    .forty_yard_dash
                    .map(|t| Self::normalize_forty_dash(t) * 0.20)
                    .unwrap_or(10.0);
                let cone = combine
                    .three_cone_drill
                    .map(|t| Self::normalize_cone_drill(t) * 0.10)
                    .unwrap_or(5.0);
                bench + broad + forty + cone
            }
            // DL: Balance of speed and strength
            Position::DE | Position::DT => {
                let forty = combine
                    .forty_yard_dash
                    .map(|t| Self::normalize_forty_dash(t) * 0.30)
                    .unwrap_or(15.0);
                let bench = combine
                    .bench_press
                    .map(|r| Self::normalize_bench_press(r) * 0.25)
                    .unwrap_or(12.5);
                let vertical = combine
                    .vertical_jump
                    .map(|j| Self::normalize_vertical_jump(j) * 0.25)
                    .unwrap_or(12.5);
                let cone = combine
                    .three_cone_drill
                    .map(|t| Self::normalize_cone_drill(t) * 0.20)
                    .unwrap_or(10.0);
                forty + bench + vertical + cone
            }
            // LB: Balance
            Position::LB => {
                let forty = combine
                    .forty_yard_dash
                    .map(|t| Self::normalize_forty_dash(t) * 0.30)
                    .unwrap_or(15.0);
                let bench = combine
                    .bench_press
                    .map(|r| Self::normalize_bench_press(r) * 0.20)
                    .unwrap_or(10.0);
                let vertical = combine
                    .vertical_jump
                    .map(|j| Self::normalize_vertical_jump(j) * 0.20)
                    .unwrap_or(10.0);
                let cone = combine
                    .three_cone_drill
                    .map(|t| Self::normalize_cone_drill(t) * 0.15)
                    .unwrap_or(7.5);
                let shuttle = combine
                    .twenty_yard_shuttle
                    .map(|t| Self::normalize_shuttle(t) * 0.15)
                    .unwrap_or(7.5);
                forty + bench + vertical + cone + shuttle
            }
            // DB: Speed and agility
            Position::CB | Position::S => {
                let forty = combine
                    .forty_yard_dash
                    .map(|t| Self::normalize_forty_dash(t) * 0.40)
                    .unwrap_or(20.0);
                let vertical = combine
                    .vertical_jump
                    .map(|j| Self::normalize_vertical_jump(j) * 0.20)
                    .unwrap_or(10.0);
                let cone = combine
                    .three_cone_drill
                    .map(|t| Self::normalize_cone_drill(t) * 0.20)
                    .unwrap_or(10.0);
                let shuttle = combine
                    .twenty_yard_shuttle
                    .map(|t| Self::normalize_shuttle(t) * 0.20)
                    .unwrap_or(10.0);
                forty + vertical + cone + shuttle
            }
            // TE: Balance
            Position::TE => {
                let forty = combine
                    .forty_yard_dash
                    .map(|t| Self::normalize_forty_dash(t) * 0.30)
                    .unwrap_or(15.0);
                let vertical = combine
                    .vertical_jump
                    .map(|j| Self::normalize_vertical_jump(j) * 0.25)
                    .unwrap_or(12.5);
                let bench = combine
                    .bench_press
                    .map(|r| Self::normalize_bench_press(r) * 0.25)
                    .unwrap_or(12.5);
                let cone = combine
                    .three_cone_drill
                    .map(|t| Self::normalize_cone_drill(t) * 0.20)
                    .unwrap_or(10.0);
                forty + vertical + bench + cone
            }
            // K/P: Use default score
            Position::K | Position::P => 50.0,
        }
    }

    // Normalization functions (convert raw values to 0-100 scale)

    fn normalize_scouting_grade(grade: f64) -> f64 {
        // Grade is 0-10, convert to 0-100
        grade * 10.0
    }

    fn normalize_forty_dash(time: f64) -> f64 {
        // Faster is better. 4.3s = 100, 5.0s = 50, 5.5s = 0
        ((5.5 - time) / 1.2 * 100.0).clamp(0.0, 100.0)
    }

    fn normalize_bench_press(reps: i32) -> f64 {
        // More reps is better. 30 reps = 100, 20 reps = 50, 10 reps = 0
        ((reps as f64 - 10.0) / 20.0 * 100.0).clamp(0.0, 100.0)
    }

    fn normalize_vertical_jump(inches: f64) -> f64 {
        // Higher is better. 42" = 100, 33" = 50, 24" = 0
        ((inches - 24.0) / 18.0 * 100.0).clamp(0.0, 100.0)
    }

    fn normalize_broad_jump(inches: i32) -> f64 {
        // Higher is better. 130" = 100, 110" = 50, 90" = 0
        ((inches as f64 - 90.0) / 40.0 * 100.0).clamp(0.0, 100.0)
    }

    fn normalize_cone_drill(time: f64) -> f64 {
        // Faster is better. 6.5s = 100, 7.2s = 50, 8.0s = 0
        ((8.0 - time) / 1.5 * 100.0).clamp(0.0, 100.0)
    }

    fn normalize_shuttle(time: f64) -> f64 {
        // Faster is better. 4.0s = 100, 4.4s = 50, 4.8s = 0
        ((4.8 - time) / 0.8 * 100.0).clamp(0.0, 100.0)
    }

    fn calculate_fit_score(scouting_report: &ScoutingReport) -> f64 {
        match scouting_report.fit_grade {
            Some(fit_grade) => match fit_grade {
                crate::models::FitGrade::A => 100.0,
                crate::models::FitGrade::B => 80.0,
                crate::models::FitGrade::C => 60.0,
                crate::models::FitGrade::D => 40.0,
                crate::models::FitGrade::F => 20.0,
            },
            None => 60.0, // Default to C grade if not specified
        }
    }

    fn calculate_concern_penalty(scouting_report: &ScoutingReport) -> f64 {
        let mut penalty = 0.0;
        if scouting_report.injury_concern {
            penalty += 5.0;
        }
        if scouting_report.character_concern {
            penalty += 5.0;
        }
        penalty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FitGrade;
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

    fn create_test_player(position: Position) -> Player {
        Player::new("John".to_string(), "Doe".to_string(), position, 2026).unwrap()
    }

    fn create_test_scouting_report(
        player_id: Uuid,
        team_id: Uuid,
        grade: f64,
        fit_grade: Option<FitGrade>,
        injury: bool,
        character: bool,
    ) -> ScoutingReport {
        let mut report = ScoutingReport::new(player_id, team_id, grade).unwrap();
        report.fit_grade = fit_grade;
        report.injury_concern = injury;
        report.character_concern = character;
        report
    }

    #[tokio::test]
    async fn test_calculate_bpa_score() {
        let mut scouting_mock = MockScoutingReportRepo::new();
        let mut combine_mock = MockCombineResultsRepo::new();

        let player = create_test_player(Position::QB);
        let team_id = Uuid::new_v4();

        let scouting_report =
            create_test_scouting_report(player.id, team_id, 8.5, Some(FitGrade::A), false, false);

        let combine = CombineResults::new(player.id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.5)
            .unwrap()
            .with_three_cone_drill(6.8)
            .unwrap()
            .with_twenty_yard_shuttle(4.2)
            .unwrap();

        scouting_mock
            .expect_find_by_team_and_player()
            .with(eq(team_id), eq(player.id))
            .times(1)
            .returning(move |_, _| Ok(Some(scouting_report.clone())));

        combine_mock
            .expect_find_by_player_id()
            .with(eq(player.id))
            .times(1)
            .returning(move |_| Ok(vec![combine.clone()]));

        let service = PlayerEvaluationService::new(Arc::new(scouting_mock), Arc::new(combine_mock));

        let score = service.calculate_bpa_score(&player, team_id).await.unwrap();

        // Score should be > 0 and <= 100
        assert!(score > 0.0);
        assert!(score <= 100.0);
    }

    #[tokio::test]
    async fn test_concern_penalties() {
        let mut scouting_mock = MockScoutingReportRepo::new();
        let mut combine_mock = MockCombineResultsRepo::new();

        let player = create_test_player(Position::QB);
        let team_id = Uuid::new_v4();

        // Report with both injury and character concerns
        let scouting_report =
            create_test_scouting_report(player.id, team_id, 8.5, Some(FitGrade::A), true, true);

        scouting_mock
            .expect_find_by_team_and_player()
            .with(eq(team_id), eq(player.id))
            .times(1)
            .returning(move |_, _| Ok(Some(scouting_report.clone())));

        combine_mock
            .expect_find_by_player_id()
            .with(eq(player.id))
            .times(1)
            .returning(|_| Ok(vec![]));

        let service = PlayerEvaluationService::new(Arc::new(scouting_mock), Arc::new(combine_mock));

        let score = service.calculate_bpa_score(&player, team_id).await.unwrap();

        // Score should be penalized for concerns
        assert!(score < 100.0);
    }

    #[test]
    fn test_normalize_forty_dash() {
        assert!(PlayerEvaluationService::normalize_forty_dash(4.3) > 95.0);
        assert!((PlayerEvaluationService::normalize_forty_dash(5.0) - 50.0).abs() < 10.0);
        assert!(PlayerEvaluationService::normalize_forty_dash(5.5) < 5.0);
    }

    #[test]
    fn test_normalize_bench_press() {
        assert!(PlayerEvaluationService::normalize_bench_press(30) > 95.0);
        assert!((PlayerEvaluationService::normalize_bench_press(20) - 50.0).abs() < 5.0);
        assert!(PlayerEvaluationService::normalize_bench_press(10) < 5.0);
    }

    #[test]
    fn test_normalize_vertical_jump() {
        assert!(PlayerEvaluationService::normalize_vertical_jump(42.0) > 95.0);
        assert!((PlayerEvaluationService::normalize_vertical_jump(33.0) - 50.0).abs() < 10.0);
        assert!(PlayerEvaluationService::normalize_vertical_jump(24.0) < 5.0);
    }

    mock! {
        CombinePercentileRepo {}

        #[async_trait::async_trait]
        impl crate::repositories::CombinePercentileRepository for CombinePercentileRepo {
            async fn find_all(&self) -> DomainResult<Vec<crate::models::CombinePercentile>>;
            async fn find_by_position(&self, position: &str) -> DomainResult<Vec<crate::models::CombinePercentile>>;
            async fn find_by_position_and_measurement(
                &self,
                position: &str,
                measurement: &str,
            ) -> DomainResult<Option<crate::models::CombinePercentile>>;
            async fn upsert(&self, percentile: &crate::models::CombinePercentile) -> DomainResult<crate::models::CombinePercentile>;
            async fn delete_all(&self) -> DomainResult<u64>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
        }
    }

    fn make_percentile(
        position: &str,
        measurement: &str,
        p10: f64,
        p50: f64,
        p90: f64,
    ) -> crate::models::CombinePercentile {
        let m: crate::models::Measurement = measurement.parse().unwrap();
        crate::models::CombinePercentile::new(position.to_string(), m)
            .unwrap()
            .with_percentiles(
                100,
                p10 - 0.5 * (p50 - p10), // min
                p10,
                p10 + 0.25 * (p50 - p10),
                p10 + 0.5 * (p50 - p10),
                p10 + 0.75 * (p50 - p10),
                p50,
                p50 + 0.25 * (p90 - p50),
                p50 + 0.5 * (p90 - p50),
                p50 + 0.75 * (p90 - p50),
                p90,
                p90 + 0.5 * (p90 - p50), // max
            )
            .unwrap()
    }

    #[tokio::test]
    async fn test_calculate_bpa_score_with_ras_service() {
        let mut scouting_mock = MockScoutingReportRepo::new();
        let mut combine_mock = MockCombineResultsRepo::new();
        let mut percentile_mock = MockCombinePercentileRepo::new();

        let player = Player::new("Jane".to_string(), "Smith".to_string(), Position::WR, 2026)
            .unwrap()
            .with_physical_stats(72, 200)
            .unwrap();
        let team_id = Uuid::new_v4();

        let scouting_report =
            create_test_scouting_report(player.id, team_id, 8.0, Some(FitGrade::B), false, false);

        let combine = CombineResults::new(player.id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.4)
            .unwrap()
            .with_vertical_jump(38.0)
            .unwrap()
            .with_broad_jump(120)
            .unwrap()
            .with_three_cone_drill(6.8)
            .unwrap()
            .with_twenty_yard_shuttle(4.2)
            .unwrap()
            .with_bench_press(15)
            .unwrap()
            .with_ten_yard_split(1.5)
            .unwrap()
            .with_twenty_yard_split(2.6)
            .unwrap();

        scouting_mock
            .expect_find_by_team_and_player()
            .with(eq(team_id), eq(player.id))
            .times(1)
            .returning(move |_, _| Ok(Some(scouting_report.clone())));

        combine_mock
            .expect_find_by_player_id()
            .with(eq(player.id))
            .times(1)
            .returning(move |_| Ok(vec![combine.clone()]));

        // Mock percentile repo to return data for each measurement
        percentile_mock
            .expect_find_by_position_and_measurement()
            .returning(|position, measurement| {
                let percentile = match measurement {
                    "height" => Some(make_percentile(position, "height", 70.0, 72.0, 75.0)),
                    "weight" => Some(make_percentile(position, "weight", 185.0, 200.0, 215.0)),
                    "forty_yard_dash" => Some(make_percentile(position, "forty_yard_dash", 4.35, 4.48, 4.62)),
                    "vertical_jump" => Some(make_percentile(position, "vertical_jump", 30.0, 36.0, 41.0)),
                    "broad_jump" => Some(make_percentile(position, "broad_jump", 110.0, 120.0, 130.0)),
                    "three_cone_drill" => Some(make_percentile(position, "three_cone_drill", 6.7, 7.0, 7.3)),
                    "twenty_yard_shuttle" => Some(make_percentile(position, "twenty_yard_shuttle", 4.1, 4.3, 4.5)),
                    "bench_press" => Some(make_percentile(position, "bench_press", 10.0, 16.0, 22.0)),
                    "ten_yard_split" => Some(make_percentile(position, "ten_yard_split", 1.48, 1.55, 1.62)),
                    "twenty_yard_split" => Some(make_percentile(position, "twenty_yard_split", 2.5, 2.6, 2.7)),
                    _ => None,
                };
                Ok(percentile)
            });

        let ras_service = RasScoringService::new(Arc::new(percentile_mock));
        let service = PlayerEvaluationService::new(Arc::new(scouting_mock), Arc::new(combine_mock))
            .with_ras_service(Arc::new(ras_service));

        let score = service.calculate_bpa_score(&player, team_id).await.unwrap();

        // Score should be > 0 and <= 100, and the RAS path was used
        assert!(score > 0.0, "BPA score should be positive, got {}", score);
        assert!(score <= 100.0, "BPA score should be at most 100, got {}", score);
    }

    #[test]
    fn test_calculate_bpa_score_preloaded_with_percentiles() {
        let scouting_mock = MockScoutingReportRepo::new();
        let combine_mock = MockCombineResultsRepo::new();
        let percentile_mock = MockCombinePercentileRepo::new();

        let player = Player::new("Tom".to_string(), "Brady".to_string(), Position::WR, 2026)
            .unwrap()
            .with_physical_stats(73, 205)
            .unwrap();
        let team_id = Uuid::new_v4();

        let scouting_report =
            create_test_scouting_report(player.id, team_id, 9.0, Some(FitGrade::A), false, false);

        let combine = CombineResults::new(player.id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.4)
            .unwrap()
            .with_vertical_jump(38.0)
            .unwrap()
            .with_broad_jump(125)
            .unwrap()
            .with_three_cone_drill(6.8)
            .unwrap()
            .with_twenty_yard_shuttle(4.1)
            .unwrap()
            .with_bench_press(18)
            .unwrap()
            .with_ten_yard_split(1.5)
            .unwrap()
            .with_twenty_yard_split(2.55)
            .unwrap();

        // Build percentile data for "WR" position
        let percentiles = vec![
            make_percentile("WR", "height", 70.0, 72.0, 75.0),
            make_percentile("WR", "weight", 185.0, 200.0, 215.0),
            make_percentile("WR", "forty_yard_dash", 4.35, 4.48, 4.62),
            make_percentile("WR", "vertical_jump", 30.0, 36.0, 41.0),
            make_percentile("WR", "broad_jump", 110.0, 120.0, 130.0),
            make_percentile("WR", "three_cone_drill", 6.7, 7.0, 7.3),
            make_percentile("WR", "twenty_yard_shuttle", 4.1, 4.3, 4.5),
            make_percentile("WR", "bench_press", 10.0, 16.0, 22.0),
            make_percentile("WR", "ten_yard_split", 1.48, 1.55, 1.62),
            make_percentile("WR", "twenty_yard_split", 2.5, 2.6, 2.7),
        ];

        let ras_service = RasScoringService::new(Arc::new(percentile_mock));
        let service = PlayerEvaluationService::new(Arc::new(scouting_mock), Arc::new(combine_mock))
            .with_ras_service(Arc::new(ras_service));

        let score = service.calculate_bpa_score_preloaded(
            &player,
            &scouting_report,
            Some(&combine),
            &percentiles,
        );

        // Score should be > 0 and <= 100
        assert!(score > 0.0, "Preloaded BPA score should be positive, got {}", score);
        assert!(score <= 100.0, "Preloaded BPA score should be at most 100, got {}", score);
    }

    #[test]
    fn test_fit_score_calculation() {
        let player_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let report_a =
            create_test_scouting_report(player_id, team_id, 8.0, Some(FitGrade::A), false, false);
        assert_eq!(
            PlayerEvaluationService::calculate_fit_score(&report_a),
            100.0
        );

        let report_b =
            create_test_scouting_report(player_id, team_id, 8.0, Some(FitGrade::B), false, false);
        assert_eq!(
            PlayerEvaluationService::calculate_fit_score(&report_b),
            80.0
        );

        let report_f =
            create_test_scouting_report(player_id, team_id, 8.0, Some(FitGrade::F), false, false);
        assert_eq!(
            PlayerEvaluationService::calculate_fit_score(&report_f),
            20.0
        );

        let report_none = create_test_scouting_report(player_id, team_id, 8.0, None, false, false);
        assert_eq!(
            PlayerEvaluationService::calculate_fit_score(&report_none),
            60.0
        );
    }
}
