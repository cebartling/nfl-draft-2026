use std::collections::HashSet;

use crate::feldman_freak_loader::FreaksData;

#[derive(Debug)]
pub struct FreaksValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl FreaksValidationResult {
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

pub fn validate_freaks_data(data: &FreaksData) -> FreaksValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Meta validation
    if data.meta.source.is_empty() {
        errors.push("Meta source is empty".to_string());
    }

    if data.meta.year < 2020 || data.meta.year > 2030 {
        errors.push(format!(
            "Year {} is out of reasonable range (2020-2030)",
            data.meta.year
        ));
    }

    if data.freaks.is_empty() {
        errors.push("No freaks entries found".to_string());
    }

    // Entry validation
    let mut seen_ranks = HashSet::new();
    let mut seen_names = HashSet::new();

    for (i, entry) in data.freaks.iter().enumerate() {
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
            errors.push(format!("Entry {}: duplicate rank {}", i + 1, entry.rank));
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

        // Description must not be empty
        if entry.description.trim().is_empty() {
            errors.push(format!(
                "Entry {}: empty description for {} {}",
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

        // College should be provided
        if entry.college.trim().is_empty() {
            warnings.push(format!(
                "Entry {}: empty college for {} {}",
                i + 1,
                entry.first_name,
                entry.last_name
            ));
        }
    }

    let valid = errors.is_empty();
    FreaksValidationResult {
        valid,
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feldman_freak_loader::{FreakEntry, FreaksMeta};

    fn make_data(freaks: Vec<FreakEntry>) -> FreaksData {
        FreaksData {
            meta: FreaksMeta {
                year: 2026,
                source: "Bruce Feldman's Freaks List".to_string(),
                article_url: "https://example.com/freaks".to_string(),
            },
            freaks,
        }
    }

    #[test]
    fn test_valid_data() {
        let data = make_data(vec![FreakEntry {
            rank: 1,
            first_name: "Kenyon".to_string(),
            last_name: "Sadiq".to_string(),
            college: "Oregon".to_string(),
            position: "TE".to_string(),
            description: "Vertical jumped 41.5 inches".to_string(),
        }]);

        let result = validate_freaks_data(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_empty_freaks() {
        let data = make_data(vec![]);
        let result = validate_freaks_data(&data);
        assert!(!result.valid);
    }

    #[test]
    fn test_duplicate_rank() {
        let data = make_data(vec![
            FreakEntry {
                rank: 1,
                first_name: "Kenyon".to_string(),
                last_name: "Sadiq".to_string(),
                college: "Oregon".to_string(),
                position: "TE".to_string(),
                description: "Vertical jumped 41.5 inches".to_string(),
            },
            FreakEntry {
                rank: 1,
                first_name: "Sonny".to_string(),
                last_name: "Styles".to_string(),
                college: "Ohio State".to_string(),
                position: "LB".to_string(),
                description: "Broad jumped 11 feet".to_string(),
            },
        ]);

        let result = validate_freaks_data(&data);
        assert!(!result.valid);
    }

    #[test]
    fn test_empty_description() {
        let data = make_data(vec![FreakEntry {
            rank: 1,
            first_name: "Kenyon".to_string(),
            last_name: "Sadiq".to_string(),
            college: "Oregon".to_string(),
            position: "TE".to_string(),
            description: "".to_string(),
        }]);

        let result = validate_freaks_data(&data);
        assert!(!result.valid);
    }

    #[test]
    fn test_negative_rank() {
        let data = make_data(vec![FreakEntry {
            rank: -1,
            first_name: "Kenyon".to_string(),
            last_name: "Sadiq".to_string(),
            college: "Oregon".to_string(),
            position: "TE".to_string(),
            description: "Vertical jumped 41.5 inches".to_string(),
        }]);

        let result = validate_freaks_data(&data);
        assert!(!result.valid);
    }

    #[test]
    fn test_empty_first_name() {
        let data = make_data(vec![FreakEntry {
            rank: 1,
            first_name: "".to_string(),
            last_name: "Sadiq".to_string(),
            college: "Oregon".to_string(),
            position: "TE".to_string(),
            description: "Vertical jumped 41.5 inches".to_string(),
        }]);

        let result = validate_freaks_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("empty first name")));
    }

    #[test]
    fn test_empty_last_name() {
        let data = make_data(vec![FreakEntry {
            rank: 1,
            first_name: "Kenyon".to_string(),
            last_name: "  ".to_string(),
            college: "Oregon".to_string(),
            position: "TE".to_string(),
            description: "Vertical jumped 41.5 inches".to_string(),
        }]);

        let result = validate_freaks_data(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("empty last name")));
    }

    #[test]
    fn test_empty_position() {
        let data = make_data(vec![FreakEntry {
            rank: 1,
            first_name: "Kenyon".to_string(),
            last_name: "Sadiq".to_string(),
            college: "Oregon".to_string(),
            position: "".to_string(),
            description: "Vertical jumped 41.5 inches".to_string(),
        }]);

        let result = validate_freaks_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("empty position")));
    }

    #[test]
    fn test_empty_college_warns() {
        let data = make_data(vec![FreakEntry {
            rank: 1,
            first_name: "Kenyon".to_string(),
            last_name: "Sadiq".to_string(),
            college: "".to_string(),
            position: "TE".to_string(),
            description: "Vertical jumped 41.5 inches".to_string(),
        }]);

        let result = validate_freaks_data(&data);
        // Empty college is a warning, not an error
        assert!(result.valid);
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("empty college")));
    }

    #[test]
    fn test_empty_source() {
        let data = FreaksData {
            meta: FreaksMeta {
                year: 2026,
                source: "".to_string(),
                article_url: "https://example.com/freaks".to_string(),
            },
            freaks: vec![FreakEntry {
                rank: 1,
                first_name: "Kenyon".to_string(),
                last_name: "Sadiq".to_string(),
                college: "Oregon".to_string(),
                position: "TE".to_string(),
                description: "Vertical jumped 41.5 inches".to_string(),
            }],
        };

        let result = validate_freaks_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("source is empty")));
    }

    #[test]
    fn test_year_out_of_range() {
        let data = FreaksData {
            meta: FreaksMeta {
                year: 2050,
                source: "Bruce Feldman's Freaks List".to_string(),
                article_url: "https://example.com/freaks".to_string(),
            },
            freaks: vec![FreakEntry {
                rank: 1,
                first_name: "Kenyon".to_string(),
                last_name: "Sadiq".to_string(),
                college: "Oregon".to_string(),
                position: "TE".to_string(),
                description: "Vertical jumped 41.5 inches".to_string(),
            }],
        };

        let result = validate_freaks_data(&data);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("out of reasonable range")));
    }
}
