use std::sync::Arc;

use crate::models::ras_score::{MeasurementScore, RasScore};
use crate::models::{CombinePercentile, CombineResults, Player};
use crate::repositories::CombinePercentileRepository;

/// Service for calculating RAS (Relative Athletic Score)
pub struct RasScoringService {
    percentile_repo: Arc<dyn CombinePercentileRepository>,
}

/// Measurements that are lower-is-better (timed events)
const LOWER_IS_BETTER: &[&str] = &[
    "forty_yard_dash",
    "three_cone_drill",
    "twenty_yard_shuttle",
    "ten_yard_split",
    "twenty_yard_split",
];

/// Category groupings for sub-scores
const SIZE_MEASUREMENTS: &[&str] = &["height", "weight"];
const SPEED_MEASUREMENTS: &[&str] = &["forty_yard_dash", "ten_yard_split", "twenty_yard_split"];
const STRENGTH_MEASUREMENTS: &[&str] = &["bench_press"];
const EXPLOSION_MEASUREMENTS: &[&str] = &["vertical_jump", "broad_jump"];
const AGILITY_MEASUREMENTS: &[&str] = &["three_cone_drill", "twenty_yard_shuttle"];

impl RasScoringService {
    pub fn new(percentile_repo: Arc<dyn CombinePercentileRepository>) -> Self {
        Self { percentile_repo }
    }

    /// Calculate RAS score for a player given their combine results.
    /// Returns RasScore with overall and category breakdowns.
    pub async fn calculate_ras(
        &self,
        player: &Player,
        combine_results: &CombineResults,
    ) -> RasScore {
        let position = map_position_for_percentile(&player.position);

        // Collect all available measurements
        let mut individual_scores = Vec::new();

        // Height (from Player)
        if let Some(height) = player.height_inches {
            if let Some(score) = self
                .score_measurement(&position, "height", height as f64)
                .await
            {
                individual_scores.push(score);
            }
        }

        // Weight (from Player)
        if let Some(weight) = player.weight_pounds {
            if let Some(score) = self
                .score_measurement(&position, "weight", weight as f64)
                .await
            {
                individual_scores.push(score);
            }
        }

        // Combine measurements
        let measurements: Vec<(&str, Option<f64>)> = vec![
            ("forty_yard_dash", combine_results.forty_yard_dash),
            (
                "bench_press",
                combine_results.bench_press.map(|v| v as f64),
            ),
            ("vertical_jump", combine_results.vertical_jump),
            ("broad_jump", combine_results.broad_jump.map(|v| v as f64)),
            ("three_cone_drill", combine_results.three_cone_drill),
            ("twenty_yard_shuttle", combine_results.twenty_yard_shuttle),
            ("ten_yard_split", combine_results.ten_yard_split),
            ("twenty_yard_split", combine_results.twenty_yard_split),
        ];

        for (name, value) in measurements {
            if let Some(raw) = value {
                if let Some(score) = self.score_measurement(&position, name, raw).await {
                    individual_scores.push(score);
                }
            }
        }

        let measurements_used = individual_scores.len();

        // Calculate category scores
        let size_score = category_average(&individual_scores, SIZE_MEASUREMENTS);
        let speed_score = category_average(&individual_scores, SPEED_MEASUREMENTS);
        let strength_score = category_average(&individual_scores, STRENGTH_MEASUREMENTS);
        let explosion_score = category_average(&individual_scores, EXPLOSION_MEASUREMENTS);
        let agility_score = category_average(&individual_scores, AGILITY_MEASUREMENTS);

        // Calculate overall score (only if minimum measurements met)
        let (overall_score, explanation) = if measurements_used >= RasScore::MIN_MEASUREMENTS {
            let avg: f64 =
                individual_scores.iter().map(|s| s.score).sum::<f64>() / measurements_used as f64;
            (Some((avg * 100.0).round() / 100.0), None)
        } else {
            (
                None,
                Some(format!(
                    "Insufficient measurements: {} of {} minimum required",
                    measurements_used,
                    RasScore::MIN_MEASUREMENTS
                )),
            )
        };

        RasScore {
            player_id: player.id,
            overall_score,
            size_score,
            speed_score,
            strength_score,
            explosion_score,
            agility_score,
            measurements_used,
            measurements_total: RasScore::TOTAL_MEASUREMENTS,
            individual_scores,
            explanation,
        }
    }

