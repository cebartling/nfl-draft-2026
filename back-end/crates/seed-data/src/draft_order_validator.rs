use std::collections::HashSet;

use crate::draft_order_loader::DraftOrderData;
use crate::{COMPENSATORY_ROUND_MAX, COMPENSATORY_ROUND_MIN};

/// Valid NFL team abbreviations
const VALID_TEAM_ABBREVIATIONS: &[&str] = &[
    "ARI", "ATL", "BAL", "BUF", "CAR", "CHI", "CIN", "CLE", "DAL", "DEN", "DET", "GB", "HOU",
    "IND", "JAX", "KC", "LAC", "LAR", "LV", "MIA", "MIN", "NE", "NO", "NYG", "NYJ", "PHI", "PIT",
    "SEA", "SF", "TB", "TEN", "WAS",
];

pub struct DraftOrderValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl DraftOrderValidationResult {
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

pub fn validate_draft_order_data(data: &DraftOrderData) -> DraftOrderValidationResult {
    let mut result = DraftOrderValidationResult {
        valid: true,
        warnings: vec![],
        errors: vec![],
    };

    // Check for empty draft_order array
    if data.draft_order.is_empty() {
        result
            .errors
            .push("No draft order entries found in data".to_string());
        result.valid = false;
        return result;
    }

    // Validate meta.total_picks matches actual count
    if data.meta.total_picks != data.draft_order.len() {
        result.warnings.push(format!(
            "Meta total_picks ({}) does not match actual count ({})",
            data.meta.total_picks,
            data.draft_order.len()
        ));
    }

    // Check for duplicate overall_pick values
    let mut seen_overall: HashSet<i32> = HashSet::new();
    for entry in &data.draft_order {
        if !seen_overall.insert(entry.overall_pick) {
            result
                .errors
                .push(format!("Duplicate overall_pick: {}", entry.overall_pick));
            result.valid = false;
        }
    }

    // Check for duplicate (round, pick_in_round) pairs
    let mut seen_round_pick: HashSet<(i32, i32)> = HashSet::new();
    for entry in &data.draft_order {
        if !seen_round_pick.insert((entry.round, entry.pick_in_round)) {
            result.errors.push(format!(
                "Duplicate (round, pick_in_round): ({}, {})",
                entry.round, entry.pick_in_round
            ));
            result.valid = false;
        }
    }

    // Check overall_pick values are sequential (1..N)
    let mut overall_picks: Vec<i32> = data.draft_order.iter().map(|e| e.overall_pick).collect();
    overall_picks.sort();
    let expected: Vec<i32> = (1..=data.draft_order.len() as i32).collect();
    if overall_picks != expected {
        let missing: Vec<i32> = expected
            .iter()
            .filter(|p| !overall_picks.contains(p))
            .cloned()
            .collect();
        if !missing.is_empty() {
            result.errors.push(format!(
                "Overall picks not sequential. Missing: {:?}",
                missing
            ));
            result.valid = false;
        }
    }

    // Check pick_in_round values are sequential within each round
    let mut rounds: std::collections::BTreeMap<i32, Vec<i32>> = std::collections::BTreeMap::new();
    for entry in &data.draft_order {
        rounds
            .entry(entry.round)
            .or_default()
            .push(entry.pick_in_round);
    }
    for (round, mut picks) in rounds {
        picks.sort();
        let expected_picks: Vec<i32> = (1..=picks.len() as i32).collect();
        if picks != expected_picks {
            result.errors.push(format!(
                "Round {} pick_in_round values not sequential: got {:?}, expected {:?}",
                round, picks, expected_picks
            ));
            result.valid = false;
        }
    }

    // Validate each entry
    for entry in &data.draft_order {
        let label = format!(
            "Pick {} (R{}P{})",
            entry.overall_pick, entry.round, entry.pick_in_round
        );

        // Validate team abbreviations
        if !VALID_TEAM_ABBREVIATIONS.contains(&entry.team_abbreviation.as_str()) {
            result.errors.push(format!(
                "{}: Invalid team abbreviation '{}'",
                label, entry.team_abbreviation
            ));
            result.valid = false;
        }

        if !VALID_TEAM_ABBREVIATIONS.contains(&entry.original_team_abbreviation.as_str()) {
            result.errors.push(format!(
                "{}: Invalid original team abbreviation '{}'",
                label, entry.original_team_abbreviation
            ));
            result.valid = false;
        }

        // Validate round range
        if entry.round < 1 || entry.round > data.meta.total_rounds {
            result.errors.push(format!(
                "{}: Round {} out of range (1-{})",
                label, entry.round, data.meta.total_rounds
            ));
            result.valid = false;
        }

        // Compensatory picks only in rounds 3-7
        if entry.is_compensatory
            && (entry.round < COMPENSATORY_ROUND_MIN || entry.round > COMPENSATORY_ROUND_MAX)
        {
            result.errors.push(format!(
                "{}: Compensatory pick not allowed in round {}",
                label, entry.round
            ));
            result.valid = false;
        }
    }

    // Warn about typical pick count ranges
    let total = data.draft_order.len();
    if total < 220 {
        result.warnings.push(format!(
            "Only {} total picks (typical NFL draft has 250-260)",
            total
        ));
    } else if total > 280 {
        result.warnings.push(format!(
            "{} total picks seems high (typical NFL draft has 250-260)",
            total
        ));
    }

    // Check meta.total_rounds matches actual max round
    let max_round = data.draft_order.iter().map(|e| e.round).max().unwrap_or(0);
    if max_round != data.meta.total_rounds {
        result.warnings.push(format!(
            "Meta total_rounds ({}) does not match max round in data ({})",
            data.meta.total_rounds, max_round
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draft_order_loader::{DraftOrderEntry, DraftOrderMeta};

    fn make_meta(year: i32, rounds: i32, total: usize) -> DraftOrderMeta {
        DraftOrderMeta {
            version: "1.0.0".to_string(),
            last_updated: "2026-02-06".to_string(),
            sources: vec!["Test".to_string()],
            source: None,
            draft_year: year,
            total_rounds: rounds,
            total_picks: total,
        }
    }

    fn make_entry(
        round: i32,
        pick_in_round: i32,
        overall_pick: i32,
        team: &str,
        original_team: &str,
        is_compensatory: bool,
    ) -> DraftOrderEntry {
        DraftOrderEntry {
            round,
            pick_in_round,
            overall_pick,
            team_abbreviation: team.to_string(),
            original_team_abbreviation: original_team.to_string(),
            is_compensatory,
            notes: None,
        }
    }

    #[test]
    fn test_valid_data_passes() {
        let data = DraftOrderData {
            meta: make_meta(2026, 1, 2),
            draft_order: vec![
                make_entry(1, 1, 1, "TEN", "TEN", false),
                make_entry(1, 2, 2, "DAL", "GB", false),
            ],
        };

        let result = validate_draft_order_data(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_empty_draft_order_fails() {
        let data = DraftOrderData {
            meta: make_meta(2026, 7, 0),
            draft_order: vec![],
        };

        let result = validate_draft_order_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("No draft order entries")));
    }

    #[test]
    fn test_duplicate_overall_pick_fails() {
        let data = DraftOrderData {
            meta: make_meta(2026, 1, 2),
            draft_order: vec![
                make_entry(1, 1, 1, "TEN", "TEN", false),
                make_entry(1, 2, 1, "CLE", "CLE", false), // dup overall 1
            ],
        };

        let result = validate_draft_order_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Duplicate overall_pick")));
    }

    #[test]
    fn test_duplicate_round_pick_fails() {
        let data = DraftOrderData {
            meta: make_meta(2026, 1, 2),
            draft_order: vec![
                make_entry(1, 1, 1, "TEN", "TEN", false),
                make_entry(1, 1, 2, "CLE", "CLE", false), // dup (1,1)
            ],
        };

        let result = validate_draft_order_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Duplicate (round, pick_in_round)")));
    }

    #[test]
    fn test_non_sequential_overall_picks_fails() {
        let data = DraftOrderData {
            meta: make_meta(2026, 1, 2),
            draft_order: vec![
                make_entry(1, 1, 1, "TEN", "TEN", false),
                make_entry(1, 2, 3, "CLE", "CLE", false), // gap: missing 2
            ],
        };

        let result = validate_draft_order_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("not sequential")));
    }

    #[test]
    fn test_invalid_team_abbreviation_fails() {
        let data = DraftOrderData {
            meta: make_meta(2026, 1, 1),
            draft_order: vec![make_entry(1, 1, 1, "XYZ", "XYZ", false)],
        };

        let result = validate_draft_order_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Invalid team abbreviation")));
    }

    #[test]
    fn test_compensatory_pick_round_1_fails() {
        let data = DraftOrderData {
            meta: make_meta(2026, 1, 1),
            draft_order: vec![make_entry(1, 1, 1, "TEN", "TEN", true)],
        };

        let result = validate_draft_order_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Compensatory pick not allowed")));
    }

    #[test]
    fn test_compensatory_pick_round_3_passes() {
        let data = DraftOrderData {
            meta: make_meta(2026, 3, 3),
            draft_order: vec![
                make_entry(1, 1, 1, "TEN", "TEN", false),
                make_entry(2, 1, 2, "TEN", "TEN", false),
                make_entry(3, 1, 3, "TEN", "TEN", true),
            ],
        };

        let result = validate_draft_order_data(&data);
        assert!(result.valid);
    }

    #[test]
    fn test_meta_total_picks_mismatch_warns() {
        let data = DraftOrderData {
            meta: make_meta(2026, 1, 5), // says 5 but only 1
            draft_order: vec![make_entry(1, 1, 1, "TEN", "TEN", false)],
        };

        let result = validate_draft_order_data(&data);
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("total_picks")));
    }

