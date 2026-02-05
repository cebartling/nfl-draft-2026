use std::collections::HashSet;

use crate::position_mapper::map_position;
use crate::team_need_loader::TeamNeedData;

/// Valid NFL team abbreviations
const VALID_TEAM_ABBREVIATIONS: &[&str] = &[
    "ARI", "ATL", "BAL", "BUF", "CAR", "CHI", "CIN", "CLE", "DAL", "DEN", "DET", "GB", "HOU",
    "IND", "JAX", "KC", "LAC", "LAR", "LV", "MIA", "MIN", "NE", "NO", "NYG", "NYJ", "PHI", "PIT",
    "SEA", "SF", "TB", "TEN", "WAS",
];

pub struct TeamNeedValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl TeamNeedValidationResult {
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

pub fn validate_team_need_data(data: &TeamNeedData) -> TeamNeedValidationResult {
    let mut result = TeamNeedValidationResult {
        valid: true,
        warnings: vec![],
        errors: vec![],
    };

    // Check for empty team_needs array
    if data.team_needs.is_empty() {
        result
            .errors
            .push("No team needs found in data".to_string());
        result.valid = false;
        return result;
    }

    // Check for duplicate team abbreviations
    let mut seen_teams = HashSet::new();
    for entry in &data.team_needs {
        if !seen_teams.insert(entry.team_abbreviation.clone()) {
            result.errors.push(format!(
                "Duplicate team abbreviation: {}",
                entry.team_abbreviation
            ));
            result.valid = false;
        }
    }

    // Validate each team entry
    for entry in &data.team_needs {
        let label = format!("Team {}", entry.team_abbreviation);

        // Validate team abbreviation is one of 32 NFL teams
        if !VALID_TEAM_ABBREVIATIONS.contains(&entry.team_abbreviation.as_str()) {
            result.errors.push(format!(
                "{}: Invalid team abbreviation '{}' (not one of 32 NFL teams)",
                label, entry.team_abbreviation
            ));
            result.valid = false;
        }

        // Check for empty needs
        if entry.needs.is_empty() {
            result
                .errors
                .push(format!("{}: Team has zero positional needs", label));
            result.valid = false;
            continue;
        }

        // Track positions for this team to detect duplicates
        let mut seen_positions = HashSet::new();

        for need in &entry.needs {
            // Validate position using position_mapper
            match map_position(&need.position) {
                Err(e) => {
                    result.errors.push(format!(
                        "{}: Invalid position '{}' - {}",
                        label, need.position, e
                    ));
                    result.valid = false;
                }
                Ok(canonical_position) => {
                    // Check for duplicate positions within team using canonical position
                    // This catches aliases like EDGE and DE being duplicates
                    let canonical_str = format!("{:?}", canonical_position);
                    if !seen_positions.insert(canonical_str.clone()) {
                        result.errors.push(format!(
                            "{}: Duplicate position '{}' in needs (maps to same position as another entry)",
                            label, need.position
                        ));
                        result.valid = false;
                    }
                }
            }

            // Validate priority is 1-10
            if need.priority < 1 || need.priority > 10 {
                result.errors.push(format!(
                    "{}: Invalid priority {} for position '{}' (must be 1-10)",
                    label, need.priority, need.position
                ));
                result.valid = false;
            }
        }

        // Warn if team has fewer than 5 or more than 8 needs
        let need_count = entry.needs.len();
        if need_count < 5 {
            result.warnings.push(format!(
                "{}: Only {} positional needs (recommend 5-8)",
                label, need_count
            ));
        } else if need_count > 8 {
            result.warnings.push(format!(
                "{}: {} positional needs exceeds recommended 5-8",
                label, need_count
            ));
        }
    }

    // Check meta total_teams matches actual count
    if data.meta.total_teams != data.team_needs.len() {
        result.warnings.push(format!(
            "Meta total_teams ({}) does not match actual count ({})",
            data.meta.total_teams,
            data.team_needs.len()
        ));
    }

    // Warn if not exactly 32 teams
    if data.team_needs.len() != 32 {
        result.warnings.push(format!(
            "Expected 32 NFL teams but found {}",
            data.team_needs.len()
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::team_need_loader::{PositionalNeed, TeamNeedEntry, TeamNeedMetaData};

    fn make_meta(total: usize) -> TeamNeedMetaData {
        TeamNeedMetaData {
            version: "1.0.0".to_string(),
            last_updated: "2026-02-04".to_string(),
            sources: vec!["Test".to_string()],
            total_teams: total,
            description: "Test data".to_string(),
        }
    }

    fn make_need(pos: &str, priority: i32) -> PositionalNeed {
        PositionalNeed {
            position: pos.to_string(),
            priority,
        }
    }

    fn make_team_entry(abbr: &str, needs: Vec<PositionalNeed>) -> TeamNeedEntry {
        TeamNeedEntry {
            team_abbreviation: abbr.to_string(),
            needs,
        }
    }

    #[test]
    fn test_valid_data_passes() {
        let data = TeamNeedData {
            meta: make_meta(2),
            team_needs: vec![
                make_team_entry(
                    "DAL",
                    vec![
                        make_need("QB", 1),
                        make_need("OT", 2),
                        make_need("WR", 3),
                        make_need("CB", 4),
                        make_need("EDGE", 5),
                    ],
                ),
                make_team_entry(
                    "BUF",
                    vec![
                        make_need("CB", 1),
                        make_need("OT", 2),
                        make_need("WR", 3),
                        make_need("S", 4),
                        make_need("LB", 5),
                    ],
                ),
            ],
        };

        let result = validate_team_need_data(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
        // Will have warnings because team count != 32
        assert!(result.warnings.iter().any(|w| w.contains("32")));
    }

    #[test]
    fn test_empty_team_needs_fails() {
        let data = TeamNeedData {
            meta: make_meta(0),
            team_needs: vec![],
        };

        let result = validate_team_need_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("No team needs found")));
    }

    #[test]
    fn test_duplicate_team_abbreviations_fails() {
        let data = TeamNeedData {
            meta: make_meta(2),
            team_needs: vec![
                make_team_entry("DAL", vec![make_need("QB", 1), make_need("OT", 2)]),
                make_team_entry("DAL", vec![make_need("WR", 1), make_need("CB", 2)]),
            ],
        };

        let result = validate_team_need_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Duplicate team abbreviation")));
    }

    #[test]
    fn test_invalid_team_abbreviation_fails() {
        let data = TeamNeedData {
            meta: make_meta(1),
            team_needs: vec![make_team_entry(
                "XYZ",
                vec![make_need("QB", 1), make_need("OT", 2)],
            )],
        };

        let result = validate_team_need_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Invalid team abbreviation")));
    }

    #[test]
    fn test_invalid_position_fails() {
        let data = TeamNeedData {
            meta: make_meta(1),
            team_needs: vec![make_team_entry(
                "DAL",
                vec![make_need("QB", 1), make_need("INVALID", 2)],
            )],
        };

        let result = validate_team_need_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid position")));
    }

    #[test]
    fn test_invalid_priority_low_fails() {
        let data = TeamNeedData {
            meta: make_meta(1),
            team_needs: vec![make_team_entry(
                "DAL",
                vec![make_need("QB", 0), make_need("OT", 2)],
            )],
        };

        let result = validate_team_need_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid priority")));
    }

    #[test]
    fn test_invalid_priority_high_fails() {
        let data = TeamNeedData {
            meta: make_meta(1),
            team_needs: vec![make_team_entry(
                "DAL",
                vec![make_need("QB", 1), make_need("OT", 11)],
            )],
        };

        let result = validate_team_need_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid priority")));
    }

    #[test]
    fn test_duplicate_position_within_team_fails() {
        let data = TeamNeedData {
            meta: make_meta(1),
            team_needs: vec![make_team_entry(
                "DAL",
                vec![make_need("QB", 1), make_need("QB", 2)],
            )],
        };

        let result = validate_team_need_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Duplicate position")));
    }

    #[test]
    fn test_team_with_zero_needs_fails() {
        let data = TeamNeedData {
            meta: make_meta(1),
            team_needs: vec![make_team_entry("DAL", vec![])],
        };

        let result = validate_team_need_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("zero positional needs")));
    }

    #[test]
    fn test_meta_count_mismatch_warns() {
        let data = TeamNeedData {
            meta: make_meta(5), // Says 5 but only 1
            team_needs: vec![make_team_entry(
                "DAL",
                vec![
                    make_need("QB", 1),
                    make_need("OT", 2),
                    make_need("WR", 3),
                    make_need("CB", 4),
                    make_need("LB", 5),
                ],
            )],
        };

        let result = validate_team_need_data(&data);
        assert!(result.valid); // Mismatch is a warning, not an error
        assert!(result.warnings.iter().any(|w| w.contains("total_teams")));
    }

    #[test]
    fn test_few_needs_warns() {
        let data = TeamNeedData {
            meta: make_meta(1),
            team_needs: vec![make_team_entry(
                "DAL",
                vec![make_need("QB", 1), make_need("OT", 2)],
            )],
        };

        let result = validate_team_need_data(&data);
        assert!(result.valid);
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("Only 2 positional needs")));
    }

    #[test]
    fn test_edge_maps_to_de() {
        // EDGE should be valid via position_mapper
        let data = TeamNeedData {
            meta: make_meta(1),
            team_needs: vec![make_team_entry(
                "DAL",
                vec![
                    make_need("EDGE", 1),
                    make_need("OT", 2),
                    make_need("WR", 3),
                    make_need("CB", 4),
                    make_need("LB", 5),
                ],
            )],
        };

        let result = validate_team_need_data(&data);
        assert!(result.valid);
    }

    #[test]
    fn test_edge_and_de_are_duplicates() {
        // EDGE and DE map to the same canonical position, so they should be detected as duplicates
        let data = TeamNeedData {
            meta: make_meta(1),
            team_needs: vec![make_team_entry(
                "DAL",
                vec![
                    make_need("EDGE", 1),
                    make_need("DE", 2),
                    make_need("OT", 3),
                    make_need("CB", 4),
                    make_need("LB", 5),
                ],
            )],
        };

        let result = validate_team_need_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Duplicate position")));
    }
}