    /// Calculate RAS score using pre-fetched percentile data (avoids N+1 queries).
    /// `percentiles` should contain all percentiles for the player's position group.
    pub fn calculate_ras_with_percentiles(
        player: &Player,
        combine_results: &CombineResults,
        percentiles: &[CombinePercentile],
    ) -> RasScore {
        let position = map_position_for_percentile(&player.position);

        let mut individual_scores = Vec::new();

        // Height (from Player)
        if let Some(height) = player.height_inches {
            if let Some(score) =
                score_measurement_from_cache(&position, "height", height as f64, percentiles)
            {
                individual_scores.push(score);
            }
        }

        // Weight (from Player)
        if let Some(weight) = player.weight_pounds {
            if let Some(score) =
                score_measurement_from_cache(&position, "weight", weight as f64, percentiles)
            {
                individual_scores.push(score);
            }
        }

        // Combine measurements
        let measurements: Vec<(&str, Option<f64>)> = vec![
            ("forty_yard_dash", combine_results.forty_yard_dash),
            (
                "bench_press",
                combine_results.bench_press.map(|v| v as f64),
            ),
            ("vertical_jump", combine_results.vertical_jump),
            ("broad_jump", combine_results.broad_jump.map(|v| v as f64)),
            ("three_cone_drill", combine_results.three_cone_drill),
            ("twenty_yard_shuttle", combine_results.twenty_yard_shuttle),
            ("ten_yard_split", combine_results.ten_yard_split),
            ("twenty_yard_split", combine_results.twenty_yard_split),
        ];

        for (name, value) in measurements {
            if let Some(raw) = value {
                if let Some(score) =
                    score_measurement_from_cache(&position, name, raw, percentiles)
                {
                    individual_scores.push(score);
                }
            }
        }

        let measurements_used = individual_scores.len();

        let size_score = category_average(&individual_scores, SIZE_MEASUREMENTS);
        let speed_score = category_average(&individual_scores, SPEED_MEASUREMENTS);
        let strength_score = category_average(&individual_scores, STRENGTH_MEASUREMENTS);
        let explosion_score = category_average(&individual_scores, EXPLOSION_MEASUREMENTS);
        let agility_score = category_average(&individual_scores, AGILITY_MEASUREMENTS);

        let (overall_score, explanation) = if measurements_used >= RasScore::MIN_MEASUREMENTS {
            let avg: f64 =
                individual_scores.iter().map(|s| s.score).sum::<f64>() / measurements_used as f64;
            (Some((avg * 100.0).round() / 100.0), None)
        } else {
            (
                None,
                Some(format!(
                    "Insufficient measurements: {} of {} minimum required",
                    measurements_used,
                    RasScore::MIN_MEASUREMENTS
                )),
            )
        };

        RasScore {
            player_id: player.id,
            overall_score,
            size_score,
            speed_score,
            strength_score,
            explosion_score,
            agility_score,
            measurements_used,
            measurements_total: RasScore::TOTAL_MEASUREMENTS,
            individual_scores,
            explanation,
        }
    }

    /// Map a Position enum to its percentile position group string
    pub fn map_position(position: &crate::models::Position) -> String {
        map_position_for_percentile(position)
    }

    /// Pre-fetch all percentiles for a position group (for batch scoring).
    pub async fn fetch_percentiles_for_position(
        &self,
        position: &str,
    ) -> Vec<CombinePercentile> {
        self.percentile_repo
            .find_by_position(position)
            .await
            .unwrap_or_default()
    }

    /// Score a single measurement against position percentiles.
    /// Returns a 0-10 score based on where the value falls in the percentile distribution.
    async fn score_measurement(
        &self,
        position: &str,
        measurement: &str,
        raw_value: f64,
    ) -> Option<MeasurementScore> {
        let percentile_data = self
            .percentile_repo
            .find_by_position_and_measurement(position, measurement)
            .await
            .ok()
            .flatten()?;

        let percentile = calculate_percentile(&percentile_data, raw_value, measurement);
        let score = percentile / 10.0; // Convert 0-100 percentile to 0-10 score

        Some(MeasurementScore {
            measurement: measurement.to_string(),
            raw_value,
            percentile,
            score,
        })
    }
}