    #[test]
    fn test_traded_pick_valid() {
        let data = DraftOrderData {
            meta: make_meta(2026, 1, 1),
            draft_order: vec![make_entry(1, 1, 1, "DAL", "GB", false)],
        };

        let result = validate_draft_order_data(&data);
        assert!(result.valid);
    }

    #[test]
    fn test_compensatory_pick_round_8_fails() {
        let data = DraftOrderData {
            meta: make_meta(2026, 8, 8),
            draft_order: vec![
                make_entry(1, 1, 1, "TEN", "TEN", false),
                make_entry(2, 1, 2, "TEN", "TEN", false),
                make_entry(3, 1, 3, "TEN", "TEN", false),
                make_entry(4, 1, 4, "TEN", "TEN", false),
                make_entry(5, 1, 5, "TEN", "TEN", false),
                make_entry(6, 1, 6, "TEN", "TEN", false),
                make_entry(7, 1, 7, "TEN", "TEN", false),
                make_entry(8, 1, 8, "TEN", "TEN", true),
            ],
        };

        let result = validate_draft_order_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Compensatory pick not allowed")));
    }

    #[test]
    fn test_non_sequential_pick_in_round_fails() {
        let data = DraftOrderData {
            meta: make_meta(2026, 1, 2),
            draft_order: vec![
                make_entry(1, 1, 1, "TEN", "TEN", false),
                make_entry(1, 3, 2, "CLE", "CLE", false), // skip pick_in_round 2
            ],
        };

        let result = validate_draft_order_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("pick_in_round values not sequential")));
    }
}
