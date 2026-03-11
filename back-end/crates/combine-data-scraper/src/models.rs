use serde::{Deserialize, Serialize};

/// A single combine result entry, compatible with the seed-data combine_loader pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombineEntry {
    pub first_name: String,
    pub last_name: String,
    pub position: String,
    pub source: String,
    pub year: i32,
    pub forty_yard_dash: Option<f64>,
    pub bench_press: Option<i32>,
    pub vertical_jump: Option<f64>,
    pub broad_jump: Option<i32>,
    pub three_cone_drill: Option<f64>,
    pub twenty_yard_shuttle: Option<f64>,
    pub arm_length: Option<f64>,
    pub hand_size: Option<f64>,
    pub wingspan: Option<f64>,
    pub ten_yard_split: Option<f64>,
    pub twenty_yard_split: Option<f64>,
}

/// Top-level combine data structure matching the combine_loader expected format.
#[derive(Debug, Serialize, Deserialize)]
pub struct CombineData {
    pub meta: CombineMeta,
    pub combine_results: Vec<CombineEntry>,
}

/// Metadata for a combine data file.
#[derive(Debug, Serialize, Deserialize)]
pub struct CombineMeta {
    pub source: String,
    #[serde(default)]
    pub description: String,
    pub year: i32,
    #[serde(default)]
    pub generated_at: String,
    #[serde(default)]
    pub player_count: usize,
    #[serde(default)]
    pub entry_count: usize,
}

/// Normalize a position abbreviation from various sources (PFR, Mockdraftable)
/// to the canonical values used in the player database.
pub fn normalize_position(pos: &str) -> String {
    match pos.trim().to_uppercase().as_str() {
        "DE" | "OLB" | "EDGE" | "EDGE/LB" | "LB/EDGE" => "EDGE".to_string(),
        "ILB" | "MLB" => "LB".to_string(),
        "NT" => "DT".to_string(),
        "C" | "OG" | "G" => "IOL".to_string(),
        "FS" | "SS" | "DB" | "SAF" => "S".to_string(),
        "FB" | "HB" => "RB".to_string(),
        "T" | "OT" => "OT".to_string(),
        "QB" | "WR" | "TE" | "RB" | "CB" | "DL" | "DT" | "LB" | "S" | "IOL" | "K" | "P" => {
            pos.trim().to_uppercase()
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_entry_construction_with_source() {
        let entry = CombineEntry {
            first_name: "Cam".to_string(),
            last_name: "Ward".to_string(),
            position: "QB".to_string(),
            source: "pro_football_reference".to_string(),
            year: 2026,
            forty_yard_dash: Some(4.72),
            bench_press: Some(18),
            vertical_jump: Some(32.0),
            broad_jump: Some(108),
            three_cone_drill: Some(7.05),
            twenty_yard_shuttle: Some(4.30),
            arm_length: Some(32.5),
            hand_size: Some(9.75),
            wingspan: Some(77.5),
            ten_yard_split: Some(1.65),
            twenty_yard_split: Some(2.72),
        };

        assert_eq!(entry.source, "pro_football_reference");
        assert_eq!(entry.forty_yard_dash, Some(4.72));
        assert_eq!(entry.bench_press, Some(18));
        assert_eq!(entry.vertical_jump, Some(32.0));
        assert_eq!(entry.broad_jump, Some(108));
        assert_eq!(entry.three_cone_drill, Some(7.05));
        assert_eq!(entry.twenty_yard_shuttle, Some(4.30));
        assert_eq!(entry.arm_length, Some(32.5));
        assert_eq!(entry.hand_size, Some(9.75));
        assert_eq!(entry.wingspan, Some(77.5));
        assert_eq!(entry.ten_yard_split, Some(1.65));
        assert_eq!(entry.twenty_yard_split, Some(2.72));
    }

    #[test]
    fn test_combine_entry_serde_roundtrip() {
        let entry = CombineEntry {
            first_name: "Travis".to_string(),
            last_name: "Hunter".to_string(),
            position: "CB".to_string(),
            source: "pro_football_reference".to_string(),
            year: 2026,
            forty_yard_dash: Some(4.38),
            bench_press: None,
            vertical_jump: Some(40.5),
            broad_jump: Some(130),
            three_cone_drill: None,
            twenty_yard_shuttle: None,
            arm_length: None,
            hand_size: None,
            wingspan: None,
            ten_yard_split: None,
            twenty_yard_split: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: CombineEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_combine_data_serde_roundtrip() {
        let data = CombineData {
            meta: CombineMeta {
                source: "pro_football_reference".to_string(),
                description: "2026 NFL Combine results".to_string(),
                year: 2026,
                generated_at: "2026-03-10".to_string(),
                player_count: 1,
                entry_count: 1,
            },
            combine_results: vec![CombineEntry {
                first_name: "Cam".to_string(),
                last_name: "Ward".to_string(),
                position: "QB".to_string(),
                source: "combine".to_string(),
                year: 2026,
                forty_yard_dash: Some(4.72),
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
            }],
        };

        let json = serde_json::to_string_pretty(&data).unwrap();
        let deserialized: CombineData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.meta.source, "pro_football_reference");
        assert_eq!(deserialized.combine_results.len(), 1);
        assert_eq!(deserialized.combine_results[0].first_name, "Cam");
    }

    // Position normalization tests
    #[test]
    fn test_normalize_position_edge_mappings() {
        assert_eq!(normalize_position("DE"), "EDGE");
        assert_eq!(normalize_position("OLB"), "EDGE");
        assert_eq!(normalize_position("EDGE"), "EDGE");
        assert_eq!(normalize_position("EDGE/LB"), "EDGE");
    }

    #[test]
    fn test_normalize_position_interior_mappings() {
        assert_eq!(normalize_position("ILB"), "LB");
        assert_eq!(normalize_position("MLB"), "LB");
        assert_eq!(normalize_position("NT"), "DT");
        assert_eq!(normalize_position("C"), "IOL");
        assert_eq!(normalize_position("OG"), "IOL");
        assert_eq!(normalize_position("G"), "IOL");
    }

    #[test]
    fn test_normalize_position_safety_mappings() {
        assert_eq!(normalize_position("FS"), "S");
        assert_eq!(normalize_position("SS"), "S");
        assert_eq!(normalize_position("DB"), "S");
    }

    #[test]
    fn test_normalize_position_backfield_mappings() {
        assert_eq!(normalize_position("FB"), "RB");
        assert_eq!(normalize_position("HB"), "RB");
    }

    #[test]
    fn test_normalize_position_passthrough() {
        assert_eq!(normalize_position("QB"), "QB");
        assert_eq!(normalize_position("WR"), "WR");
        assert_eq!(normalize_position("TE"), "TE");
        assert_eq!(normalize_position("OT"), "OT");
        assert_eq!(normalize_position("CB"), "CB");
        assert_eq!(normalize_position("DL"), "DL");
        assert_eq!(normalize_position("RB"), "RB");
        assert_eq!(normalize_position("S"), "S");
        assert_eq!(normalize_position("LB"), "LB");
    }

    #[test]
    fn test_normalize_position_case_insensitive() {
        assert_eq!(normalize_position("de"), "EDGE");
        assert_eq!(normalize_position("qb"), "QB");
        assert_eq!(normalize_position("ilb"), "LB");
    }

    #[test]
    fn test_normalize_position_trims_whitespace() {
        assert_eq!(normalize_position(" DE "), "EDGE");
        assert_eq!(normalize_position("  QB  "), "QB");
    }
}