/// Score a single measurement using pre-fetched percentile data
fn score_measurement_from_cache(
    position: &str,
    measurement: &str,
    raw_value: f64,
    percentiles: &[CombinePercentile],
) -> Option<MeasurementScore> {
    let percentile_data = percentiles
        .iter()
        .find(|p| p.position == position && p.measurement.to_string() == measurement)?;

    let percentile = calculate_percentile(percentile_data, raw_value, measurement);
    let score = percentile / 10.0;

    Some(MeasurementScore {
        measurement: measurement.to_string(),
        raw_value,
        percentile,
        score,
    })
}

/// Calculate what percentile a value falls in, given the percentile breakpoints.
/// For lower-is-better measurements, we invert the percentile.
fn calculate_percentile(
    data: &CombinePercentile,
    value: f64,
    measurement: &str,
) -> f64 {
    let is_lower_better = LOWER_IS_BETTER.contains(&measurement);

    // The breakpoints from database: min, p10, p20, ..., p90, max
    let breakpoints = [
        (0.0, data.min_value),
        (10.0, data.p10),
        (20.0, data.p20),
        (30.0, data.p30),
        (40.0, data.p40),
        (50.0, data.p50),
        (60.0, data.p60),
        (70.0, data.p70),
        (80.0, data.p80),
        (90.0, data.p90),
        (100.0, data.max_value),
    ];

    // Linear interpolation between breakpoints
    let raw_percentile = if value <= breakpoints[0].1 {
        0.0
    } else if value >= breakpoints[breakpoints.len() - 1].1 {
        100.0
    } else {
        let mut percentile = 50.0; // default
        for i in 0..breakpoints.len() - 1 {
            let (pct_low, val_low) = breakpoints[i];
            let (pct_high, val_high) = breakpoints[i + 1];

            if val_low <= val_high {
                // Ascending breakpoints
                if value >= val_low && value <= val_high {
                    if (val_high - val_low).abs() < f64::EPSILON {
                        percentile = pct_low;
                    } else {
                        percentile =
                            pct_low + (value - val_low) / (val_high - val_low) * (pct_high - pct_low);
                    }
                    break;
                }
            } else {
                // Descending breakpoints (shouldn't happen with our data, but handle it)
                if value <= val_low && value >= val_high {
                    if (val_low - val_high).abs() < f64::EPSILON {
                        percentile = pct_low;
                    } else {
                        percentile =
                            pct_low + (val_low - value) / (val_low - val_high) * (pct_high - pct_low);
                    }
                    break;
                }
            }
        }
        percentile
    };

    if is_lower_better {
        // For timed events: lower value = higher percentile
        100.0 - raw_percentile
    } else {
        raw_percentile
    }
}

/// Calculate the average score for a category of measurements
fn category_average(scores: &[MeasurementScore], category: &[&str]) -> Option<f64> {
    let matching: Vec<f64> = scores
        .iter()
        .filter(|s| category.contains(&s.measurement.as_str()))
        .map(|s| s.score)
        .collect();

    if matching.is_empty() {
        None
    } else {
        let avg = matching.iter().sum::<f64>() / matching.len() as f64;
        Some((avg * 100.0).round() / 100.0)
    }
}

