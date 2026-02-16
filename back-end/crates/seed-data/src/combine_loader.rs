use anyhow::Result;
use serde::Deserialize;

use domain::models::{CombineResults, CombineSource};
use domain::repositories::{CombineResultsRepository, PlayerRepository};

#[derive(Debug, Deserialize)]
pub struct CombineFileData {
    pub meta: CombineFileMeta,
    pub combine_results: Vec<CombineFileEntry>,
}

#[derive(Debug, Deserialize)]
pub struct CombineFileMeta {
    pub source: String,
    pub year: i32,
}

#[derive(Debug, Deserialize)]
pub struct CombineFileEntry {
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

pub struct CombineLoadStats {
    pub loaded: usize,
    pub skipped: usize,
    pub player_not_found: usize,
    pub errors: Vec<String>,
}

pub fn parse_combine_json(json: &str) -> Result<CombineFileData> {
    let data: CombineFileData = serde_json::from_str(json)?;
    Ok(data)
}

pub async fn load_combine_data(
    data: &CombineFileData,
    player_repo: &dyn PlayerRepository,
    combine_repo: &dyn CombineResultsRepository,
) -> Result<CombineLoadStats> {
    let mut loaded = 0;
    let mut skipped = 0;
    let mut player_not_found = 0;
    let mut errors: Vec<String> = Vec::new();

    // Load all players for name matching
    let all_players = player_repo.find_all().await.map_err(|e| {
        anyhow::anyhow!("Failed to load players: {}", e)
    })?;

    for entry in &data.combine_results {
        // Find player by name match
        let player = all_players.iter().find(|p| {
            p.first_name.eq_ignore_ascii_case(&entry.first_name)
                && p.last_name.eq_ignore_ascii_case(&entry.last_name)
        });

        let player = match player {
            Some(p) => p,
            None => {
                player_not_found += 1;
                continue;
            }
        };

        // Check if combine results already exist for this player/year/source
        let existing = combine_repo
            .find_by_player_year_source(player.id, entry.year, &entry.source)
            .await;

        if let Ok(Some(_)) = existing {
            skipped += 1;
            continue;
        }

        // Parse source
        let source: CombineSource = match entry.source.parse() {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!(
                    "Invalid source '{}' for {} {}: {}",
                    entry.source, entry.first_name, entry.last_name, e
                ));
                continue;
            }
        };

        // Build combine results
        let mut results = match CombineResults::new(player.id, entry.year) {
            Ok(r) => r.with_source(source),
            Err(e) => {
                errors.push(format!(
                    "Failed to create results for {} {}: {}",
                    entry.first_name, entry.last_name, e
                ));
                continue;
            }
        };

        // Apply measurements
        if let Some(v) = entry.forty_yard_dash {
            results = match results.with_forty_yard_dash(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: forty_yard_dash: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.bench_press {
            results = match results.with_bench_press(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: bench_press: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.vertical_jump {
            results = match results.with_vertical_jump(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: vertical_jump: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.broad_jump {
            results = match results.with_broad_jump(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: broad_jump: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.three_cone_drill {
            results = match results.with_three_cone_drill(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: three_cone_drill: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.twenty_yard_shuttle {
            results = match results.with_twenty_yard_shuttle(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: twenty_yard_shuttle: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.arm_length {
            results = match results.with_arm_length(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: arm_length: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.hand_size {
            results = match results.with_hand_size(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: hand_size: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.wingspan {
            results = match results.with_wingspan(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: wingspan: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.ten_yard_split {
            results = match results.with_ten_yard_split(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: ten_yard_split: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }
        if let Some(v) = entry.twenty_yard_split {
            results = match results.with_twenty_yard_split(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("{} {}: twenty_yard_split: {}", entry.first_name, entry.last_name, e));
                    continue;
                }
            };
        }

        match combine_repo.create(&results).await {
            Ok(_) => loaded += 1,
            Err(e) => {
                errors.push(format!(
                    "Failed to save {} {}: {}",
                    entry.first_name, entry.last_name, e
                ));
            }
        }
    }

    Ok(CombineLoadStats {
        loaded,
        skipped,
        player_not_found,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_combine_json() {
        let json = r#"{
            "meta": { "source": "mock_generator", "year": 2026, "description": "test", "generated_at": "2026-02-16", "player_count": 1, "entry_count": 1 },
            "combine_results": [
                {
                    "first_name": "Cam",
                    "last_name": "Ward",
                    "position": "QB",
                    "source": "combine",
                    "year": 2026,
                    "forty_yard_dash": 4.72,
                    "bench_press": 18,
                    "vertical_jump": 32.0,
                    "broad_jump": 108,
                    "three_cone_drill": null,
                    "twenty_yard_shuttle": null,
                    "arm_length": 32.5,
                    "hand_size": 9.75,
                    "wingspan": 77.5,
                    "ten_yard_split": 1.65,
                    "twenty_yard_split": 2.72
                }
            ]
        }"#;

        let data = parse_combine_json(json).unwrap();
        assert_eq!(data.combine_results.len(), 1);
        assert_eq!(data.combine_results[0].first_name, "Cam");
        assert_eq!(data.combine_results[0].forty_yard_dash, Some(4.72));
        assert!(data.combine_results[0].three_cone_drill.is_none());
    }
}
