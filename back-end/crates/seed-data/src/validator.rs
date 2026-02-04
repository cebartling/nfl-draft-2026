use std::collections::HashSet;

use crate::loader::PlayerData;
use crate::position_mapper;

pub struct ValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn print_summary(&self) {
        if !self.errors.is_empty() {
            println!("\nErrors ({}):", self.errors.len());
            for error in &self.errors {
                println!("  [ERROR] {}", error);
            }
        }

        if !self.warnings.is_empty() {
            println!("\nWarnings ({}):", self.warnings.len());
            for warning in &self.warnings {
                println!("  [WARN] {}", warning);
            }
        }

        if self.valid {
            println!("\nValidation PASSED");
        } else {
            println!("\nValidation FAILED");
        }
    }
}

pub fn validate_player_data(data: &PlayerData) -> ValidationResult {
    let mut result = ValidationResult {
        valid: true,
        warnings: vec![],
        errors: vec![],
    };

    if data.players.is_empty() {
        result.errors.push("No players found in data".to_string());
        result.valid = false;
        return result;
    }

    // Check for duplicate names
    let mut seen_names = HashSet::new();
    for player in &data.players {
        let full_name = format!("{} {}", player.first_name, player.last_name);
        if !seen_names.insert(full_name.clone()) {
            result
                .warnings
                .push(format!("Duplicate player: {}", full_name));
        }
    }

    // Validate each player entry
    for (i, player) in data.players.iter().enumerate() {
        let label = format!(
            "Player #{} ({} {})",
            i + 1,
            player.first_name,
            player.last_name
        );

        // Validate names are non-empty
        if player.first_name.trim().is_empty() {
            result
                .errors
                .push(format!("{}: First name is empty", label));
            result.valid = false;
        }
        if player.last_name.trim().is_empty() {
            result.errors.push(format!("{}: Last name is empty", label));
            result.valid = false;
        }

        // Validate physical stats if provided
        if let Some(height) = player.height_inches {
            if !(60..=90).contains(&height) {
                result.errors.push(format!(
                    "{}: Invalid height {} (must be 60-90)",
                    label, height
                ));
                result.valid = false;
            }
        }

        if let Some(weight) = player.weight_pounds {
            if !(150..=400).contains(&weight) {
                result.errors.push(format!(
                    "{}: Invalid weight {} (must be 150-400)",
                    label, weight
                ));
                result.valid = false;
            }
        }

        // Validate position can be mapped
        if let Err(e) = position_mapper::map_position(&player.position) {
            result.errors.push(format!("{}: {}", label, e));
            result.valid = false;
        }

        // Warn if college is missing
        if player.college.is_none() {
            result.warnings.push(format!("{}: Missing college", label));
        }
    }

    // Check total count
    if data.meta.total_players != data.players.len() {
        result.warnings.push(format!(
            "Meta total_players ({}) does not match actual count ({})",
            data.meta.total_players,
            data.players.len()
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::{MetaData, PlayerEntry};

    fn make_meta(total: usize) -> MetaData {
        MetaData {
            version: "1.0.0".to_string(),
            draft_year: 2026,
            last_updated: "2026-02-04".to_string(),
            sources: vec!["Test".to_string()],
            total_players: total,
        }
    }

    fn make_player(first: &str, last: &str, pos: &str) -> PlayerEntry {
        PlayerEntry {
            first_name: first.to_string(),
            last_name: last.to_string(),
            position: pos.to_string(),
            college: Some("Test University".to_string()),
            height_inches: Some(72),
            weight_pounds: Some(200),
            notes: None,
        }
    }

    #[test]
    fn test_valid_data_passes() {
        let data = PlayerData {
            meta: make_meta(2),
            players: vec![
                make_player("John", "Doe", "QB"),
                make_player("Jane", "Smith", "WR"),
            ],
        };

        let result = validate_player_data(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_empty_players_fails() {
        let data = PlayerData {
            meta: make_meta(0),
            players: vec![],
        };

        let result = validate_player_data(&data);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_duplicate_names_warns() {
        let data = PlayerData {
            meta: make_meta(2),
            players: vec![
                make_player("John", "Doe", "QB"),
                make_player("John", "Doe", "WR"),
            ],
        };

        let result = validate_player_data(&data);
        assert!(result.valid); // Duplicates are warnings, not errors
        assert!(result.warnings.iter().any(|w| w.contains("Duplicate")));
    }

    #[test]
    fn test_invalid_height_fails() {
        let mut player = make_player("John", "Doe", "QB");
        player.height_inches = Some(50);

        let data = PlayerData {
            meta: make_meta(1),
            players: vec![player],
        };

        let result = validate_player_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("height")));
    }

    #[test]
    fn test_invalid_weight_fails() {
        let mut player = make_player("John", "Doe", "QB");
        player.weight_pounds = Some(500);

        let data = PlayerData {
            meta: make_meta(1),
            players: vec![player],
        };

        let result = validate_player_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("weight")));
    }

    #[test]
    fn test_invalid_position_fails() {
        let data = PlayerData {
            meta: make_meta(1),
            players: vec![make_player("John", "Doe", "ATH")],
        };

        let result = validate_player_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("position")));
    }

    #[test]
    fn test_null_physical_stats_ok() {
        let mut player = make_player("John", "Doe", "QB");
        player.height_inches = None;
        player.weight_pounds = None;

        let data = PlayerData {
            meta: make_meta(1),
            players: vec![player],
        };

        let result = validate_player_data(&data);
        assert!(result.valid);
    }

    #[test]
    fn test_missing_college_warns() {
        let mut player = make_player("John", "Doe", "QB");
        player.college = None;

        let data = PlayerData {
            meta: make_meta(1),
            players: vec![player],
        };

        let result = validate_player_data(&data);
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("college")));
    }

    #[test]
    fn test_meta_count_mismatch_warns() {
        let data = PlayerData {
            meta: make_meta(5), // Says 5 but only 1
            players: vec![make_player("John", "Doe", "QB")],
        };

        let result = validate_player_data(&data);
        assert!(result.valid); // Mismatch is a warning, not an error
        assert!(result.warnings.iter().any(|w| w.contains("total_players")));
    }

    #[test]
    fn test_empty_first_name_fails() {
        let data = PlayerData {
            meta: make_meta(1),
            players: vec![make_player("", "Doe", "QB")],
        };

        let result = validate_player_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("First name")));
    }

    #[test]
    fn test_empty_last_name_fails() {
        let data = PlayerData {
            meta: make_meta(1),
            players: vec![make_player("John", "", "QB")],
        };

        let result = validate_player_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Last name")));
    }
}
