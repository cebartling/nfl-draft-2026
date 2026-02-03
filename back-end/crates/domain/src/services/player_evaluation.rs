use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};
use crate::models::{CombineResults, Player, Position, ScoutingReport};
use crate::repositories::{CombineResultsRepository, ScoutingReportRepository};

/// Service for evaluating players and calculating BPA (Best Player Available) scores
pub struct PlayerEvaluationService {
    scouting_repo: Arc<dyn ScoutingReportRepository>,
    combine_repo: Arc<dyn CombineResultsRepository>,
}

impl PlayerEvaluationService {
    pub fn new(
        scouting_repo: Arc<dyn ScoutingReportRepository>,
        combine_repo: Arc<dyn CombineResultsRepository>,
    ) -> Self {
        Self {
            scouting_repo,
            combine_repo,
        }
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

        // Calculate components
        let scouting_component = Self::normalize_scouting_grade(scouting_report.grade) * 0.60;
        let combine_component = combine_results
            .map(|c| self.calculate_combine_score(c, &player.position))
            .unwrap_or(50.0)
            * 0.20;
        let fit_component = Self::calculate_fit_score(&scouting_report) * 0.15;
        let concern_penalty = Self::calculate_concern_penalty(&scouting_report);

        let bpa_score = scouting_component + combine_component + fit_component - concern_penalty;

        Ok(bpa_score.max(0.0).min(100.0))
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
        scored_players.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(scored_players)
    }

    /// Calculate position-weighted combine score (0-100 scale)
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
        ((5.5 - time) / 1.2 * 100.0).max(0.0).min(100.0)
    }

    fn normalize_bench_press(reps: i32) -> f64 {
        // More reps is better. 30 reps = 100, 20 reps = 50, 10 reps = 0
        ((reps as f64 - 10.0) / 20.0 * 100.0).max(0.0).min(100.0)
    }

    fn normalize_vertical_jump(inches: f64) -> f64 {
        // Higher is better. 42" = 100, 33" = 50, 24" = 0
        ((inches - 24.0) / 18.0 * 100.0).max(0.0).min(100.0)
    }

    fn normalize_broad_jump(inches: i32) -> f64 {
        // Higher is better. 130" = 100, 110" = 50, 90" = 0
        ((inches as f64 - 90.0) / 40.0 * 100.0).max(0.0).min(100.0)
    }

    fn normalize_cone_drill(time: f64) -> f64 {
        // Faster is better. 6.5s = 100, 7.2s = 50, 8.0s = 0
        ((8.0 - time) / 1.5 * 100.0).max(0.0).min(100.0)
    }

    fn normalize_shuttle(time: f64) -> f64 {
        // Faster is better. 4.0s = 100, 4.4s = 50, 4.8s = 0
        ((4.8 - time) / 0.8 * 100.0).max(0.0).min(100.0)
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
