use anyhow::Result;
use domain::models::Player;
use domain::repositories::PlayerRepository;
use serde::Deserialize;

use crate::position_mapper;

#[derive(Debug, Deserialize)]
pub struct PlayerData {
    pub meta: MetaData,
    pub players: Vec<PlayerEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MetaData {
    pub version: String,
    pub draft_year: i32,
    pub last_updated: String,
    pub sources: Vec<String>,
    pub total_players: usize,
}

#[derive(Debug, Deserialize)]
pub struct PlayerEntry {
    pub first_name: String,
    pub last_name: String,
    pub position: String,
    pub college: Option<String>,
    pub height_inches: Option<i32>,
    pub weight_pounds: Option<i32>,
    #[allow(dead_code)]
    pub notes: Option<String>,
}

impl PlayerEntry {
    pub fn to_domain(&self, draft_year: i32) -> Result<Player> {
        let position = position_mapper::map_position(&self.position)?;

        let mut player = Player::new(
            self.first_name.clone(),
            self.last_name.clone(),
            position,
            draft_year,
        )?;

        if let Some(ref college) = self.college {
            player = player.with_college(college.clone())?;
        }

        if let (Some(height), Some(weight)) = (self.height_inches, self.weight_pounds) {
            player = player.with_physical_stats(height, weight)?;
        }

        Ok(player)
    }
}

#[derive(Debug, Default)]
pub struct LoadStats {
    pub success: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl LoadStats {
    pub fn print_summary(&self) {
        println!("\nLoad Summary:");
        println!("  Succeeded: {}", self.success);
        println!("  Skipped:   {}", self.skipped);
        println!("  Errors:    {}", self.errors.len());
        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }
    }
}

pub fn parse_player_file(file_path: &str) -> Result<PlayerData> {
    let content = std::fs::read_to_string(file_path)?;
    let data: PlayerData = serde_json::from_str(&content)?;
    Ok(data)
}

/// Maximum number of consecutive failures before aborting.
const MAX_CONSECUTIVE_FAILURES: usize = 5;

pub fn load_players_dry_run(data: &PlayerData) -> Result<LoadStats> {
    let mut stats = LoadStats::default();
    let mut consecutive_failures: usize = 0;

    for entry in &data.players {
        let full_name = format!("{} {}", entry.first_name, entry.last_name);

        match entry.to_domain(data.meta.draft_year) {
            Ok(_) => {
                println!("[DRY RUN] Would insert: {} ({})", full_name, entry.position);
                stats.success += 1;
                consecutive_failures = 0;
            }
            Err(e) => {
                let msg = format!("Validation failed for {}: {}", full_name, e);
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                consecutive_failures += 1;

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    let abort_msg = format!(
                        "Aborting: {} consecutive failures detected. This may indicate a systematic problem (e.g., schema mismatch).",
                        consecutive_failures
                    );
                    tracing::error!("{}", abort_msg);
                    stats.errors.push(abort_msg);
                    break;
                }
            }
        }
    }

    Ok(stats)
}

