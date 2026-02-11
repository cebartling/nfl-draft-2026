use std::collections::HashSet;

use crate::scouting_report_loader::RankingData;

#[derive(Debug)]
pub struct RankingsValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl RankingsValidationResult {
    pub fn print_summary(&self) {
        if self.valid {
            println!("Validation: PASSED");
        } else {
            println!("Validation: FAILED");
        }

        if !self.warnings.is_empty() {
            println!("\nWarnings ({}):", self.warnings.len());
            for w in &self.warnings {
                println!("  - {}", w);
            }
        }

        if !self.errors.is_empty() {
            println!("\nErrors ({}):", self.errors.len());
            for e in &self.errors {
                println!("  - {}", e);
            }
        }
    }
}

pub fn validate_ranking_data(data: &RankingData) -> RankingsValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Meta validation
    if data.meta.source.is_empty() {
        errors.push("Meta source is empty".to_string());
    }

    if data.meta.draft_year < 2020 || data.meta.draft_year > 2030 {
        errors.push(format!(
            "Draft year {} is out of reasonable range (2020-2030)",
            data.meta.draft_year
        ));
    }

    if data.rankings.is_empty() {
        errors.push("No rankings entries found".to_string());
    }

    // Check total_prospects matches
    if data.meta.total_prospects != data.rankings.len() {
        warnings.push(format!(
            "Meta total_prospects ({}) doesn't match actual count ({})",
            data.meta.total_prospects,
            data.rankings.len()
        ));
    }

    // Entry validation
    let mut seen_ranks = HashSet::new();
    let mut seen_names = HashSet::new();

    for (i, entry) in data.rankings.iter().enumerate() {
        // Rank must be positive
        if entry.rank <= 0 {
            errors.push(format!(
                "Entry {}: rank must be positive, got {}",
                i + 1,
                entry.rank
            ));
        }

        // Check for duplicate ranks
        if !seen_ranks.insert(entry.rank) {
            warnings.push(format!(
                "Entry {}: duplicate rank {}",
                i + 1,
                entry.rank
            ));
        }

        // Names must not be empty
        if entry.first_name.trim().is_empty() {
            errors.push(format!("Entry {}: empty first name", i + 1));
        }
        if entry.last_name.trim().is_empty() {
            errors.push(format!("Entry {}: empty last name", i + 1));
        }

        // Check for duplicate names
        let name_key = format!(
            "{}-{}",
            entry.first_name.to_lowercase(),
            entry.last_name.to_lowercase()
        );
        if !seen_names.insert(name_key) {
            warnings.push(format!(
                "Entry {}: duplicate name {} {}",
                i + 1,
                entry.first_name,
                entry.last_name
            ));
        }

        // Position must not be empty
        if entry.position.trim().is_empty() {
            errors.push(format!(
                "Entry {}: empty position for {} {}",
                i + 1,
                entry.first_name,
                entry.last_name
            ));
        }

        // School must not be empty
        if entry.school.trim().is_empty() {
            errors.push(format!(
                "Entry {}: empty school for {} {}",
                i + 1,
                entry.first_name,
                entry.last_name
            ));
        }
    }

    let valid = errors.is_empty();
    RankingsValidationResult {
        valid,
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scouting_report_loader::{RankingEntry, RankingMeta};

    fn make_data(rankings: Vec<RankingEntry>) -> RankingData {
        let count = rankings.len();
        RankingData {
            meta: RankingMeta {
                version: "1.0.0".to_string(),
                source: "test".to_string(),
                source_url: "https://example.com".to_string(),
                draft_year: 2026,
                scraped_at: "2026-02-11".to_string(),
                total_prospects: count,
            },
            rankings,
        }
    }

    #[test]
    fn test_valid_data() {
        let data = make_data(vec![
            RankingEntry {
                rank: 1,
                first_name: "Test".to_string(),
                last_name: "Player".to_string(),
                position: "QB".to_string(),
                school: "Alabama".to_string(),
            },
        ]);

        let result = validate_ranking_data(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_empty_rankings() {
        let data = make_data(vec![]);
        let result = validate_ranking_data(&data);
        assert!(!result.valid);
    }

    #[test]
    fn test_negative_rank() {
        let data = make_data(vec![
            RankingEntry {
                rank: -1,
                first_name: "Test".to_string(),
                last_name: "Player".to_string(),
                position: "QB".to_string(),
                school: "Alabama".to_string(),
            },
        ]);

        let result = validate_ranking_data(&data);
        assert!(!result.valid);
    }

    #[test]
    fn test_empty_name() {
        let data = make_data(vec![
            RankingEntry {
                rank: 1,
                first_name: "".to_string(),
                last_name: "Player".to_string(),
                position: "QB".to_string(),
                school: "Alabama".to_string(),
            },
        ]);

        let result = validate_ranking_data(&data);
        assert!(!result.valid);
    }
}