/// Map domain Position enum to the position strings used in combine_percentiles
pub fn map_position_for_percentile(position: &crate::models::Position) -> String {
    use crate::models::Position;
    match position {
        Position::QB => "QB",
        Position::RB => "RB",
        Position::WR => "WR",
        Position::TE => "TE",
        Position::OT => "OT",
        Position::OG | Position::C => "IOL",
        Position::DE => "EDGE",
        Position::DT => "DL",
        Position::LB => "LB",
        Position::CB => "CB",
        Position::S => "S",
        Position::K => "K",
        Position::P => "P",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Measurement, Position};

    fn make_percentile(
        position: &str,
        measurement: &str,
        p10: f64,
        p50: f64,
        p90: f64,
    ) -> CombinePercentile {
        let m: Measurement = measurement.parse().unwrap();
        CombinePercentile::new(position.to_string(), m)
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

    #[test]
    fn test_score_measurement_higher_is_better() {
        // Vertical jump: higher is better
        let data = make_percentile("WR", "vertical_jump", 28.0, 36.0, 41.0);

        // Value at p50 should score ~50th percentile = ~5.0
        let pct = calculate_percentile(&data, 36.0, "vertical_jump");
        assert!(
            (pct - 50.0).abs() < 5.0,
            "p50 value should be near 50th percentile, got {}",
            pct
        );

        // Value at p90 should score ~90th percentile = ~9.0
        let pct = calculate_percentile(&data, 41.0, "vertical_jump");
        assert!(
            (pct - 90.0).abs() < 5.0,
            "p90 value should be near 90th percentile, got {}",
            pct
        );
    }

    #[test]
    fn test_score_measurement_lower_is_better() {
        // 40-yard dash: lower is better
        // Breakpoints stored in ASCENDING order (fast→slow): p10=4.35 (fast), p50=4.48, p90=4.62 (slow)
        // Inversion happens at scoring time: 100 - raw_percentile
        let data = make_percentile("WR", "forty_yard_dash", 4.35, 4.48, 4.62);

        // Very fast time (4.35 = p10) → raw percentile ~10 → inverted: ~90 → high
        let pct = calculate_percentile(&data, 4.35, "forty_yard_dash");
        assert!(pct > 70.0, "Fast 40 should have high percentile, got {}", pct);

        // Slow time (4.62 = p90) → raw percentile ~90 → inverted: ~10 → low
        let pct = calculate_percentile(&data, 4.62, "forty_yard_dash");
        assert!(pct < 30.0, "Slow 40 should have low percentile, got {}", pct);
    }

    #[test]
    fn test_ras_category_averages() {
        let scores = vec![
            MeasurementScore {
                measurement: "forty_yard_dash".to_string(),
                raw_value: 4.5,
                percentile: 70.0,
                score: 7.0,
            },
            MeasurementScore {
                measurement: "ten_yard_split".to_string(),
                raw_value: 1.55,
                percentile: 60.0,
                score: 6.0,
            },
            MeasurementScore {
                measurement: "vertical_jump".to_string(),
                raw_value: 36.0,
                percentile: 50.0,
                score: 5.0,
            },
        ];

        let speed = category_average(&scores, SPEED_MEASUREMENTS).unwrap();
        assert!((speed - 6.5).abs() < 0.01, "Speed should avg 7+6/2 = 6.5, got {}", speed);

        let explosion = category_average(&scores, EXPLOSION_MEASUREMENTS).unwrap();
        assert!((explosion - 5.0).abs() < 0.01, "Explosion should be 5.0, got {}", explosion);

        let strength = category_average(&scores, STRENGTH_MEASUREMENTS);
        assert!(strength.is_none(), "No strength measurements, should be None");
    }

    #[test]
    fn test_map_position_for_percentile() {
        assert_eq!(map_position_for_percentile(&Position::QB), "QB");
        assert_eq!(map_position_for_percentile(&Position::OG), "IOL");
        assert_eq!(map_position_for_percentile(&Position::C), "IOL");
        assert_eq!(map_position_for_percentile(&Position::DE), "EDGE");
        assert_eq!(map_position_for_percentile(&Position::DT), "DL");
    }

    // --- Edge case tests for calculate_percentile (Fix H5) ---

    #[test]
    fn test_calculate_percentile_value_below_min() {
        // Higher-is-better measurement (vertical_jump)
        let data = make_percentile("WR", "vertical_jump", 28.0, 36.0, 41.0);
        // Value well below min_value should yield raw percentile 0
        let min_val = data.min_value;
        let pct = calculate_percentile(&data, min_val - 5.0, "vertical_jump");
        assert!(
            (pct - 0.0).abs() < f64::EPSILON,
            "Value below min (higher-is-better) should return 0, got {}",
            pct
        );

        // Lower-is-better measurement (forty_yard_dash): below min means very fast, so inverted = 100
        let data_dash = make_percentile("WR", "forty_yard_dash", 4.35, 4.48, 4.62);
        let min_dash = data_dash.min_value;
        let pct_dash = calculate_percentile(&data_dash, min_dash - 0.5, "forty_yard_dash");
        assert!(
            (pct_dash - 100.0).abs() < f64::EPSILON,
            "Value below min (lower-is-better) should return 100, got {}",
            pct_dash
        );
    }

    #[test]
    fn test_calculate_percentile_value_above_max() {
        // Higher-is-better measurement (vertical_jump): above max should yield raw percentile 100
        let data = make_percentile("WR", "vertical_jump", 28.0, 36.0, 41.0);
        let max_val = data.max_value;
        let pct = calculate_percentile(&data, max_val + 5.0, "vertical_jump");
        assert!(
            (pct - 100.0).abs() < f64::EPSILON,
            "Value above max (higher-is-better) should return 100, got {}",
            pct
        );

        // Lower-is-better measurement (forty_yard_dash): above max means very slow, inverted = 0
        let data_dash = make_percentile("WR", "forty_yard_dash", 4.35, 4.48, 4.62);
        let max_dash = data_dash.max_value;
        let pct_dash = calculate_percentile(&data_dash, max_dash + 0.5, "forty_yard_dash");
        assert!(
            (pct_dash - 0.0).abs() < f64::EPSILON,
            "Value above max (lower-is-better) should return 0, got {}",
            pct_dash
        );
    }

    #[test]
    fn test_calculate_percentile_equal_breakpoints() {
        // Create percentile data where all breakpoints are the same value (flat segment)
        // This tests that the function does not divide by zero
        let m: Measurement = "bench_press".parse().unwrap();
        let data = CombinePercentile::new("WR".to_string(), m)
            .unwrap()
            .with_percentiles(
                50,
                20.0, // min
                20.0, // p10
                20.0, // p20
                20.0, // p30
                20.0, // p40
                20.0, // p50
                20.0, // p60
                20.0, // p70
                20.0, // p80
                20.0, // p90
                20.0, // max
            )
            .unwrap();

        // Value equal to all breakpoints should not panic (no division by zero)
        let pct = calculate_percentile(&data, 20.0, "bench_press");
        assert!(
            pct.is_finite(),
            "Percentile with equal breakpoints should be finite, got {}",
            pct
        );

        // Value above the flat breakpoints
        let pct_above = calculate_percentile(&data, 25.0, "bench_press");
        assert!(
            pct_above.is_finite(),
            "Percentile above flat breakpoints should be finite, got {}",
            pct_above
        );

        // Value below the flat breakpoints
        let pct_below = calculate_percentile(&data, 15.0, "bench_press");
        assert!(
            pct_below.is_finite(),
            "Percentile below flat breakpoints should be finite, got {}",
            pct_below
        );
    }

    #[test]
    fn test_category_average_empty_returns_none() {
        let empty_scores: Vec<MeasurementScore> = vec![];

        assert!(
            category_average(&empty_scores, SIZE_MEASUREMENTS).is_none(),
            "Empty scores should return None for size"
        );
        assert!(
            category_average(&empty_scores, SPEED_MEASUREMENTS).is_none(),
            "Empty scores should return None for speed"
        );
        assert!(
            category_average(&empty_scores, STRENGTH_MEASUREMENTS).is_none(),
            "Empty scores should return None for strength"
        );
        assert!(
            category_average(&empty_scores, EXPLOSION_MEASUREMENTS).is_none(),
            "Empty scores should return None for explosion"
        );
        assert!(
            category_average(&empty_scores, AGILITY_MEASUREMENTS).is_none(),
            "Empty scores should return None for agility"
        );
    }

    // --- Async unit tests for RasScoringService.calculate_ras (Fix M6) ---

    use crate::errors::DomainResult;
    use crate::models::CombineResults;
    use crate::repositories::CombinePercentileRepository;
    use mockall::mock;

    mock! {
        CombinePercentileRepo {}

        #[async_trait::async_trait]
        impl CombinePercentileRepository for CombinePercentileRepo {
            async fn find_all(&self) -> DomainResult<Vec<CombinePercentile>>;
            async fn find_by_position(&self, position: &str) -> DomainResult<Vec<CombinePercentile>>;
            async fn find_by_position_and_measurement(
                &self,
                position: &str,
                measurement: &str,
            ) -> DomainResult<Option<CombinePercentile>>;
            async fn upsert(&self, percentile: &CombinePercentile) -> DomainResult<CombinePercentile>;
            async fn delete_all(&self) -> DomainResult<u64>;
            async fn delete(&self, id: uuid::Uuid) -> DomainResult<()>;
        }
    }

    #[tokio::test]
    async fn test_calculate_ras_full_measurements() {
        let mut mock_repo = MockCombinePercentileRepo::new();

        // Mock returns percentile data for every measurement
        mock_repo
            .expect_find_by_position_and_measurement()
            .returning(|position, measurement| {
                let percentile = match measurement {
                    "height" => Some(make_percentile(position, "height", 70.0, 73.0, 76.0)),
                    "weight" => Some(make_percentile(position, "weight", 185.0, 210.0, 235.0)),
                    "forty_yard_dash" => Some(make_percentile(position, "forty_yard_dash", 4.35, 4.48, 4.62)),
                    "vertical_jump" => Some(make_percentile(position, "vertical_jump", 28.0, 36.0, 41.0)),
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

        let service = RasScoringService::new(Arc::new(mock_repo));

        let player = crate::models::Player::new(
            "Test".to_string(),
            "Player".to_string(),
            Position::WR,
            2026,
        )
        .unwrap()
        .with_physical_stats(73, 210)
        .unwrap();

        let combine = CombineResults::new(player.id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.45)
            .unwrap()
            .with_vertical_jump(36.0)
            .unwrap()
            .with_broad_jump(120)
            .unwrap()
            .with_three_cone_drill(7.0)
            .unwrap()
            .with_twenty_yard_shuttle(4.3)
            .unwrap()
            .with_bench_press(16)
            .unwrap()
            .with_ten_yard_split(1.55)
            .unwrap()
            .with_twenty_yard_split(2.6)
            .unwrap();

        let ras = service.calculate_ras(&player, &combine).await;

        // With 10 measurements (height, weight + 8 combine), should have an overall score
        assert!(
            ras.overall_score.is_some(),
            "Full measurements should produce an overall score, got explanation: {:?}",
            ras.explanation
        );
        let overall = ras.overall_score.unwrap();
        assert!(
            overall >= 0.0 && overall <= 10.0,
            "Overall score should be between 0 and 10, got {}",
            overall
        );
        assert!(
            ras.measurements_used >= RasScore::MIN_MEASUREMENTS,
            "Should have at least {} measurements, got {}",
            RasScore::MIN_MEASUREMENTS,
            ras.measurements_used
        );
        assert!(ras.explanation.is_none(), "No explanation needed when score is produced");
    }

    #[tokio::test]
    async fn test_calculate_ras_insufficient_measurements() {
        let mut mock_repo = MockCombinePercentileRepo::new();

        // Mock returns percentile data for only one measurement
        mock_repo
            .expect_find_by_position_and_measurement()
            .returning(|position, measurement| {
                if measurement == "forty_yard_dash" {
                    Ok(Some(make_percentile(
                        position,
                        "forty_yard_dash",
                        4.35,
                        4.48,
                        4.62,
                    )))
                } else {
                    Ok(None)
                }
            });

        let service = RasScoringService::new(Arc::new(mock_repo));

        // Player without height/weight so those won't score either
        let player = crate::models::Player::new(
            "Minimal".to_string(),
            "Data".to_string(),
            Position::WR,
            2026,
        )
        .unwrap();

        // Only provide forty_yard_dash
        let combine = CombineResults::new(player.id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.5)
            .unwrap();

        let ras = service.calculate_ras(&player, &combine).await;

        assert!(
            ras.overall_score.is_none(),
            "Insufficient measurements should yield no overall score"
        );
        assert!(
            ras.measurements_used < RasScore::MIN_MEASUREMENTS,
            "Should have fewer than {} measurements, got {}",
            RasScore::MIN_MEASUREMENTS,
            ras.measurements_used
        );
        let explanation = ras.explanation.as_ref().expect("Should have an explanation");
        assert!(
            explanation.contains("Insufficient measurements"),
            "Explanation should mention insufficient measurements, got: {}",
            explanation
        );
    }
}