pub async fn load_players(data: &PlayerData, repo: &dyn PlayerRepository) -> Result<LoadStats> {
    let mut stats = LoadStats::default();
    let mut consecutive_failures: usize = 0;

    for entry in &data.players {
        let full_name = format!("{} {}", entry.first_name, entry.last_name);

        match entry.to_domain(data.meta.draft_year) {
            Ok(player) => match repo.create(&player).await {
                Ok(_) => {
                    tracing::info!("Inserted: {} ({})", full_name, entry.position);
                    stats.success += 1;
                    consecutive_failures = 0;
                }
                Err(e) => {
                    let msg = format!("Failed to insert {}: {}", full_name, e);
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                    consecutive_failures += 1;
                }
            },
            Err(e) => {
                let msg = format!("Validation failed for {}: {}", full_name, e);
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                consecutive_failures += 1;
            }
        }

        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            let abort_msg = format!(
                "Aborting: {} consecutive failures detected. This may indicate a systematic problem (e.g., database down, schema mismatch).",
                consecutive_failures
            );
            tracing::error!("{}", abort_msg);
            stats.errors.push(abort_msg);
            break;
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> &'static str {
        r#"{
            "meta": {
                "version": "1.0.0",
                "draft_year": 2026,
                "last_updated": "2026-02-04",
                "sources": ["Test"],
                "total_players": 2
            },
            "players": [
                {
                    "first_name": "Travis",
                    "last_name": "Hunter",
                    "position": "CB",
                    "college": "University of Colorado",
                    "height_inches": 73,
                    "weight_pounds": 185,
                    "notes": "Two-way player"
                },
                {
                    "first_name": "Shedeur",
                    "last_name": "Sanders",
                    "position": "QB",
                    "college": "University of Colorado",
                    "height_inches": 74,
                    "weight_pounds": 215,
                    "notes": null
                }
            ]
        }"#
    }

    #[test]
    fn test_parse_json() {
        let data: PlayerData = serde_json::from_str(sample_json()).unwrap();
        assert_eq!(data.meta.draft_year, 2026);
        assert_eq!(data.meta.total_players, 2);
        assert_eq!(data.players.len(), 2);
        assert_eq!(data.players[0].first_name, "Travis");
        assert_eq!(data.players[0].last_name, "Hunter");
    }

    #[test]
    fn test_player_entry_to_domain() {
        let entry = PlayerEntry {
            first_name: "Travis".to_string(),
            last_name: "Hunter".to_string(),
            position: "CB".to_string(),
            college: Some("University of Colorado".to_string()),
            height_inches: Some(73),
            weight_pounds: Some(185),
            notes: Some("Two-way player".to_string()),
        };

        let player = entry.to_domain(2026).unwrap();
        assert_eq!(player.first_name, "Travis");
        assert_eq!(player.last_name, "Hunter");
        assert_eq!(player.position, domain::models::Position::CB);
        assert_eq!(player.college, Some("University of Colorado".to_string()));
        assert_eq!(player.height_inches, Some(73));
        assert_eq!(player.weight_pounds, Some(185));
        assert_eq!(player.draft_year, 2026);
    }

    #[test]
    fn test_player_entry_without_optional_fields() {
        let entry = PlayerEntry {
            first_name: "Test".to_string(),
            last_name: "Player".to_string(),
            position: "QB".to_string(),
            college: None,
            height_inches: None,
            weight_pounds: None,
            notes: None,
        };

        let player = entry.to_domain(2026).unwrap();
        assert!(player.college.is_none());
        assert!(player.height_inches.is_none());
        assert!(player.weight_pounds.is_none());
    }

    #[test]
    fn test_player_entry_with_edge_position() {
        let entry = PlayerEntry {
            first_name: "Test".to_string(),
            last_name: "Player".to_string(),
            position: "EDGE".to_string(),
            college: None,
            height_inches: None,
            weight_pounds: None,
            notes: None,
        };

        let player = entry.to_domain(2026).unwrap();
        assert_eq!(player.position, domain::models::Position::DE);
    }

    #[test]
    fn test_player_entry_invalid_position() {
        let entry = PlayerEntry {
            first_name: "Test".to_string(),
            last_name: "Player".to_string(),
            position: "ATH".to_string(),
            college: None,
            height_inches: None,
            weight_pounds: None,
            notes: None,
        };

        assert!(entry.to_domain(2026).is_err());
    }

    #[test]
    fn test_partial_physical_stats_ignored() {
        // If only height is provided (no weight), neither should be set
        let entry = PlayerEntry {
            first_name: "Test".to_string(),
            last_name: "Player".to_string(),
            position: "QB".to_string(),
            college: None,
            height_inches: Some(72),
            weight_pounds: None,
            notes: None,
        };

        let player = entry.to_domain(2026).unwrap();
        // Only sets physical stats when both are provided
        assert!(player.height_inches.is_none());
        assert!(player.weight_pounds.is_none());
    }
}
