use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use domain::models::{CombineResults, CombineSource, Player};
use domain::repositories::{CombineResultsRepository, PlayerRepository};

use crate::position_mapper::map_position;
use crate::rankings_loader::normalize_name;

#[derive(Debug, Deserialize, Serialize)]
pub struct CombineFileData {
    pub meta: CombineFileMeta,
    pub combine_results: Vec<CombineFileEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CombineFileMeta {
    pub source: String,
    pub year: i32,
}

#[derive(Debug, Deserialize, Serialize)]
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
    pub skipped_no_data: usize,
    pub player_not_found: usize,
    pub players_discovered: usize,
    pub errors: Vec<String>,
}

impl CombineLoadStats {
    pub fn print_summary(&self) {
        println!("\nCombine Load Summary:");
        println!("  Loaded:              {}", self.loaded);
        println!("  Skipped (exists):    {}", self.skipped);
        println!("  Skipped (no data):   {}", self.skipped_no_data);
        println!("  Player not found:    {}", self.player_not_found);
        println!("  Players discovered:  {}", self.players_discovered);
        if !self.errors.is_empty() {
            println!("  Errors: {}", self.errors.len());
            for err in &self.errors {
                println!("    - {}", err);
            }
        }
    }
}

pub fn parse_combine_json(json: &str) -> Result<CombineFileData> {
    let data: CombineFileData = serde_json::from_str(json)?;
    Ok(data)
}

pub fn parse_combine_file(path: &str) -> Result<CombineFileData> {
    let json = std::fs::read_to_string(path)?;
    parse_combine_json(&json)
}

pub fn entry_has_any_measurement(entry: &CombineFileEntry) -> bool {
    entry.forty_yard_dash.is_some()
        || entry.bench_press.is_some()
        || entry.vertical_jump.is_some()
        || entry.broad_jump.is_some()
        || entry.three_cone_drill.is_some()
        || entry.twenty_yard_shuttle.is_some()
        || entry.arm_length.is_some()
        || entry.hand_size.is_some()
        || entry.wingspan.is_some()
        || entry.ten_yard_split.is_some()
        || entry.twenty_yard_split.is_some()
}

pub async fn load_combine_data(
    data: &CombineFileData,
    player_repo: &dyn PlayerRepository,
    combine_repo: &dyn CombineResultsRepository,
) -> Result<CombineLoadStats> {
    let mut loaded = 0;
    let mut skipped = 0;
    let mut skipped_no_data = 0;
    let mut player_not_found = 0;
    let mut players_discovered = 0;
    let mut errors: Vec<String> = Vec::new();

    // Load all players and build a normalized name lookup map
    let all_players = player_repo
        .find_all()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load players: {}", e))?;

    let mut player_map: HashMap<(String, String), Player> = all_players
        .into_iter()
        .map(|p| {
            (
                (normalize_name(&p.first_name), normalize_name(&p.last_name)),
                p,
            )
        })
        .collect();

    for entry in &data.combine_results {
        // Skip entries where every measurement is null
        if !entry_has_any_measurement(entry) {
            skipped_no_data += 1;
            continue;
        }

        // Find player by normalized name match
        let lookup_key = (
            normalize_name(&entry.first_name),
            normalize_name(&entry.last_name),
        );

        let player_id = if let Some(existing) = player_map.get(&lookup_key) {
            existing.id
        } else {
            // Auto-discover: create a new player from combine entry
            let position = match map_position(&entry.position) {
                Ok(p) => p,
                Err(e) => {
                    let msg = format!(
                        "Skipping {} {} - unknown position '{}': {}",
                        entry.first_name, entry.last_name, entry.position, e
                    );
                    tracing::warn!("{}", msg);
                    player_not_found += 1;
                    errors.push(msg);
                    continue;
                }
            };

            let new_player = Player::new(
                entry.first_name.clone(),
                entry.last_name.clone(),
                position,
                entry.year,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create player {} {}: {}",
                    entry.first_name,
                    entry.last_name,
                    e
                )
            })?;

            let player_id = new_player.id;
            player_repo.create(&new_player).await.map_err(|e| {
                anyhow::anyhow!(
                    "Failed to insert player {} {}: {}",
                    entry.first_name,
                    entry.last_name,
                    e
                )
            })?;

            println!(
                "  New prospect discovered from combine: {} {} ({}) [created]",
                entry.first_name, entry.last_name, entry.position
            );
            players_discovered += 1;

            // Add to map to avoid duplicates
            player_map.insert(lookup_key, new_player);

            player_id
        };

        // Check if combine results already exist for this player/year/source
        let existing = combine_repo
            .find_by_player_year_source(player_id, entry.year, &entry.source)
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
        let mut results = match CombineResults::new(player_id, entry.year) {
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
                    errors.push(format!(
                        "{} {}: forty_yard_dash: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.bench_press {
            results = match results.with_bench_press(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: bench_press: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.vertical_jump {
            results = match results.with_vertical_jump(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: vertical_jump: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.broad_jump {
            results = match results.with_broad_jump(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: broad_jump: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.three_cone_drill {
            results = match results.with_three_cone_drill(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: three_cone_drill: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.twenty_yard_shuttle {
            results = match results.with_twenty_yard_shuttle(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: twenty_yard_shuttle: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.arm_length {
            results = match results.with_arm_length(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: arm_length: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.hand_size {
            results = match results.with_hand_size(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: hand_size: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.wingspan {
            results = match results.with_wingspan(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: wingspan: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.ten_yard_split {
            results = match results.with_ten_yard_split(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: ten_yard_split: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
        }
        if let Some(v) = entry.twenty_yard_split {
            results = match results.with_twenty_yard_split(v) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!(
                        "{} {}: twenty_yard_split: {}",
                        entry.first_name, entry.last_name, e
                    ));
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
        skipped_no_data,
        player_not_found,
        players_discovered,
        errors,
    })
}

pub fn load_combine_data_dry_run(data: &CombineFileData) -> Result<CombineLoadStats> {
    let mut valid = 0;
    let mut skipped_no_data = 0;
    let mut errors: Vec<String> = Vec::new();

    for entry in &data.combine_results {
        // Check if entry has any measurements
        if !entry_has_any_measurement(entry) {
            skipped_no_data += 1;
            continue;
        }

        // Validate source string parses to a valid CombineSource
        if let Err(e) = entry.source.parse::<CombineSource>() {
            errors.push(format!(
                "Invalid source '{}' for {} {}: {}",
                entry.source, entry.first_name, entry.last_name, e
            ));
            continue;
        }

        valid += 1;
    }

    println!("\nDry Run Summary:");
    println!("  Valid entries:        {}", valid);
    println!("  Skipped (no data):   {}", skipped_no_data);
    if !errors.is_empty() {
        println!("  Errors:              {}", errors.len());
        for err in &errors {
            println!("    - {}", err);
        }
    }

    Ok(CombineLoadStats {
        loaded: valid,
        skipped: 0,
        skipped_no_data,
        player_not_found: 0,
        players_discovered: 0,
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
