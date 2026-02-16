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
fn map_position_for_percentile(position: &crate::models::Position) -> String {
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
}
