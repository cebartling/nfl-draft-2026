use anyhow::{Context, Result};
use serde::Deserialize;

use crate::combine_loader::{CombineFileData, CombineFileEntry, CombineFileMeta};
use crate::position_mapper::map_position;

/// A row from the nflverse combine CSV file.
/// Fields match the CSV header: season,draft_year,draft_team,draft_round,draft_ovr,
/// pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
#[derive(Debug, Deserialize)]
pub struct NflverseCombineRow {
    pub season: i32,
    #[serde(default)]
    pub draft_year: Option<String>,
    #[serde(default)]
    pub draft_team: Option<String>,
    #[serde(default)]
    pub draft_round: Option<String>,
    #[serde(default)]
    pub draft_ovr: Option<String>,
    #[serde(default)]
    pub pfr_id: Option<String>,
    #[serde(default)]
    pub cfb_id: Option<String>,
    pub player_name: String,
    pub pos: String,
    #[serde(default)]
    pub school: Option<String>,
    #[serde(default)]
    pub ht: Option<String>,
    #[serde(default)]
    pub wt: Option<String>,
    #[serde(default)]
    pub forty: Option<f64>,
    #[serde(default)]
    pub bench: Option<i32>,
    #[serde(default)]
    pub vertical: Option<f64>,
    #[serde(default)]
    pub broad_jump: Option<i32>,
    #[serde(default)]
    pub cone: Option<f64>,
    #[serde(default)]
    pub shuttle: Option<f64>,
}

/// Statistics from a conversion run.
pub struct ConversionStats {
    pub converted: usize,
    pub skipped_position: usize,
    pub skipped_name: usize,
    pub total_rows: usize,
    pub warnings: Vec<String>,
}

impl ConversionStats {
    pub fn print_summary(&self) {
        println!("\nConversion Summary:");
        println!("  Total CSV rows:    {}", self.total_rows);
        println!("  Converted:         {}", self.converted);
        println!("  Skipped (position): {}", self.skipped_position);
        println!("  Skipped (name):    {}", self.skipped_name);
        if !self.warnings.is_empty() {
            println!("\nWarnings:");
            for w in &self.warnings {
                println!("  - {}", w);
            }
        }
    }
}

/// Split a full player name into (first_name, last_name).
///
/// Splits on the first space. Suffixes like "Jr.", "III", "II", "IV", "V"
/// are kept with the last name.
///
/// Examples:
///   "Cam Ward" → ("Cam", "Ward")
///   "Travis Hunter Jr." → ("Travis", "Hunter Jr.")
///   "BJ Adams" → ("BJ", "Adams")
///   "De'Rickey Wright" → ("De'Rickey", "Wright")
pub fn split_player_name(full_name: &str) -> Option<(String, String)> {
    let trimmed = full_name.trim();
    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    if parts.len() < 2 {
        return None;
    }
    let first = parts[0].to_string();
    let last = parts[1].trim().to_string();
    if last.is_empty() {
        return None;
    }
    Some((first, last))
}

/// Convert a collection of nflverse CSV rows into the combine JSON format.
///
/// Filters by year, maps positions, splits names, and produces the
/// `CombineFileData` struct that the existing `combine_loader` expects.
pub fn convert_nflverse_to_combine_json(
    rows: Vec<NflverseCombineRow>,
    year: i32,
    source: &str,
) -> (CombineFileData, ConversionStats) {
    let filtered: Vec<_> = rows.into_iter().filter(|r| r.season == year).collect();
    let total_rows = filtered.len();
    let mut entries = Vec::new();
    let mut skipped_position = 0;
    let mut skipped_name = 0;
    let mut warnings = Vec::new();

    for row in filtered {
        // Split name
        let (first_name, last_name) = match split_player_name(&row.player_name) {
            Some(names) => names,
            None => {
                skipped_name += 1;
                warnings.push(format!(
                    "Could not split name: '{}'",
                    row.player_name
                ));
                continue;
            }
        };

        // Validate position
        if let Err(_) = map_position(&row.pos) {
            skipped_position += 1;
            warnings.push(format!(
                "Unmapped position '{}' for {} {}",
                row.pos, first_name, last_name
            ));
            continue;
        }

        // Map the position to canonical form for the JSON output.
        // Position derives Debug which gives us the variant name (e.g., "QB", "S").
        let canonical_pos = format!("{:?}", map_position(&row.pos).unwrap());

        let entry = CombineFileEntry {
            first_name,
            last_name,
            position: canonical_pos,
            source: source.to_string(),
            year,
            forty_yard_dash: row.forty,
            bench_press: row.bench,
            vertical_jump: row.vertical,
            broad_jump: row.broad_jump,
            three_cone_drill: row.cone,
            twenty_yard_shuttle: row.shuttle,
            arm_length: None,
            hand_size: None,
            wingspan: None,
            ten_yard_split: None,
            twenty_yard_split: None,
        };

        entries.push(entry);
    }

    let converted = entries.len();

    let data = CombineFileData {
        meta: CombineFileMeta {
            source: "nflverse".to_string(),
            year,
        },
        combine_results: entries,
    };

    let stats = ConversionStats {
        converted,
        skipped_position,
        skipped_name,
        total_rows,
        warnings,
    };

    (data, stats)
}

