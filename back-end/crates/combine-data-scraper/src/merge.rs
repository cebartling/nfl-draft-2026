use std::collections::HashMap;

use anyhow::Result;

use crate::models::{CombineData, CombineEntry, CombineMeta};

/// Clean a name component for deduplication matching.
///
/// Strips periods, Unicode curly quotes, collapses whitespace, and lowercases.
fn clean_name(name: &str) -> String {
    let cleaned = name
        .replace('.', "")
        .replace(['\u{2019}', '\u{2018}'], "'");

    cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Normalize a last name for deduplication matching.
///
/// Applies cleaning and strips generational suffixes (Jr, Sr, II, III, IV).
fn normalize_last_name(name: &str) -> String {
    let cleaned = name
        .replace('.', "")
        .replace(['\u{2019}', '\u{2018}'], "'");

    let mut parts: Vec<&str> = cleaned.split_whitespace().collect();

    if let Some(last) = parts.last().copied() {
        let last_lower = last.to_ascii_lowercase();
        match last_lower.as_str() {
            "jr" | "sr" | "ii" | "iii" | "iv" => {
                parts.pop();
            }
            _ => {}
        }
    }

    parts.join(" ").to_lowercase()
}

/// Build a lookup key from first and last name.
fn name_key(first: &str, last: &str) -> String {
    format!("{} {}", clean_name(first), normalize_last_name(last))
}

/// Merge combine data from multiple sources.
///
/// The primary source provides the base entries. For duplicate players (matched
/// by normalized name), any `None` fields in the primary entry are backfilled
/// from the secondary. Players unique to secondary sources are appended.
pub fn merge_combine_data(
    primary: CombineData,
    secondaries: Vec<CombineData>,
) -> Result<CombineData> {
    let year = primary.meta.year;
    let mut key_to_index: HashMap<String, usize> = HashMap::new();
    let mut merged: Vec<CombineEntry> = Vec::new();

    // Add all primary entries
    for entry in primary.combine_results {
        let key = name_key(&entry.first_name, &entry.last_name);
        key_to_index.insert(key, merged.len());
        merged.push(entry);
    }

    // Process secondaries
    for secondary in secondaries {
        for entry in secondary.combine_results {
            let key = name_key(&entry.first_name, &entry.last_name);

            if let Some(&idx) = key_to_index.get(&key) {
                // Duplicate: backfill None fields from secondary
                backfill_entry(&mut merged[idx], &entry);
            } else {
                // Unique to secondary: append
                key_to_index.insert(key, merged.len());
                merged.push(entry);
            }
        }
    }

    let entry_count = merged.len();

    Ok(CombineData {
        meta: CombineMeta {
            source: "merged".to_string(),
            description: format!("{} NFL Combine results (merged from multiple sources)", year),
            year,
            generated_at: chrono::Utc::now().to_rfc3339(),
            player_count: entry_count,
            entry_count,
        },
        combine_results: merged,
    })
}

/// Backfill None fields in `target` from `source`.
fn backfill_entry(target: &mut CombineEntry, source: &CombineEntry) {
    if target.forty_yard_dash.is_none() {
        target.forty_yard_dash = source.forty_yard_dash;
    }
    if target.bench_press.is_none() {
        target.bench_press = source.bench_press;
    }
    if target.vertical_jump.is_none() {
        target.vertical_jump = source.vertical_jump;
    }
    if target.broad_jump.is_none() {
        target.broad_jump = source.broad_jump;
    }
    if target.three_cone_drill.is_none() {
        target.three_cone_drill = source.three_cone_drill;
    }
    if target.twenty_yard_shuttle.is_none() {
        target.twenty_yard_shuttle = source.twenty_yard_shuttle;
    }
    if target.arm_length.is_none() {
        target.arm_length = source.arm_length;
    }
    if target.hand_size.is_none() {
        target.hand_size = source.hand_size;
    }
    if target.wingspan.is_none() {
        target.wingspan = source.wingspan;
    }
    if target.ten_yard_split.is_none() {
        target.ten_yard_split = source.ten_yard_split;
    }
    if target.twenty_yard_split.is_none() {
        target.twenty_yard_split = source.twenty_yard_split;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta(source: &str, year: i32) -> CombineMeta {
        CombineMeta {
            source: source.to_string(),
            description: "test".to_string(),
            year,
            generated_at: "2026-03-10".to_string(),
            player_count: 0,
            entry_count: 0,
        }
    }

    fn make_entry(first: &str, last: &str, pos: &str) -> CombineEntry {
        CombineEntry {
            first_name: first.to_string(),
            last_name: last.to_string(),
            position: pos.to_string(),
            source: "combine".to_string(),
            year: 2026,
            forty_yard_dash: None,
            bench_press: None,
            vertical_jump: None,
            broad_jump: None,
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            arm_length: None,
            hand_size: None,
            wingspan: None,
            ten_yard_split: None,
            twenty_yard_split: None,
        }
    }

    #[test]
    fn test_merge_backfills_missing_fields() {
        let mut pfr_entry = make_entry("Cam", "Ward", "QB");
        pfr_entry.forty_yard_dash = Some(4.72);
        // arm_length is None

        let mut md_entry = make_entry("Cam", "Ward", "QB");
        md_entry.forty_yard_dash = Some(4.75); // different value, should NOT overwrite
        md_entry.arm_length = Some(32.5); // should backfill

        let primary = CombineData {
            meta: make_meta("pfr", 2026),
            combine_results: vec![pfr_entry],
        };

        let secondary = CombineData {
            meta: make_meta("mockdraftable", 2026),
            combine_results: vec![md_entry],
        };

        let result = merge_combine_data(primary, vec![secondary]).unwrap();
        assert_eq!(result.combine_results.len(), 1);

        let merged = &result.combine_results[0];
        assert_eq!(merged.forty_yard_dash, Some(4.72)); // kept primary
        assert_eq!(merged.arm_length, Some(32.5)); // backfilled from secondary
    }

    #[test]
    fn test_merge_appends_unique_players() {
        let primary = CombineData {
            meta: make_meta("pfr", 2026),
            combine_results: vec![make_entry("Cam", "Ward", "QB")],
        };

        let secondary = CombineData {
            meta: make_meta("mockdraftable", 2026),
            combine_results: vec![make_entry("Travis", "Hunter", "CB")],
        };

        let result = merge_combine_data(primary, vec![secondary]).unwrap();
        assert_eq!(result.combine_results.len(), 2);
        assert_eq!(result.combine_results[1].first_name, "Travis");
    }

    #[test]
    fn test_merge_name_with_suffix_variations() {
        let mut pfr_entry = make_entry("Fernando", "Carmona Jr.", "OT");
        pfr_entry.forty_yard_dash = Some(5.15);

        let mut md_entry = make_entry("Fernando", "Carmona", "OT");
        md_entry.arm_length = Some(34.0);

        let primary = CombineData {
            meta: make_meta("pfr", 2026),
            combine_results: vec![pfr_entry],
        };

        let secondary = CombineData {
            meta: make_meta("mockdraftable", 2026),
            combine_results: vec![md_entry],
        };

        let result = merge_combine_data(primary, vec![secondary]).unwrap();
        assert_eq!(result.combine_results.len(), 1);
        assert_eq!(result.combine_results[0].arm_length, Some(34.0));
    }

    #[test]
    fn test_merge_case_insensitive() {
        let primary = CombineData {
            meta: make_meta("pfr", 2026),
            combine_results: vec![make_entry("CAM", "WARD", "QB")],
        };

        let secondary = CombineData {
            meta: make_meta("mockdraftable", 2026),
            combine_results: vec![make_entry("cam", "ward", "QB")],
        };

        let result = merge_combine_data(primary, vec![secondary]).unwrap();
        assert_eq!(result.combine_results.len(), 1);
    }

    #[test]
    fn test_merge_cj_period_variations() {
        let primary = CombineData {
            meta: make_meta("pfr", 2026),
            combine_results: vec![make_entry("C.J.", "Stroud", "QB")],
        };

        let secondary = CombineData {
            meta: make_meta("mockdraftable", 2026),
            combine_results: vec![make_entry("CJ", "Stroud", "QB")],
        };

        let result = merge_combine_data(primary, vec![secondary]).unwrap();
        assert_eq!(result.combine_results.len(), 1);
    }

    #[test]
    fn test_merge_empty_secondaries() {
        let primary = CombineData {
            meta: make_meta("pfr", 2026),
            combine_results: vec![make_entry("Cam", "Ward", "QB")],
        };

        let result = merge_combine_data(primary, vec![]).unwrap();
        assert_eq!(result.combine_results.len(), 1);
    }

    #[test]
    fn test_merge_meta_is_merged() {
        let primary = CombineData {
            meta: make_meta("pfr", 2026),
            combine_results: vec![make_entry("Cam", "Ward", "QB")],
        };

        let result = merge_combine_data(primary, vec![]).unwrap();
        assert_eq!(result.meta.source, "merged");
        assert_eq!(result.meta.year, 2026);
    }

    #[test]
    fn test_merge_will_lee_iii() {
        let primary = CombineData {
            meta: make_meta("pfr", 2026),
            combine_results: vec![make_entry("Will", "Lee III", "CB")],
        };

        let secondary = CombineData {
            meta: make_meta("mockdraftable", 2026),
            combine_results: vec![make_entry("Will", "Lee", "CB")],
        };

        let result = merge_combine_data(primary, vec![secondary]).unwrap();
        assert_eq!(result.combine_results.len(), 1);
    }

    #[test]
    fn test_name_key_function() {
        assert_eq!(name_key("C.J.", "Stroud"), "cj stroud");
        assert_eq!(name_key("Fernando", "Carmona Jr."), "fernando carmona");
        assert_eq!(name_key("Will", "Lee III"), "will lee");
        assert_eq!(name_key("CAM", "WARD"), "cam ward");
    }

    #[test]
    fn test_merge_does_not_overwrite_existing_fields() {
        let mut pfr_entry = make_entry("Cam", "Ward", "QB");
        pfr_entry.forty_yard_dash = Some(4.72);
        pfr_entry.bench_press = Some(18);
        pfr_entry.arm_length = Some(32.5);

        let mut md_entry = make_entry("Cam", "Ward", "QB");
        md_entry.forty_yard_dash = Some(4.75); // different — should NOT overwrite
        md_entry.bench_press = Some(20); // different — should NOT overwrite
        md_entry.arm_length = Some(33.0); // different — should NOT overwrite
        md_entry.hand_size = Some(9.75); // new — should backfill

        let primary = CombineData {
            meta: make_meta("pfr", 2026),
            combine_results: vec![pfr_entry],
        };

        let secondary = CombineData {
            meta: make_meta("mockdraftable", 2026),
            combine_results: vec![md_entry],
        };

        let result = merge_combine_data(primary, vec![secondary]).unwrap();
        let merged = &result.combine_results[0];
        assert_eq!(merged.forty_yard_dash, Some(4.72)); // kept primary
        assert_eq!(merged.bench_press, Some(18)); // kept primary
        assert_eq!(merged.arm_length, Some(32.5)); // kept primary
        assert_eq!(merged.hand_size, Some(9.75)); // backfilled from secondary
    }
}
