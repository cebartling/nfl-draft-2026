use std::collections::HashSet;

use crate::team_season_loader::TeamSeasonData;

/// Valid NFL team abbreviations
const VALID_TEAM_ABBREVIATIONS: &[&str] = &[
    "ARI", "ATL", "BAL", "BUF", "CAR", "CHI", "CIN", "CLE", "DAL", "DEN", "DET", "GB", "HOU",
    "IND", "JAX", "KC", "LAC", "LAR", "LV", "MIA", "MIN", "NE", "NO", "NYG", "NYJ", "PHI", "PIT",
    "SEA", "SF", "TB", "TEN", "WAS",
];

/// Valid playoff result strings
const VALID_PLAYOFF_RESULTS: &[&str] = &[
    "MissedPlayoffs",
    "WildCard",
    "Divisional",
    "Conference",
    "SuperBowlLoss",
    "SuperBowlWin",
];

pub struct TeamSeasonValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl TeamSeasonValidationResult {
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

pub fn validate_team_season_data(data: &TeamSeasonData) -> TeamSeasonValidationResult {
    let mut result = TeamSeasonValidationResult {
        valid: true,
        warnings: vec![],
        errors: vec![],
    };

    // Check for empty team_seasons array
    if data.team_seasons.is_empty() {
        result
            .errors
            .push("No team seasons found in data".to_string());
        result.valid = false;
        return result;
    }

    // Check for duplicate team abbreviations
    let mut seen_teams = HashSet::new();
    for entry in &data.team_seasons {
        if !seen_teams.insert(entry.team_abbreviation.clone()) {
            result.errors.push(format!(
                "Duplicate team abbreviation: {}",
                entry.team_abbreviation
            ));
            result.valid = false;
        }
    }

    // Check for duplicate draft positions
    let mut seen_positions: HashSet<i32> = HashSet::new();
    for entry in &data.team_seasons {
        if let Some(pos) = entry.draft_position {
            if !seen_positions.insert(pos) {
                result.errors.push(format!(
                    "Duplicate draft position {}: {} already assigned",
                    pos, entry.team_abbreviation
                ));
                result.valid = false;
            }
        }
    }

    // Validate each team entry
    for entry in &data.team_seasons {
        let label = format!("Team {}", entry.team_abbreviation);

        // Validate team abbreviation is one of 32 NFL teams
        if !VALID_TEAM_ABBREVIATIONS.contains(&entry.team_abbreviation.as_str()) {
            result.errors.push(format!(
                "{}: Invalid team abbreviation '{}' (not one of 32 NFL teams)",
                label, entry.team_abbreviation
            ));
            result.valid = false;
        }

        // Validate wins, losses, ties
        if entry.wins < 0 || entry.wins > 17 {
            result.errors.push(format!(
                "{}: Invalid wins {} (must be 0-17)",
                label, entry.wins
            ));
            result.valid = false;
        }

        if entry.losses < 0 || entry.losses > 17 {
            result.errors.push(format!(
                "{}: Invalid losses {} (must be 0-17)",
                label, entry.losses
            ));
            result.valid = false;
        }

        if entry.ties < 0 || entry.ties > 17 {
            result.errors.push(format!(
                "{}: Invalid ties {} (must be 0-17)",
                label, entry.ties
            ));
            result.valid = false;
        }

        // Validate total games
        let total_games = entry.wins + entry.losses + entry.ties;
        if total_games > 17 {
            result
                .errors
                .push(format!("{}: Total games {} exceeds 17", label, total_games));
            result.valid = false;
        }

        // Warn if total games is less than 17
        if total_games < 17 {
            result.warnings.push(format!(
                "{}: Only {} total games (expected 17)",
                label, total_games
            ));
        }

        // Validate playoff result if present
        if let Some(ref pr) = entry.playoff_result {
            if !VALID_PLAYOFF_RESULTS.contains(&pr.as_str()) {
                result.errors.push(format!(
                    "{}: Invalid playoff result '{}' (must be one of: {:?})",
                    label, pr, VALID_PLAYOFF_RESULTS
                ));
                result.valid = false;
            }
        }

        // Validate draft position if present
        if let Some(pos) = entry.draft_position {
            if !(1..=32).contains(&pos) {
                result.errors.push(format!(
                    "{}: Invalid draft position {} (must be 1-32)",
                    label, pos
                ));
                result.valid = false;
            }
        }
    }

    // Check meta total_teams matches actual count
    if data.meta.total_teams != data.team_seasons.len() {
        result.warnings.push(format!(
            "Meta total_teams ({}) does not match actual count ({})",
            data.meta.total_teams,
            data.team_seasons.len()
        ));
    }

    // Warn if not exactly 32 teams
    if data.team_seasons.len() != 32 {
        result.warnings.push(format!(
            "Expected 32 NFL teams but found {}",
            data.team_seasons.len()
        ));
    }

    // Check if all draft positions 1-32 are present
    let entries_with_positions: Vec<_> = data
        .team_seasons
        .iter()
        .filter_map(|e| e.draft_position)
        .collect();

    if entries_with_positions.len() == 32 {
        // All teams have positions, check for completeness
        let mut positions: Vec<i32> = entries_with_positions.clone();
        positions.sort();
        let expected: Vec<i32> = (1..=32).collect();
        if positions != expected {
            let missing: Vec<i32> = expected
                .iter()
                .filter(|p| !positions.contains(p))
                .cloned()
                .collect();
            result.errors.push(format!(
                "Draft positions not complete. Missing: {:?}",
                missing
            ));
            result.valid = false;
        }
    } else if !entries_with_positions.is_empty() {
        // Some but not all have positions
        result.warnings.push(format!(
            "Only {} of {} teams have draft positions assigned",
            entries_with_positions.len(),
            data.team_seasons.len()
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::team_season_loader::{TeamSeasonEntry, TeamSeasonMetaData};

    fn make_meta(season_year: i32, total: usize) -> TeamSeasonMetaData {
        TeamSeasonMetaData {
            version: "1.0.0".to_string(),
            last_updated: "2026-02-04".to_string(),
            sources: vec!["Test".to_string()],
            season_year,
            total_teams: total,
        }
    }

    fn make_entry(
        abbr: &str,
        wins: i32,
        losses: i32,
        ties: i32,
        playoff_result: Option<&str>,
        draft_position: Option<i32>,
    ) -> TeamSeasonEntry {
        TeamSeasonEntry {
            team_abbreviation: abbr.to_string(),
            wins,
            losses,
            ties,
            playoff_result: playoff_result.map(|s| s.to_string()),
            draft_position,
        }
    }

    #[test]
    fn test_valid_data_passes() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 2),
            team_seasons: vec![
                make_entry("DAL", 10, 7, 0, Some("MissedPlayoffs"), Some(1)),
                make_entry("PHI", 14, 3, 0, Some("SuperBowlWin"), Some(2)),
            ],
        };

        let result = validate_team_season_data(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
        // Will have warnings about team count != 32
        assert!(result.warnings.iter().any(|w| w.contains("32")));
    }

    #[test]
    fn test_empty_team_seasons_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 0),
            team_seasons: vec![],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("No team seasons found")));
    }

    #[test]
    fn test_duplicate_team_abbreviation_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 2),
            team_seasons: vec![
                make_entry("DAL", 10, 7, 0, None, Some(1)),
                make_entry("DAL", 8, 9, 0, None, Some(2)),
            ],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Duplicate team abbreviation")));
    }

    #[test]
    fn test_duplicate_draft_position_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 2),
            team_seasons: vec![
                make_entry("DAL", 10, 7, 0, None, Some(1)),
                make_entry("PHI", 14, 3, 0, None, Some(1)),
            ],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Duplicate draft position")));
    }

    #[test]
    fn test_invalid_team_abbreviation_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("XYZ", 10, 7, 0, None, Some(1))],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Invalid team abbreviation")));
    }

    #[test]
    fn test_invalid_wins_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("DAL", 18, 0, 0, None, Some(1))],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid wins")));
    }

    #[test]
    fn test_invalid_losses_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("DAL", 0, -1, 0, None, Some(1))],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid losses")));
    }

    #[test]
    fn test_too_many_games_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("DAL", 10, 5, 5, None, Some(1))],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("exceeds 17")));
    }

    #[test]
    fn test_invalid_playoff_result_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("DAL", 10, 7, 0, Some("Invalid"), Some(1))],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Invalid playoff result")));
    }

    #[test]
    fn test_invalid_draft_position_low_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("DAL", 10, 7, 0, None, Some(0))],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Invalid draft position")));
    }

    #[test]
    fn test_invalid_draft_position_high_fails() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("DAL", 10, 7, 0, None, Some(33))],
        };

        let result = validate_team_season_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Invalid draft position")));
    }

    #[test]
    fn test_meta_count_mismatch_warns() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 5), // Says 5 but only 1
            team_seasons: vec![make_entry("DAL", 10, 7, 0, None, Some(1))],
        };

        let result = validate_team_season_data(&data);
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("total_teams")));
    }

    #[test]
    fn test_fewer_than_17_games_warns() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("DAL", 10, 6, 0, None, Some(1))],
        };

        let result = validate_team_season_data(&data);
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("16 total games")));
    }

    #[test]
    fn test_null_playoff_result_is_valid() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("DAL", 10, 7, 0, None, Some(1))],
        };

        let result = validate_team_season_data(&data);
        assert!(result.valid);
    }

    #[test]
    fn test_null_draft_position_is_valid() {
        let data = TeamSeasonData {
            meta: make_meta(2025, 1),
            team_seasons: vec![make_entry("DAL", 10, 7, 0, None, None)],
        };

        let result = validate_team_season_data(&data);
        assert!(result.valid);
    }
}
