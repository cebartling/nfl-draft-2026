use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileEntry {
    pub position: String,
    pub measurement: String,
    pub sample_size: i32,
    pub min_value: f64,
    pub p10: f64,
    pub p20: f64,
    pub p30: f64,
    pub p40: f64,
    pub p50: f64,
    pub p60: f64,
    pub p70: f64,
    pub p80: f64,
    pub p90: f64,
    pub max_value: f64,
    pub years_start: i32,
    pub years_end: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PercentileData {
    pub meta: PercentileMeta,
    pub percentiles: Vec<PercentileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PercentileMeta {
    pub source: String,
    pub description: String,
    pub generated_at: String,
}

/// Position-specific measurement baselines (median, stddev-ish spread).
/// Based on published NFL Combine averages 2000-2025.
struct MeasurementBaseline {
    measurement: &'static str,
    /// (position, median, spread) — spread is approx 1 standard deviation
    position_data: Vec<(&'static str, f64, f64)>,
    /// true if lower is better (timed events)
    lower_is_better: bool,
    sample_size: i32,
}

/// Generate template percentile data from known NFL averages.
pub fn generate_template_percentiles() -> PercentileData {
    let baselines = get_baselines();
    let mut entries = Vec::new();

    for baseline in &baselines {
        for &(position, median, spread) in &baseline.position_data {
            let entry = generate_entry(
                position,
                baseline.measurement,
                median,
                spread,
                baseline.lower_is_better,
                baseline.sample_size,
            );
            entries.push(entry);
        }
    }

    PercentileData {
        meta: PercentileMeta {
            source: "template".to_string(),
            description: "Template percentile data based on published NFL Combine averages (2000-2025)".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        },
        percentiles: entries,
    }
}

fn generate_entry(
    position: &str,
    measurement: &str,
    median: f64,
    spread: f64,
    lower_is_better: bool,
    sample_size: i32,
) -> PercentileEntry {
    // Generate percentile breakpoints using normal-ish distribution
    // For lower_is_better (timed drills), lower values = higher percentile
    let (min_val, p10, p20, p30, p40, p50, p60, p70, p80, p90, max_val) = if lower_is_better {
        // Timed events: p90 is fast (low), p10 is slow (high)
        (
            round2(median - 2.5 * spread), // min (fastest)
            round2(median - 1.5 * spread), // p10 (slow end)
            round2(median - 1.0 * spread), // p20
            round2(median - 0.7 * spread), // p30
            round2(median - 0.4 * spread), // p40
            round2(median),                // p50
            round2(median + 0.4 * spread), // p60
            round2(median + 0.7 * spread), // p70
            round2(median + 1.0 * spread), // p80
            round2(median + 1.5 * spread), // p90 (fast end)
            round2(median + 2.5 * spread), // max (slowest)
        )
    } else {
        // Non-timed: higher is better
        (
            round2(median - 2.5 * spread), // min
            round2(median - 1.5 * spread), // p10
            round2(median - 1.0 * spread), // p20
            round2(median - 0.7 * spread), // p30
            round2(median - 0.4 * spread), // p40
            round2(median),                // p50
            round2(median + 0.4 * spread), // p60
            round2(median + 0.7 * spread), // p70
            round2(median + 1.0 * spread), // p80
            round2(median + 1.5 * spread), // p90
            round2(median + 2.5 * spread), // max
        )
    };

    PercentileEntry {
        position: position.to_string(),
        measurement: measurement.to_string(),
        sample_size,
        min_value: min_val,
        p10,
        p20,
        p30,
        p40,
        p50,
        p60,
        p70,
        p80,
        p90,
        max_value: max_val,
        years_start: 2000,
        years_end: 2025,
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Known NFL Combine averages by position group.
/// Sources: NFL Combine historical data, mockdraftable.com, ras.football
fn get_baselines() -> Vec<MeasurementBaseline> {
    let positions = [
        "QB", "RB", "WR", "TE", "OT", "IOL", "EDGE", "DL", "LB", "CB", "S",
    ];

    // 40-yard dash (seconds) — lower is better
    let forty_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (4.80, 0.12),
                "RB" => (4.52, 0.10),
                "WR" => (4.48, 0.09),
                "TE" => (4.68, 0.10),
                "OT" => (5.15, 0.12),
                "IOL" => (5.20, 0.12),
                "EDGE" => (4.72, 0.11),
                "DL" => (4.95, 0.12),
                "LB" => (4.65, 0.10),
                "CB" => (4.45, 0.08),
                "S" => (4.52, 0.09),
                _ => (4.70, 0.15),
            };
            (pos, median, spread)
        })
        .collect();

    // Bench press (reps) — higher is better
    let bench_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (18.0, 3.0),
                "RB" => (20.0, 3.5),
                "WR" => (14.0, 3.0),
                "TE" => (22.0, 3.5),
                "OT" => (26.0, 4.0),
                "IOL" => (27.0, 4.0),
                "EDGE" => (24.0, 4.0),
                "DL" => (26.0, 4.5),
                "LB" => (22.0, 3.5),
                "CB" => (14.0, 3.0),
                "S" => (16.0, 3.0),
                _ => (20.0, 4.0),
            };
            (pos, median, spread)
        })
        .collect();

    // Vertical jump (inches) — higher is better
    let vert_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (31.0, 2.5),
                "RB" => (35.0, 2.5),
                "WR" => (36.0, 2.5),
                "TE" => (33.5, 2.5),
                "OT" => (28.0, 2.5),
                "IOL" => (27.5, 2.5),
                "EDGE" => (34.0, 2.5),
                "DL" => (30.0, 2.5),
                "LB" => (34.0, 2.5),
                "CB" => (37.0, 2.5),
                "S" => (36.0, 2.5),
                _ => (33.0, 3.0),
            };
            (pos, median, spread)
        })
        .collect();

    // Broad jump (inches) — higher is better
    let broad_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (106.0, 6.0),
                "RB" => (118.0, 5.0),
                "WR" => (120.0, 5.0),
                "TE" => (115.0, 5.0),
                "OT" => (100.0, 6.0),
                "IOL" => (99.0, 6.0),
                "EDGE" => (116.0, 5.0),
                "DL" => (106.0, 6.0),
                "LB" => (116.0, 5.0),
                "CB" => (122.0, 5.0),
                "S" => (120.0, 5.0),
                _ => (112.0, 6.0),
            };
            (pos, median, spread)
        })
        .collect();

    // 3-cone drill (seconds) — lower is better
    let three_cone_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (7.10, 0.15),
                "RB" => (6.95, 0.12),
                "WR" => (6.90, 0.12),
                "TE" => (7.05, 0.12),
                "OT" => (7.60, 0.15),
                "IOL" => (7.55, 0.15),
                "EDGE" => (7.05, 0.12),
                "DL" => (7.40, 0.15),
                "LB" => (7.00, 0.12),
                "CB" => (6.80, 0.10),
                "S" => (6.90, 0.12),
                _ => (7.10, 0.15),
            };
            (pos, median, spread)
        })
        .collect();

    // 20-yard shuttle (seconds) — lower is better
    let shuttle_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (4.35, 0.10),
                "RB" => (4.25, 0.08),
                "WR" => (4.20, 0.08),
                "TE" => (4.30, 0.08),
                "OT" => (4.65, 0.10),
                "IOL" => (4.60, 0.10),
                "EDGE" => (4.30, 0.08),
                "DL" => (4.55, 0.10),
                "LB" => (4.25, 0.08),
                "CB" => (4.10, 0.07),
                "S" => (4.18, 0.08),
                _ => (4.35, 0.10),
            };
            (pos, median, spread)
        })
        .collect();

    // Arm length (inches) — higher is better
    let arm_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (32.5, 0.8),
                "RB" => (31.5, 0.8),
                "WR" => (32.5, 1.0),
                "TE" => (33.5, 0.8),
                "OT" => (34.5, 0.8),
                "IOL" => (33.0, 0.8),
                "EDGE" => (34.0, 0.8),
                "DL" => (33.5, 0.8),
                "LB" => (32.5, 0.8),
                "CB" => (31.5, 0.8),
                "S" => (32.0, 0.8),
                _ => (32.5, 1.0),
            };
            (pos, median, spread)
        })
        .collect();

    // Hand size (inches) — higher is better
    let hand_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (9.5, 0.4),
                "RB" => (9.25, 0.35),
                "WR" => (9.5, 0.4),
                "TE" => (9.75, 0.4),
                "OT" => (10.0, 0.4),
                "IOL" => (9.75, 0.4),
                "EDGE" => (10.0, 0.4),
                "DL" => (10.0, 0.4),
                "LB" => (9.75, 0.4),
                "CB" => (9.25, 0.35),
                "S" => (9.5, 0.35),
                _ => (9.5, 0.4),
            };
            (pos, median, spread)
        })
        .collect();

    // Wingspan (inches) — higher is better
    let wingspan_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (77.0, 1.5),
                "RB" => (75.0, 1.5),
                "WR" => (77.5, 2.0),
                "TE" => (80.0, 1.5),
                "OT" => (82.0, 1.5),
                "IOL" => (80.0, 1.5),
                "EDGE" => (81.0, 1.5),
                "DL" => (80.5, 1.5),
                "LB" => (78.0, 1.5),
                "CB" => (76.0, 1.5),
                "S" => (77.0, 1.5),
                _ => (78.0, 2.0),
            };
            (pos, median, spread)
        })
        .collect();

    // 10-yard split (seconds) — lower is better
    let ten_yd_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (1.68, 0.05),
                "RB" => (1.56, 0.04),
                "WR" => (1.54, 0.04),
                "TE" => (1.62, 0.04),
                "OT" => (1.80, 0.05),
                "IOL" => (1.82, 0.05),
                "EDGE" => (1.64, 0.05),
                "DL" => (1.74, 0.05),
                "LB" => (1.60, 0.04),
                "CB" => (1.52, 0.03),
                "S" => (1.55, 0.04),
                _ => (1.64, 0.06),
            };
            (pos, median, spread)
        })
        .collect();

    // 20-yard split (seconds) — lower is better
    let twenty_yd_data: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|&pos| {
            let (median, spread) = match pos {
                "QB" => (2.78, 0.06),
                "RB" => (2.62, 0.05),
                "WR" => (2.58, 0.05),
                "TE" => (2.70, 0.05),
                "OT" => (2.98, 0.06),
                "IOL" => (3.00, 0.06),
                "EDGE" => (2.72, 0.05),
                "DL" => (2.88, 0.06),
                "LB" => (2.66, 0.05),
                "CB" => (2.56, 0.04),
                "S" => (2.60, 0.05),
                _ => (2.72, 0.07),
            };
            (pos, median, spread)
        })
        .collect();

    vec![
        MeasurementBaseline {
            measurement: "forty_yard_dash",
            position_data: forty_data,
            lower_is_better: true,
            sample_size: 300,
        },
        MeasurementBaseline {
            measurement: "bench_press",
            position_data: bench_data,
            lower_is_better: false,
            sample_size: 250,
        },
        MeasurementBaseline {
            measurement: "vertical_jump",
            position_data: vert_data,
            lower_is_better: false,
            sample_size: 300,
        },
        MeasurementBaseline {
            measurement: "broad_jump",
            position_data: broad_data,
            lower_is_better: false,
            sample_size: 300,
        },
        MeasurementBaseline {
            measurement: "three_cone_drill",
            position_data: three_cone_data,
            lower_is_better: true,
            sample_size: 200,
        },
        MeasurementBaseline {
            measurement: "twenty_yard_shuttle",
            position_data: shuttle_data,
            lower_is_better: true,
            sample_size: 200,
        },
        MeasurementBaseline {
            measurement: "arm_length",
            position_data: arm_data,
            lower_is_better: false,
            sample_size: 350,
        },
        MeasurementBaseline {
            measurement: "hand_size",
            position_data: hand_data,
            lower_is_better: false,
            sample_size: 350,
        },
        MeasurementBaseline {
            measurement: "wingspan",
            position_data: wingspan_data,
            lower_is_better: false,
            sample_size: 350,
        },
        MeasurementBaseline {
            measurement: "ten_yard_split",
            position_data: ten_yd_data,
            lower_is_better: true,
            sample_size: 250,
        },
        MeasurementBaseline {
            measurement: "twenty_yard_split",
            position_data: twenty_yd_data,
            lower_is_better: true,
            sample_size: 250,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_template_percentiles() {
        let data = generate_template_percentiles();
        assert_eq!(data.meta.source, "template");
        // 11 positions x 11 measurements = 121 entries
        assert_eq!(data.percentiles.len(), 121);
    }

    #[test]
    fn test_percentile_ordering() {
        let data = generate_template_percentiles();
        for entry in &data.percentiles {
            assert!(
                entry.min_value <= entry.p10,
                "{} {} min_value {} > p10 {}",
                entry.position, entry.measurement, entry.min_value, entry.p10
            );
            assert!(entry.p10 <= entry.p20, "{} {} p10 > p20", entry.position, entry.measurement);
            assert!(entry.p20 <= entry.p30, "{} {} p20 > p30", entry.position, entry.measurement);
            assert!(entry.p90 <= entry.max_value, "{} {} p90 > max", entry.position, entry.measurement);
        }
    }

    #[test]
    fn test_minimum_sample_size() {
        let data = generate_template_percentiles();
        for entry in &data.percentiles {
            assert!(entry.sample_size >= 20, "{} {} has sample_size < 20", entry.position, entry.measurement);
        }
    }
}
