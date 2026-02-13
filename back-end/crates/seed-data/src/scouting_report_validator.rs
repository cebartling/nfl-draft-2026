use std::collections::HashSet;

use crate::position_mapper::map_position;
use crate::scouting_report_loader::RankingData;

pub struct ScoutingReportValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ScoutingReportValidationResult {
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

pub fn validate_ranking_data(data: &RankingData) -> ScoutingReportValidationResult {
    let mut result = ScoutingReportValidationResult {
        valid: true,
        warnings: vec![],
        errors: vec![],
    };

    // Check for empty rankings
    if data.rankings.is_empty() {
        result.errors.push("No rankings found in data".to_string());
        result.valid = false;
        return result;
    }

    // Check for duplicate names
    let mut seen_names = HashSet::new();
    for entry in &data.rankings {
        let key = format!(
            "{} {}",
            entry.first_name.to_lowercase(),
            entry.last_name.to_lowercase()
        );
        if !seen_names.insert(key.clone()) {
            result.errors.push(format!(
                "Duplicate prospect name: {} {}",
                entry.first_name, entry.last_name
            ));
            result.valid = false;
        }
    }

    // Validate each entry
    let mut seen_ranks = HashSet::new();
    for entry in &data.rankings {
        // Validate rank is non-negative (0 = unranked, 1+ = ranked)
        if entry.rank < 0 {
            result.errors.push(format!(
                "Invalid rank {} for {} {}",
                entry.rank, entry.first_name, entry.last_name
            ));
            result.valid = false;
        }

        // Check for duplicate ranks (only for positive/ranked entries)
        if entry.rank > 0 && !seen_ranks.insert(entry.rank) {
            result.warnings.push(format!(
                "Duplicate rank {} (may indicate tied rankings)",
                entry.rank
            ));
        }

        // Validate position
        if let Err(e) = map_position(&entry.position) {
            result.errors.push(format!(
                "Invalid position '{}' for {} {}: {}",
                entry.position, entry.first_name, entry.last_name, e
            ));
            result.valid = false;
        }

        // Validate name fields are non-empty
        if entry.first_name.trim().is_empty() {
            result
                .errors
                .push(format!("Empty first name for rank {}", entry.rank));
            result.valid = false;
        }

        if entry.last_name.trim().is_empty() {
            result
                .errors
                .push(format!("Empty last name for rank {}", entry.rank));
            result.valid = false;
        }
    }

    // Check meta total_prospects matches actual count
    if data.meta.total_prospects != data.rankings.len() {
        result.warnings.push(format!(
            "Meta total_prospects ({}) does not match actual count ({})",
            data.meta.total_prospects,
            data.rankings.len()
        ));
    }

    // Check for gaps in rank sequence
    let mut ranks: Vec<i32> = data.rankings.iter().map(|e| e.rank).collect();
    ranks.sort();
    if let (Some(&first), Some(&last)) = (ranks.first(), ranks.last()) {
        if first != 1 {
            result.warnings.push(format!(
                "Rankings don't start at 1 (first rank is {})",
                first
            ));
        }
        let expected_count = (last - first + 1) as usize;
        if expected_count != ranks.len() {
            result.warnings.push(format!(
                "Gaps detected in rank sequence: {} ranks spanning {} to {} (expected {})",
                ranks.len(),
                first,
                last,
                expected_count
            ));
        }
    }

    // Warn if prospect count is unusually low
    if data.rankings.len() < 100 {
        result.warnings.push(format!(
            "Only {} prospects (recommend at least 100 for realistic scouting reports)",
            data.rankings.len()
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scouting_report_loader::{RankingEntry, RankingMeta};

    fn make_meta(total: usize) -> RankingMeta {
        RankingMeta {
            version: "1.0.0".to_string(),
            source: "test".to_string(),
            source_url: "N/A".to_string(),
            draft_year: 2026,
            scraped_at: "2026-02-09".to_string(),
            total_prospects: total,
        }
    }

    fn make_entry(rank: i32, first: &str, last: &str, pos: &str) -> RankingEntry {
        RankingEntry {
            rank,
            first_name: first.to_string(),
            last_name: last.to_string(),
            position: pos.to_string(),
            school: "Test University".to_string(),
        }
    }

    #[test]
    fn test_valid_data_passes() {
        let data = RankingData {
            meta: make_meta(3),
            rankings: vec![
                make_entry(1, "John", "Smith", "QB"),
                make_entry(2, "Jane", "Doe", "WR"),
                make_entry(3, "Bob", "Jones", "CB"),
            ],
        };

        let result = validate_ranking_data(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_empty_rankings_fails() {
        let data = RankingData {
            meta: make_meta(0),
            rankings: vec![],
        };

        let result = validate_ranking_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("No rankings")));
    }

    #[test]
    fn test_duplicate_names_fails() {
        let data = RankingData {
            meta: make_meta(2),
            rankings: vec![
                make_entry(1, "John", "Smith", "QB"),
                make_entry(2, "John", "Smith", "WR"),
            ],
        };

        let result = validate_ranking_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Duplicate prospect")));
    }

    #[test]
    fn test_invalid_position_fails() {
        let data = RankingData {
            meta: make_meta(1),
            rankings: vec![make_entry(1, "John", "Smith", "INVALID")],
        };

        let result = validate_ranking_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid position")));
    }

    #[test]
    fn test_edge_position_valid() {
        let data = RankingData {
            meta: make_meta(1),
            rankings: vec![make_entry(1, "John", "Smith", "EDGE")],
        };

        let result = validate_ranking_data(&data);
        assert!(result.valid);
    }

    #[test]
    fn test_rank_zero_is_valid() {
        let data = RankingData {
            meta: make_meta(1),
            rankings: vec![make_entry(0, "John", "Smith", "QB")],
        };

        let result = validate_ranking_data(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_negative_rank_fails() {
        let data = RankingData {
            meta: make_meta(1),
            rankings: vec![make_entry(-1, "John", "Smith", "QB")],
        };

        let result = validate_ranking_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid rank")));
    }

    #[test]
    fn test_empty_first_name_fails() {
        let data = RankingData {
            meta: make_meta(1),
            rankings: vec![make_entry(1, "", "Smith", "QB")],
        };

        let result = validate_ranking_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Empty first name")));
    }

    #[test]
    fn test_empty_last_name_fails() {
        let data = RankingData {
            meta: make_meta(1),
            rankings: vec![make_entry(1, "John", "", "QB")],
        };

        let result = validate_ranking_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Empty last name")));
    }

    #[test]
    fn test_meta_count_mismatch_warns() {
        let data = RankingData {
            meta: make_meta(5), // Says 5 but only 2
            rankings: vec![
                make_entry(1, "John", "Smith", "QB"),
                make_entry(2, "Jane", "Doe", "WR"),
            ],
        };

        let result = validate_ranking_data(&data);
        assert!(result.valid);
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("total_prospects")));
    }

    #[test]
    fn test_rank_gap_warns() {
        let data = RankingData {
            meta: make_meta(2),
            rankings: vec![
                make_entry(1, "John", "Smith", "QB"),
                make_entry(5, "Jane", "Doe", "WR"),
            ],
        };

        let result = validate_ranking_data(&data);
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("Gaps")));
    }

    #[test]
    fn test_duplicate_rank_warns() {
        let data = RankingData {
            meta: make_meta(2),
            rankings: vec![
                make_entry(1, "John", "Smith", "QB"),
                make_entry(1, "Jane", "Doe", "WR"),
            ],
        };

        let result = validate_ranking_data(&data);
        assert!(result.valid); // Duplicate rank is a warning, not error
        assert!(result.warnings.iter().any(|w| w.contains("Duplicate rank")));
    }

    #[test]
    fn test_low_prospect_count_warns() {
        let data = RankingData {
            meta: make_meta(2),
            rankings: vec![
                make_entry(1, "John", "Smith", "QB"),
                make_entry(2, "Jane", "Doe", "WR"),
            ],
        };

        let result = validate_ranking_data(&data);
        assert!(result.valid);
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("Only 2 prospects")));
    }
}