/// Parse nflverse combine CSV content from a reader.
pub fn parse_nflverse_csv<R: std::io::Read>(reader: R) -> Result<Vec<NflverseCombineRow>> {
    let mut csv_reader = csv::Reader::from_reader(reader);
    let mut rows = Vec::new();
    for result in csv_reader.deserialize() {
        let row: NflverseCombineRow =
            result.context("Failed to parse CSV row")?;
        rows.push(row);
    }
    Ok(rows)
}

/// Read nflverse CSV from a file path and convert to combine JSON format.
pub fn convert_csv_file(
    input_path: &str,
    year: i32,
    source: &str,
) -> Result<(CombineFileData, ConversionStats)> {
    let file = std::fs::File::open(input_path)
        .with_context(|| format!("Failed to open CSV file: {}", input_path))?;
    let rows = parse_nflverse_csv(file)?;
    Ok(convert_nflverse_to_combine_json(rows, year, source))
}

/// Write the combine JSON data to a file.
pub fn write_combine_json(data: &CombineFileData, output_path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(data)
        .context("Failed to serialize combine data to JSON")?;
    std::fs::write(output_path, json)
        .with_context(|| format!("Failed to write output file: {}", output_path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_simple_name() {
        let (first, last) = split_player_name("Cam Ward").unwrap();
        assert_eq!(first, "Cam");
        assert_eq!(last, "Ward");
    }

    #[test]
    fn test_split_name_with_suffix() {
        let (first, last) = split_player_name("Travis Hunter Jr.").unwrap();
        assert_eq!(first, "Travis");
        assert_eq!(last, "Hunter Jr.");
    }

    #[test]
    fn test_split_name_with_apostrophe() {
        let (first, last) = split_player_name("De'Rickey Wright").unwrap();
        assert_eq!(first, "De'Rickey");
        assert_eq!(last, "Wright");
    }

    #[test]
    fn test_split_name_with_numeral_suffix() {
        let (first, last) = split_player_name("Rueben Bain III").unwrap();
        assert_eq!(first, "Rueben");
        assert_eq!(last, "Bain III");
    }

    #[test]
    fn test_split_single_word_name_returns_none() {
        assert!(split_player_name("Pelé").is_none());
    }

    #[test]
    fn test_split_empty_name_returns_none() {
        assert!(split_player_name("").is_none());
    }

    #[test]
    fn test_split_name_with_whitespace() {
        let (first, last) = split_player_name("  BJ  Adams  ").unwrap();
        assert_eq!(first, "BJ");
        assert_eq!(last, "Adams");
    }

    #[test]
    fn test_parse_csv() {
        let csv_data = "\
season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,,,,,,,BJ Adams,CB,Central Florida,6-2,182,4.53,,32.5,117,,
2025,,,,,,darius-alexander-1,Darius Alexander,DT,Toledo,6-4,305,4.95,28,31.5,111,7.6,4.79";

        let rows = parse_nflverse_csv(csv_data.as_bytes()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].player_name, "BJ Adams");
        assert_eq!(rows[0].pos, "CB");
        assert_eq!(rows[0].forty, Some(4.53));
        assert!(rows[0].bench.is_none());
        assert_eq!(rows[1].bench, Some(28));
        assert_eq!(rows[1].cone, Some(7.6));
    }

    #[test]
    fn test_convert_filters_by_year() {
        let csv_data = "\
season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2024,,,,,,,John Doe,QB,Alabama,6-2,220,4.72,,,,,
2025,,,,,,,BJ Adams,CB,Central Florida,6-2,182,4.53,,32.5,117,,";

        let rows = parse_nflverse_csv(csv_data.as_bytes()).unwrap();
        let (data, stats) = convert_nflverse_to_combine_json(rows, 2025, "combine");

        assert_eq!(stats.total_rows, 1);
        assert_eq!(stats.converted, 1);
        assert_eq!(data.combine_results.len(), 1);
        assert_eq!(data.combine_results[0].first_name, "BJ");
        assert_eq!(data.combine_results[0].last_name, "Adams");
    }

    #[test]
    fn test_convert_skips_unmapped_positions() {
        let csv_data = "\
season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,,,,,,,Jake Smith,LS,Ohio State,6-2,240,4.80,,30.0,105,,";

        let rows = parse_nflverse_csv(csv_data.as_bytes()).unwrap();
        let (data, stats) = convert_nflverse_to_combine_json(rows, 2025, "combine");

        assert_eq!(stats.total_rows, 1);
        assert_eq!(stats.converted, 0);
        assert_eq!(stats.skipped_position, 1);
        assert!(data.combine_results.is_empty());
    }

    #[test]
    fn test_convert_maps_nflverse_positions() {
        let csv_data = "\
season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,,,,,,,John Doe,DB,Alabama,6-1,195,4.45,,35.0,120,,
2025,,,,,,,Jane Doe,SAF,Oregon,6-0,205,4.50,,33.0,118,,
2025,,,,,,,Bob Smith,FB,Michigan,6-0,245,4.70,25,30.0,105,,";

        let rows = parse_nflverse_csv(csv_data.as_bytes()).unwrap();
        let (data, stats) = convert_nflverse_to_combine_json(rows, 2025, "combine");

        assert_eq!(stats.converted, 3);
        assert_eq!(data.combine_results[0].position, "S");
        assert_eq!(data.combine_results[1].position, "S");
        assert_eq!(data.combine_results[2].position, "RB");
    }

    #[test]
    fn test_convert_null_measurements() {
        let csv_data = "\
season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,,,,,,,BJ Adams,CB,Central Florida,6-2,182,4.53,,32.5,117,,";

        let rows = parse_nflverse_csv(csv_data.as_bytes()).unwrap();
        let (data, _) = convert_nflverse_to_combine_json(rows, 2025, "combine");

        let entry = &data.combine_results[0];
        assert_eq!(entry.forty_yard_dash, Some(4.53));
        assert!(entry.bench_press.is_none());
        assert_eq!(entry.vertical_jump, Some(32.5));
        assert_eq!(entry.broad_jump, Some(117));
        assert!(entry.three_cone_drill.is_none());
        assert!(entry.twenty_yard_shuttle.is_none());
        // nflverse doesn't provide these — always None
        assert!(entry.arm_length.is_none());
        assert!(entry.hand_size.is_none());
        assert!(entry.wingspan.is_none());
        assert!(entry.ten_yard_split.is_none());
        assert!(entry.twenty_yard_split.is_none());
    }

    #[test]
    fn test_convert_meta_fields() {
        let csv_data = "\
season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,,,,,,,BJ Adams,CB,Central Florida,6-2,182,4.53,,32.5,117,,";

        let rows = parse_nflverse_csv(csv_data.as_bytes()).unwrap();
        let (data, _) = convert_nflverse_to_combine_json(rows, 2025, "combine");

        assert_eq!(data.meta.source, "nflverse");
        assert_eq!(data.meta.year, 2025);
    }

    #[test]
    fn test_convert_source_field() {
        let csv_data = "\
season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,,,,,,,BJ Adams,CB,Central Florida,6-2,182,4.53,,32.5,117,,";

        let rows = parse_nflverse_csv(csv_data.as_bytes()).unwrap();
        let (data, _) = convert_nflverse_to_combine_json(rows, 2025, "pro_day");

        assert_eq!(data.combine_results[0].source, "pro_day");
    }

    #[test]
    fn test_roundtrip_json_serialization() {
        let csv_data = "\
season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,,,,,,,BJ Adams,CB,Central Florida,6-2,182,4.53,,32.5,117,,";

        let rows = parse_nflverse_csv(csv_data.as_bytes()).unwrap();
        let (data, _) = convert_nflverse_to_combine_json(rows, 2025, "combine");

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&data).unwrap();

        // Deserialize back using the existing combine_loader parser
        let parsed: CombineFileData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.combine_results.len(), 1);
        assert_eq!(parsed.combine_results[0].first_name, "BJ");
        assert_eq!(parsed.combine_results[0].last_name, "Adams");
        assert_eq!(parsed.combine_results[0].forty_yard_dash, Some(4.53));
    }
}
