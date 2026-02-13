use std::collections::HashSet;

use anyhow::Result;

use crate::models::{RankingData, RankingMeta};

/// Normalize a name component for deduplication matching.
///
/// Strips periods, collapses whitespace, and lowercases.
fn normalize_name(name: &str) -> String {
    name.replace('.', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Build a lookup key from first and last name.
fn name_key(first: &str, last: &str) -> String {
    format!("{} {}", normalize_name(first), normalize_name(last))
}

/// Merge multiple ranking files into a single combined ranking.
///
/// The primary file provides the base rankings. Each secondary file is scanned
/// in order and any prospects whose normalized names are not already present
/// are appended with continuing rank numbers.
pub fn merge_rankings(primary: RankingData, secondaries: Vec<RankingData>) -> Result<RankingData> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let draft_year = primary.meta.draft_year;

    let mut seen: HashSet<String> = HashSet::new();
    let mut merged = Vec::new();

    // Add all primary entries
    for entry in primary.rankings {
        let key = name_key(&entry.first_name, &entry.last_name);
        seen.insert(key);
        merged.push(entry);
    }

    let mut next_rank = merged.len() as i32 + 1;

    // Append unique entries from each secondary file
    for secondary in secondaries {
        for mut entry in secondary.rankings {
            let key = name_key(&entry.first_name, &entry.last_name);
            if seen.insert(key) {
                entry.rank = next_rank;
                next_rank += 1;
                merged.push(entry);
            }
        }
    }

    let total = merged.len();

    Ok(RankingData {
        meta: RankingMeta {
            version: "1.0.0".to_string(),
            source: "merged".to_string(),
            source_url: "N/A".to_string(),
            draft_year,
            scraped_at: today,
            total_prospects: total,
        },
        rankings: merged,
    })
}

/// Load a ranking file from disk.
pub fn load_ranking_file(path: &str) -> Result<RankingData> {
    let content = std::fs::read_to_string(path)?;
    let data: RankingData = serde_json::from_str(&content)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RankingEntry, RankingMeta};

    fn make_meta(source: &str, year: i32, total: usize) -> RankingMeta {
        RankingMeta {
            version: "1.0.0".to_string(),
            source: source.to_string(),
            source_url: "N/A".to_string(),
            draft_year: year,
            scraped_at: "2026-02-13".to_string(),
            total_prospects: total,
        }
    }

    fn make_entry(rank: i32, first: &str, last: &str, pos: &str, school: &str) -> RankingEntry {
        RankingEntry {
            rank,
            first_name: first.to_string(),
            last_name: last.to_string(),
            position: pos.to_string(),
            school: school.to_string(),
        }
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("C.J."), "cj");
        assert_eq!(normalize_name("Jr."), "jr");
        assert_eq!(normalize_name("Fernando"), "fernando");
        assert_eq!(normalize_name("R. Mason"), "r mason");
        assert_eq!(normalize_name("  Extra   Spaces  "), "extra spaces");
    }

    #[test]
    fn test_merge_no_duplicates() {
        let primary = RankingData {
            meta: make_meta("primary", 2026, 2),
            rankings: vec![
                make_entry(1, "John", "Smith", "QB", "Alabama"),
                make_entry(2, "Jane", "Doe", "WR", "Ohio State"),
            ],
        };

        let secondary = RankingData {
            meta: make_meta("secondary", 2026, 2),
            rankings: vec![
                make_entry(1, "Bob", "Jones", "CB", "Georgia"),
                make_entry(2, "Alice", "Brown", "OT", "Michigan"),
            ],
        };

        let result = merge_rankings(primary, vec![secondary]).unwrap();
        assert_eq!(result.rankings.len(), 4);
        assert_eq!(result.meta.source, "merged");
        assert_eq!(result.meta.total_prospects, 4);

        // Verify rank numbering continues
        assert_eq!(result.rankings[2].rank, 3);
        assert_eq!(result.rankings[3].rank, 4);
    }

    #[test]
    fn test_merge_deduplicates_by_name() {
        let primary = RankingData {
            meta: make_meta("primary", 2026, 2),
            rankings: vec![
                make_entry(1, "John", "Smith", "QB", "Alabama"),
                make_entry(2, "Jane", "Doe", "WR", "Ohio State"),
            ],
        };

        let secondary = RankingData {
            meta: make_meta("secondary", 2026, 2),
            rankings: vec![
                make_entry(1, "John", "Smith", "QB", "Alabama"), // duplicate
                make_entry(2, "Bob", "Jones", "CB", "Georgia"),  // unique
            ],
        };

        let result = merge_rankings(primary, vec![secondary]).unwrap();
        assert_eq!(result.rankings.len(), 3);
    }

    #[test]
    fn test_merge_deduplicates_with_period_variations() {
        let primary = RankingData {
            meta: make_meta("primary", 2026, 1),
            rankings: vec![make_entry(1, "C.J.", "Stroud", "QB", "Ohio State")],
        };

        let secondary = RankingData {
            meta: make_meta("secondary", 2026, 1),
            rankings: vec![make_entry(1, "CJ", "Stroud", "QB", "Ohio State")],
        };

        let result = merge_rankings(primary, vec![secondary]).unwrap();
        assert_eq!(result.rankings.len(), 1);
    }

    #[test]
    fn test_merge_deduplicates_case_insensitive() {
        let primary = RankingData {
            meta: make_meta("primary", 2026, 1),
            rankings: vec![make_entry(1, "JOHN", "SMITH", "QB", "Alabama")],
        };

        let secondary = RankingData {
            meta: make_meta("secondary", 2026, 1),
            rankings: vec![make_entry(1, "john", "smith", "QB", "Alabama")],
        };

        let result = merge_rankings(primary, vec![secondary]).unwrap();
        assert_eq!(result.rankings.len(), 1);
    }

    #[test]
    fn test_merge_multiple_secondaries() {
        let primary = RankingData {
            meta: make_meta("primary", 2026, 1),
            rankings: vec![make_entry(1, "John", "Smith", "QB", "Alabama")],
        };

        let sec1 = RankingData {
            meta: make_meta("sec1", 2026, 1),
            rankings: vec![make_entry(1, "Jane", "Doe", "WR", "Ohio State")],
        };

        let sec2 = RankingData {
            meta: make_meta("sec2", 2026, 1),
            rankings: vec![make_entry(1, "Bob", "Jones", "CB", "Georgia")],
        };

        let result = merge_rankings(primary, vec![sec1, sec2]).unwrap();
        assert_eq!(result.rankings.len(), 3);
        assert_eq!(result.rankings[0].rank, 1);
        assert_eq!(result.rankings[1].rank, 2);
        assert_eq!(result.rankings[2].rank, 3);
    }

    #[test]
    fn test_merge_preserves_primary_draft_year() {
        let primary = RankingData {
            meta: make_meta("primary", 2026, 1),
            rankings: vec![make_entry(1, "John", "Smith", "QB", "Alabama")],
        };

        let secondary = RankingData {
            meta: make_meta("secondary", 2025, 1),
            rankings: vec![make_entry(1, "Jane", "Doe", "WR", "Ohio State")],
        };

        let result = merge_rankings(primary, vec![secondary]).unwrap();
        assert_eq!(result.meta.draft_year, 2026);
    }

    #[test]
    fn test_merge_empty_secondary() {
        let primary = RankingData {
            meta: make_meta("primary", 2026, 2),
            rankings: vec![
                make_entry(1, "John", "Smith", "QB", "Alabama"),
                make_entry(2, "Jane", "Doe", "WR", "Ohio State"),
            ],
        };

        let secondary = RankingData {
            meta: make_meta("secondary", 2026, 0),
            rankings: vec![],
        };

        let result = merge_rankings(primary, vec![secondary]).unwrap();
        assert_eq!(result.rankings.len(), 2);
    }
}
