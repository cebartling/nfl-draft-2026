use std::collections::HashSet;

use crate::team_loader::{map_conference, map_division, TeamData};

pub struct TeamValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl TeamValidationResult {
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

pub fn validate_team_data(data: &TeamData) -> TeamValidationResult {
    let mut result = TeamValidationResult {
        valid: true,
        warnings: vec![],
        errors: vec![],
    };

    if data.teams.is_empty() {
        result.errors.push("No teams found in data".to_string());
        result.valid = false;
        return result;
    }

    // Check for duplicate abbreviations
    let mut seen_abbreviations = HashSet::new();
    for team in &data.teams {
        if !seen_abbreviations.insert(team.abbreviation.clone()) {
            result
                .errors
                .push(format!("Duplicate abbreviation: {}", team.abbreviation));
            result.valid = false;
        }
    }

    // Validate each team entry
    for (i, team) in data.teams.iter().enumerate() {
        let label = format!("Team #{} ({} - {})", i + 1, team.name, team.abbreviation);

        // Validate name is non-empty and within length limits
        if team.name.trim().is_empty() {
            result.errors.push(format!("{}: Name is empty", label));
            result.valid = false;
        } else if team.name.len() > 100 {
            result
                .errors
                .push(format!("{}: Name exceeds 100 characters", label));
            result.valid = false;
        }

        // Validate abbreviation is non-empty and within length limits
        if team.abbreviation.trim().is_empty() {
            result
                .errors
                .push(format!("{}: Abbreviation is empty", label));
            result.valid = false;
        } else if team.abbreviation.len() > 5 {
            result
                .errors
                .push(format!("{}: Abbreviation exceeds 5 characters", label));
            result.valid = false;
        }

        // Validate city is non-empty and within length limits
        if team.city.trim().is_empty() {
            result.errors.push(format!("{}: City is empty", label));
            result.valid = false;
        } else if team.city.len() > 100 {
            result
                .errors
                .push(format!("{}: City exceeds 100 characters", label));
            result.valid = false;
        }

        // Validate conference
        let conference = match map_conference(&team.conference) {
            Ok(c) => Some(c),
            Err(_) => {
                result.errors.push(format!(
                    "{}: Invalid conference '{}' (must be AFC or NFC)",
                    label, team.conference
                ));
                result.valid = false;
                None
            }
        };

        // Validate division
        let division = match map_division(&team.division) {
            Ok(d) => Some(d),
            Err(_) => {
                result.errors.push(format!(
                    "{}: Invalid division '{}' (must be one of: AFC East, AFC North, AFC South, AFC West, NFC East, NFC North, NFC South, NFC West)",
                    label, team.division
                ));
                result.valid = false;
                None
            }
        };

        // Validate conference-division match
        if let (Some(conf), Some(div)) = (conference, division) {
            let div_str = format!("{:?}", div);

            // Division name should start with the conference name
            let matches = match conf {
                domain::models::Conference::AFC => div_str.starts_with("AFC"),
                domain::models::Conference::NFC => div_str.starts_with("NFC"),
            };

            if !matches {
                result.errors.push(format!(
                    "{}: Division '{}' does not match conference '{}'",
                    label, team.division, team.conference
                ));
                result.valid = false;
            }
        }
    }

    // Check total count matches meta
    if data.meta.total_teams != data.teams.len() {
        result.warnings.push(format!(
            "Meta total_teams ({}) does not match actual count ({})",
            data.meta.total_teams,
            data.teams.len()
        ));
    }

    // Warn if count is not 32 (expected NFL team count)
    if data.teams.len() != 32 {
        result.warnings.push(format!(
            "Expected 32 NFL teams but found {}",
            data.teams.len()
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::team_loader::{TeamEntry, TeamMetaData};

    fn make_meta(total: usize) -> TeamMetaData {
        TeamMetaData {
            version: "1.0.0".to_string(),
            last_updated: "2026-02-04".to_string(),
            sources: vec!["Test".to_string()],
            total_teams: total,
        }
    }

    fn make_team(name: &str, abbrev: &str, city: &str, conf: &str, div: &str) -> TeamEntry {
        TeamEntry {
            name: name.to_string(),
            abbreviation: abbrev.to_string(),
            city: city.to_string(),
            conference: conf.to_string(),
            division: div.to_string(),
        }
    }

    #[test]
    fn test_valid_data_passes() {
        let data = TeamData {
            meta: make_meta(2),
            teams: vec![
                make_team("Dallas Cowboys", "DAL", "Arlington", "NFC", "NFC East"),
                make_team("Buffalo Bills", "BUF", "Buffalo", "AFC", "AFC East"),
            ],
        };

        let result = validate_team_data(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
        // Will have a warning because team count != 32
        assert!(result.warnings.iter().any(|w| w.contains("32")));
    }

    #[test]
    fn test_empty_teams_fails() {
        let data = TeamData {
            meta: make_meta(0),
            teams: vec![],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("No teams found")));
    }

    #[test]
    fn test_duplicate_abbreviations_fails() {
        let data = TeamData {
            meta: make_meta(2),
            teams: vec![
                make_team("Dallas Cowboys", "DAL", "Arlington", "NFC", "NFC East"),
                make_team("Duplicate Team", "DAL", "City", "NFC", "NFC East"),
            ],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Duplicate")));
    }

    #[test]
    fn test_empty_name_fails() {
        let data = TeamData {
            meta: make_meta(1),
            teams: vec![make_team("", "DAL", "Arlington", "NFC", "NFC East")],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Name is empty")));
    }

    #[test]
    fn test_name_too_long_fails() {
        let long_name = "a".repeat(101);
        let data = TeamData {
            meta: make_meta(1),
            teams: vec![make_team(&long_name, "DAL", "Arlington", "NFC", "NFC East")],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("exceeds 100")));
    }

    #[test]
    fn test_empty_abbreviation_fails() {
        let data = TeamData {
            meta: make_meta(1),
            teams: vec![make_team(
                "Dallas Cowboys",
                "",
                "Arlington",
                "NFC",
                "NFC East",
            )],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Abbreviation is empty")));
    }

    #[test]
    fn test_abbreviation_too_long_fails() {
        let data = TeamData {
            meta: make_meta(1),
            teams: vec![make_team(
                "Dallas Cowboys",
                "DALLAS",
                "Arlington",
                "NFC",
                "NFC East",
            )],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("exceeds 5")));
    }

    #[test]
    fn test_empty_city_fails() {
        let data = TeamData {
            meta: make_meta(1),
            teams: vec![make_team("Dallas Cowboys", "DAL", "", "NFC", "NFC East")],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("City is empty")));
    }

    #[test]
    fn test_invalid_conference_fails() {
        let data = TeamData {
            meta: make_meta(1),
            teams: vec![make_team(
                "Dallas Cowboys",
                "DAL",
                "Arlington",
                "XFL",
                "NFC East",
            )],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Invalid conference")));
    }

    #[test]
    fn test_invalid_division_fails() {
        let data = TeamData {
            meta: make_meta(1),
            teams: vec![make_team(
                "Dallas Cowboys",
                "DAL",
                "Arlington",
                "NFC",
                "Invalid Division",
            )],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid division")));
    }

    #[test]
    fn test_mismatched_conference_division_fails() {
        let data = TeamData {
            meta: make_meta(1),
            teams: vec![make_team(
                "Dallas Cowboys",
                "DAL",
                "Arlington",
                "NFC",
                "AFC East",
            )],
        };

        let result = validate_team_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("does not match")));
    }

    #[test]
    fn test_meta_count_mismatch_warns() {
        let data = TeamData {
            meta: make_meta(5), // Says 5 but only 1
            teams: vec![make_team(
                "Dallas Cowboys",
                "DAL",
                "Arlington",
                "NFC",
                "NFC East",
            )],
        };

        let result = validate_team_data(&data);
        assert!(result.valid); // Mismatch is a warning, not an error
        assert!(result.warnings.iter().any(|w| w.contains("total_teams")));
    }

    #[test]
    fn test_not_32_teams_warns() {
        let data = TeamData {
            meta: make_meta(2),
            teams: vec![
                make_team("Dallas Cowboys", "DAL", "Arlington", "NFC", "NFC East"),
                make_team("Buffalo Bills", "BUF", "Buffalo", "AFC", "AFC East"),
            ],
        };

        let result = validate_team_data(&data);
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("32")));
    }
}
