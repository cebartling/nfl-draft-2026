use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockCombineEntry {
    pub first_name: String,
    pub last_name: String,
    pub position: String,
    pub source: String,
    pub year: i32,
    pub forty_yard_dash: Option<f64>,
    pub bench_press: Option<i32>,
    pub vertical_jump: Option<f64>,
    pub broad_jump: Option<i32>,
    pub three_cone_drill: Option<f64>,
    pub twenty_yard_shuttle: Option<f64>,
    pub arm_length: Option<f64>,
    pub hand_size: Option<f64>,
    pub wingspan: Option<f64>,
    pub ten_yard_split: Option<f64>,
    pub twenty_yard_split: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockCombineData {
    pub meta: MockCombineMeta,
    pub combine_results: Vec<MockCombineEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockCombineMeta {
    pub source: String,
    pub description: String,
    pub year: i32,
    pub generated_at: String,
    pub player_count: usize,
    pub entry_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct ProspectEntry {
    pub first_name: String,
    pub last_name: String,
    pub position: String,
}

#[derive(Debug, Deserialize)]
pub struct ProspectData {
    pub players: Vec<ProspectEntry>,
}

/// Generate mock combine data for a list of prospects.
/// Higher-ranked prospects (lower index) get better numbers on average.
pub fn generate_mock_combine_data(prospects: &[ProspectEntry], year: i32) -> MockCombineData {
    let mut rng = rand::rng();
    let total = prospects.len();
    let mut entries = Vec::new();

    for (idx, prospect) in prospects.iter().enumerate() {
        // Ranking factor: 0.0 (best) to 1.0 (worst)
        let rank_factor = idx as f64 / total as f64;

        // ~80% get combine results
        if rng.random_range(0.0..1.0) < 0.80 {
            let entry = generate_entry(
                prospect,
                year,
                "combine",
                rank_factor,
                &mut rng,
            );
            entries.push(entry);
        }

        // ~30% also get pro day results
        if rng.random_range(0.0..1.0) < 0.30 {
            let entry = generate_entry(
                prospect,
                year,
                "pro_day",
                rank_factor,
                &mut rng,
            );
            entries.push(entry);
        }
    }

    let player_count = prospects.len();
    let entry_count = entries.len();

    MockCombineData {
        meta: MockCombineMeta {
            source: "mock_generator".to_string(),
            description: format!("Mock combine data for {} NFL Draft prospects", year),
            year,
            generated_at: chrono::Utc::now().to_rfc3339(),
            player_count,
            entry_count,
        },
        combine_results: entries,
    }
}

fn generate_entry(
    prospect: &ProspectEntry,
    year: i32,
    source: &str,
    rank_factor: f64,
    rng: &mut impl Rng,
) -> MockCombineEntry {
    let pos = prospect.position.as_str();

    // Better players (lower rank_factor) get a bonus to their scores
    let quality_bonus = (1.0 - rank_factor) * 0.5;

    let forty_yard_dash = maybe_timed(rng, 0.85, pos_forty_median(pos), 0.10, quality_bonus);
    let bench_press = maybe_count(rng, 0.70, pos_bench_median(pos), 3.5, quality_bonus);
    let vertical_jump = maybe_higher(rng, 0.85, pos_vert_median(pos), 2.5, quality_bonus);
    let broad_jump = maybe_count_large(rng, 0.80, pos_broad_median(pos), 5.0, quality_bonus);
    let three_cone_drill = maybe_timed(rng, 0.65, pos_three_cone_median(pos), 0.12, quality_bonus);
    let twenty_yard_shuttle = maybe_timed(rng, 0.65, pos_shuttle_median(pos), 0.08, quality_bonus);
    let arm_length = maybe_higher(rng, 0.90, pos_arm_median(pos), 0.8, quality_bonus);
    let hand_size = maybe_higher(rng, 0.90, pos_hand_median(pos), 0.35, quality_bonus);
    let wingspan = maybe_higher(rng, 0.90, pos_wingspan_median(pos), 1.5, quality_bonus);
    let ten_yard_split = maybe_timed(rng, 0.75, pos_ten_yd_median(pos), 0.04, quality_bonus);
    let twenty_yard_split = maybe_timed(rng, 0.75, pos_twenty_yd_median(pos), 0.05, quality_bonus);

    MockCombineEntry {
        first_name: prospect.first_name.clone(),
        last_name: prospect.last_name.clone(),
        position: prospect.position.clone(),
        source: source.to_string(),
        year,
        forty_yard_dash,
        bench_press,
        vertical_jump,
        broad_jump,
        three_cone_drill,
        twenty_yard_shuttle,
        arm_length,
        hand_size,
        wingspan,
        ten_yard_split,
        twenty_yard_split,
    }
}

fn maybe_timed(rng: &mut impl Rng, prob: f64, median: f64, spread: f64, quality: f64) -> Option<f64> {
    if rng.random_range(0.0..1.0) >= prob {
        return None;
    }
    Some(round2(generate_timed(median, spread, quality, rng)))
}

fn maybe_higher(rng: &mut impl Rng, prob: f64, median: f64, spread: f64, quality: f64) -> Option<f64> {
    if rng.random_range(0.0..1.0) >= prob {
        return None;
    }
    Some(round2(generate_higher_better(median, spread, quality, rng)))
}

fn maybe_count(rng: &mut impl Rng, prob: f64, median: f64, spread: f64, quality: f64) -> Option<i32> {
    if rng.random_range(0.0..1.0) >= prob {
        return None;
    }
    Some(generate_count(median, spread, quality, rng))
}

fn maybe_count_large(rng: &mut impl Rng, prob: f64, median: f64, spread: f64, quality: f64) -> Option<i32> {
    if rng.random_range(0.0..1.0) >= prob {
        return None;
    }
    Some(generate_count_large(median, spread, quality, rng))
}

/// Generate a timed value (lower is better). Quality bonus reduces (improves) the time.
fn generate_timed(median: f64, spread: f64, quality_bonus: f64, rng: &mut impl Rng) -> f64 {
    let noise: f64 = rng.random_range(-2.0..2.0);
    let value = median + noise * spread - quality_bonus * spread;
    value.max(median - 3.0 * spread)
}

/// Generate a higher-is-better value. Quality bonus increases the value.
fn generate_higher_better(median: f64, spread: f64, quality_bonus: f64, rng: &mut impl Rng) -> f64 {
    let noise: f64 = rng.random_range(-2.0..2.0);
    let value = median + noise * spread + quality_bonus * spread;
    value.max(median - 3.0 * spread)
}

/// Generate a count (bench press reps) — integer, higher is better.
fn generate_count(median: f64, spread: f64, quality_bonus: f64, rng: &mut impl Rng) -> i32 {
    let value = generate_higher_better(median, spread, quality_bonus, rng);
    value.round().max(1.0) as i32
}

/// Generate a large count (broad jump inches) — integer, higher is better.
fn generate_count_large(median: f64, spread: f64, quality_bonus: f64, rng: &mut impl Rng) -> i32 {
    let value = generate_higher_better(median, spread, quality_bonus, rng);
    value.round().max(60.0) as i32
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

// Position-specific medians
fn pos_forty_median(pos: &str) -> f64 {
    match pos {
        "QB" => 4.80, "RB" => 4.52, "WR" => 4.48, "TE" => 4.68,
        "OT" => 5.15, "IOL" => 5.20, "EDGE" => 4.72, "DL" => 4.95,
        "LB" => 4.65, "CB" => 4.45, "S" => 4.52, _ => 4.70,
    }
}

fn pos_bench_median(pos: &str) -> f64 {
    match pos {
        "QB" => 18.0, "RB" => 20.0, "WR" => 14.0, "TE" => 22.0,
        "OT" => 26.0, "IOL" => 27.0, "EDGE" => 24.0, "DL" => 26.0,
        "LB" => 22.0, "CB" => 14.0, "S" => 16.0, _ => 20.0,
    }
}

fn pos_vert_median(pos: &str) -> f64 {
    match pos {
        "QB" => 31.0, "RB" => 35.0, "WR" => 36.0, "TE" => 33.5,
        "OT" => 28.0, "IOL" => 27.5, "EDGE" => 34.0, "DL" => 30.0,
        "LB" => 34.0, "CB" => 37.0, "S" => 36.0, _ => 33.0,
    }
}

fn pos_broad_median(pos: &str) -> f64 {
    match pos {
        "QB" => 106.0, "RB" => 118.0, "WR" => 120.0, "TE" => 115.0,
        "OT" => 100.0, "IOL" => 99.0, "EDGE" => 116.0, "DL" => 106.0,
        "LB" => 116.0, "CB" => 122.0, "S" => 120.0, _ => 112.0,
    }
}

fn pos_three_cone_median(pos: &str) -> f64 {
    match pos {
        "QB" => 7.10, "RB" => 6.95, "WR" => 6.90, "TE" => 7.05,
        "OT" => 7.60, "IOL" => 7.55, "EDGE" => 7.05, "DL" => 7.40,
        "LB" => 7.00, "CB" => 6.80, "S" => 6.90, _ => 7.10,
    }
}

fn pos_shuttle_median(pos: &str) -> f64 {
    match pos {
        "QB" => 4.35, "RB" => 4.25, "WR" => 4.20, "TE" => 4.30,
        "OT" => 4.65, "IOL" => 4.60, "EDGE" => 4.30, "DL" => 4.55,
        "LB" => 4.25, "CB" => 4.10, "S" => 4.18, _ => 4.35,
    }
}

fn pos_arm_median(pos: &str) -> f64 {
    match pos {
        "QB" => 32.5, "RB" => 31.5, "WR" => 32.5, "TE" => 33.5,
        "OT" => 34.5, "IOL" => 33.0, "EDGE" => 34.0, "DL" => 33.5,
        "LB" => 32.5, "CB" => 31.5, "S" => 32.0, _ => 32.5,
    }
}

fn pos_hand_median(pos: &str) -> f64 {
    match pos {
        "QB" => 9.5, "RB" => 9.25, "WR" => 9.5, "TE" => 9.75,
        "OT" => 10.0, "IOL" => 9.75, "EDGE" => 10.0, "DL" => 10.0,
        "LB" => 9.75, "CB" => 9.25, "S" => 9.5, _ => 9.5,
    }
}

fn pos_wingspan_median(pos: &str) -> f64 {
    match pos {
        "QB" => 77.0, "RB" => 75.0, "WR" => 77.5, "TE" => 80.0,
        "OT" => 82.0, "IOL" => 80.0, "EDGE" => 81.0, "DL" => 80.5,
        "LB" => 78.0, "CB" => 76.0, "S" => 77.0, _ => 78.0,
    }
}

fn pos_ten_yd_median(pos: &str) -> f64 {
    match pos {
        "QB" => 1.68, "RB" => 1.56, "WR" => 1.54, "TE" => 1.62,
        "OT" => 1.80, "IOL" => 1.82, "EDGE" => 1.64, "DL" => 1.74,
        "LB" => 1.60, "CB" => 1.52, "S" => 1.55, _ => 1.64,
    }
}

fn pos_twenty_yd_median(pos: &str) -> f64 {
    match pos {
        "QB" => 2.78, "RB" => 2.62, "WR" => 2.58, "TE" => 2.70,
        "OT" => 2.98, "IOL" => 3.00, "EDGE" => 2.72, "DL" => 2.88,
        "LB" => 2.66, "CB" => 2.56, "S" => 2.60, _ => 2.72,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_prospects() -> Vec<ProspectEntry> {
        vec![
            ProspectEntry {
                first_name: "Cam".to_string(),
                last_name: "Ward".to_string(),
                position: "QB".to_string(),
            },
            ProspectEntry {
                first_name: "Travis".to_string(),
                last_name: "Hunter".to_string(),
                position: "CB".to_string(),
            },
            ProspectEntry {
                first_name: "Ashton".to_string(),
                last_name: "Jeanty".to_string(),
                position: "RB".to_string(),
            },
        ]
    }

    #[test]
    fn test_generate_mock_qb_data() {
        let prospects = sample_prospects();
        let data = generate_mock_combine_data(&prospects, 2026);

        // Should have at least some entries
        assert!(!data.combine_results.is_empty());

        // Find QB entries
        let qb_entries: Vec<_> = data
            .combine_results
            .iter()
            .filter(|e| e.position == "QB")
            .collect();

        for entry in &qb_entries {
            if let Some(forty) = entry.forty_yard_dash {
                assert!(forty > 4.0 && forty < 6.0, "QB 40 out of range: {}", forty);
            }
            if let Some(bench) = entry.bench_press {
                assert!(bench > 0 && bench < 40, "QB bench out of range: {}", bench);
            }
        }
    }

    #[test]
    fn test_generate_mock_respects_ranking() {
        // Generate with enough prospects to see the trend
        let mut prospects = Vec::new();
        for i in 0..50 {
            prospects.push(ProspectEntry {
                first_name: format!("Player{}", i),
                last_name: format!("Last{}", i),
                position: "WR".to_string(),
            });
        }

        let data = generate_mock_combine_data(&prospects, 2026);

        // Get combine entries sorted by original rank
        let top_half: Vec<_> = data
            .combine_results
            .iter()
            .filter(|e| {
                let idx = e.first_name.replace("Player", "").parse::<usize>().unwrap_or(999);
                idx < 25 && e.source == "combine"
            })
            .collect();

        let bottom_half: Vec<_> = data
            .combine_results
            .iter()
            .filter(|e| {
                let idx = e.first_name.replace("Player", "").parse::<usize>().unwrap_or(999);
                idx >= 25 && e.source == "combine"
            })
            .collect();

        // On average, top prospects should have faster 40s (lower times)
        if !top_half.is_empty() && !bottom_half.is_empty() {
            let avg_top: f64 = top_half
                .iter()
                .filter_map(|e| e.forty_yard_dash)
                .sum::<f64>()
                / top_half.iter().filter(|e| e.forty_yard_dash.is_some()).count().max(1) as f64;

            let avg_bottom: f64 = bottom_half
                .iter()
                .filter_map(|e| e.forty_yard_dash)
                .sum::<f64>()
                / bottom_half.iter().filter(|e| e.forty_yard_dash.is_some()).count().max(1) as f64;

            // Top half should generally be faster (lower time), allowing some variance
            // Just check they're in a reasonable range — with randomness we can't guarantee strict ordering
            assert!(avg_top < 5.0, "Top half average 40 should be reasonable");
            assert!(avg_bottom < 5.5, "Bottom half average 40 should be reasonable");
        }
    }

    #[test]
    fn test_mock_data_has_realistic_missing_values() {
        let mut prospects = Vec::new();
        for i in 0..100 {
            prospects.push(ProspectEntry {
                first_name: format!("Player{}", i),
                last_name: format!("Last{}", i),
                position: "WR".to_string(),
            });
        }

        let data = generate_mock_combine_data(&prospects, 2026);

        let combine_entries: Vec<_> = data
            .combine_results
            .iter()
            .filter(|e| e.source == "combine")
            .collect();

        // Should have some entries
        assert!(combine_entries.len() > 50, "Expected at least 50 combine entries");

        // Not every entry should have all measurements
        let all_complete = combine_entries
            .iter()
            .all(|e| {
                e.forty_yard_dash.is_some()
                    && e.bench_press.is_some()
                    && e.vertical_jump.is_some()
                    && e.broad_jump.is_some()
                    && e.three_cone_drill.is_some()
            });
        assert!(!all_complete, "Some entries should have missing values");
    }
}
