use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::models::{CombineData, CombineEntry, CombineMeta, normalize_position};

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const TIMEOUT_SECS: u64 = 30;

/// Build the Mockdraftable combine search URL for a given year.
pub fn combine_url(year: i32) -> String {
    format!(
        "https://www.mockdraftable.com/search?year={}&beginYear={}&endYear={}&sort=name",
        year, year, year
    )
}

/// Extract the `window.INITIAL_STATE` JSON blob from page HTML.
pub fn extract_initial_state(html: &str) -> Result<serde_json::Value> {
    let marker = "window.INITIAL_STATE";
    let marker_pos = html
        .find(marker)
        .context("Could not find window.INITIAL_STATE in HTML. Try using --browser flag.")?;

    // Find the opening brace after the marker
    let after_marker = &html[marker_pos..];
    let brace_start = after_marker
        .find('{')
        .context("No opening brace after INITIAL_STATE")?;

    let json_start = marker_pos + brace_start;
    let rest = &html[json_start..];

    // Brace-matched extraction
    let mut depth = 0;
    let mut end = 0;
    for (i, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if end == 0 {
        anyhow::bail!("Could not find matching closing brace for INITIAL_STATE JSON");
    }

    let json_str = &rest[..end];
    let value: serde_json::Value =
        serde_json::from_str(json_str).context("Failed to parse INITIAL_STATE JSON")?;

    Ok(value)
}

/// Map a measurable ID (from Mockdraftable's `measurables` map) to a CombineEntry field.
///
/// Mockdraftable measurable IDs and their names:
///   1=Height, 2=Weight, 3=Wingspan, 4=Arm Length, 5=Hand Size,
///   6=10 Yard Split, 7=20 Yard Split, 8=40 Yard Dash, 9=Bench Press,
///   10=Vertical Jump, 11=Broad Jump, 12=3-Cone Drill, 13=20 Yard Shuttle
fn set_measurement_by_key(entry: &mut CombineEntry, key: i64, value: f64) {
    match key {
        3 => entry.wingspan = Some(value),
        4 => entry.arm_length = Some(value),
        5 => entry.hand_size = Some(value),
        6 => entry.ten_yard_split = Some(value),
        7 => entry.twenty_yard_split = Some(value),
        8 => entry.forty_yard_dash = Some(value),
        9 => entry.bench_press = Some(value.round() as i32),
        10 => entry.vertical_jump = Some(value),
        11 => entry.broad_jump = Some(value.round() as i32),
        12 => entry.three_cone_drill = Some(value),
        13 => entry.twenty_yard_shuttle = Some(value),
        // 1=Height, 2=Weight, 14=60yd Shuttle — not in CombineEntry
        _ => {}
    }
}

/// Map Mockdraftable measurement type names to CombineEntry field setters.
/// Used when measurements have string-based type names instead of numeric keys.
fn set_measurement_by_name(entry: &mut CombineEntry, name: &str, value: f64) {
    match name.to_lowercase().as_str() {
        "40 yard dash" | "forty yard dash" | "40-yard dash" | "40yd" => {
            entry.forty_yard_dash = Some(value);
        }
        "bench press" | "bench" => {
            entry.bench_press = Some(value.round() as i32);
        }
        "vertical jump" | "vertical" | "vert" => {
            entry.vertical_jump = Some(value);
        }
        "broad jump" => {
            entry.broad_jump = Some(value.round() as i32);
        }
        "3 cone drill" | "three cone drill" | "3cone" | "3-cone drill" => {
            entry.three_cone_drill = Some(value);
        }
        "20 yard shuttle" | "twenty yard shuttle" | "shuttle" | "short shuttle" | "20yd shuttle" => {
            entry.twenty_yard_shuttle = Some(value);
        }
        "arm length" | "arms" => {
            entry.arm_length = Some(value);
        }
        "hand size" | "hands" | "hand length" => {
            entry.hand_size = Some(value);
        }
        "wingspan" | "wing span" => {
            entry.wingspan = Some(value);
        }
        "10 yard split" | "ten yard split" | "10yd" => {
            entry.ten_yard_split = Some(value);
        }
        "20 yard split" | "twenty yard split" | "20yd split" => {
            entry.twenty_yard_split = Some(value);
        }
        _ => {}
    }
}

/// Split a full name like "Cam Ward" or "Marvin Harrison Jr." into (first, last).
fn split_name(full_name: &str) -> (String, String) {
    let parts: Vec<&str> = full_name.split_whitespace().collect();
    if parts.len() <= 1 {
        return (full_name.to_string(), String::new());
    }
    let first = parts[0].to_string();
    let last = parts[1..].join(" ");
    (first, last)
}

/// Build a lookup map from measurable key IDs to their names.
/// The `measurables` object maps string keys to objects with `id` and `name` fields.
fn build_measurable_map(json: &serde_json::Value) -> HashMap<i64, String> {
    let mut map = HashMap::new();
    if let Some(measurables) = json.get("measurables").and_then(|v| v.as_object()) {
        for (_, v) in measurables {
            if let (Some(key), Some(name)) = (
                v.get("key").and_then(|k| k.as_i64()),
                v.get("name").and_then(|n| n.as_str()),
            ) {
                map.insert(key, name.to_string());
            }
        }
    }
    map
}

/// Parse the INITIAL_STATE JSON into CombineData.
///
/// Handles two formats:
/// 1. **Real Mockdraftable format**: `players` is a dict keyed by slug, each with
///    `name` (full name), `positions.primary`, and `measurements` using numeric
///    `measurableKey` IDs decoded via the top-level `measurables` map.
/// 2. **Array format**: `players` is an array with `firstName`/`lastName` fields
///    and string-based measurement type names (for backward compatibility with tests).
pub fn parse_initial_state(json: &serde_json::Value, year: i32) -> Result<CombineData> {
    let mut entries = Vec::new();
    let measurable_map = build_measurable_map(json);

    // Try dict-of-players format first (real Mockdraftable)
    if let Some(players_obj) = json.get("players").and_then(|v| v.as_object()) {
        // Check if this is actually a dict of player objects (not an array)
        let first_value = players_obj.values().next();
        let is_player_dict = first_value.is_some_and(|v| {
            v.get("name").is_some() || v.get("id").is_some()
        });

        if is_player_dict {
            for (_slug, player) in players_obj {
                if let Some(entry) = parse_dict_player(player, year, &measurable_map) {
                    entries.push(entry);
                }
            }

            let entry_count = entries.len();
            return Ok(CombineData {
                meta: CombineMeta {
                    source: "mockdraftable".to_string(),
                    description: format!("{} NFL Combine results from Mockdraftable", year),
                    year,
                    generated_at: chrono::Utc::now().to_rfc3339(),
                    player_count: entry_count,
                    entry_count,
                },
                combine_results: entries,
            });
        }
    }

    // Fall back to array-of-players format
    let players = find_players_array(json)
        .context("Could not locate players in INITIAL_STATE")?;

    for player in players {
        if let Some(entry) = parse_array_player(player, year) {
            entries.push(entry);
        }
    }

    let entry_count = entries.len();

    Ok(CombineData {
        meta: CombineMeta {
            source: "mockdraftable".to_string(),
            description: format!("{} NFL Combine results from Mockdraftable", year),
            year,
            generated_at: chrono::Utc::now().to_rfc3339(),
            player_count: entry_count,
            entry_count,
        },
        combine_results: entries,
    })
}

/// Parse a player from the dict-keyed format (real Mockdraftable structure).
fn parse_dict_player(
    player: &serde_json::Value,
    year: i32,
    measurable_map: &HashMap<i64, String>,
) -> Option<CombineEntry> {
    let full_name = player.get("name").and_then(|v| v.as_str()).unwrap_or("");
    if full_name.is_empty() {
        return None;
    }

    let (first_name, last_name) = split_name(full_name);

    // Position: positions.primary or positions.all[0]
    let position = player
        .get("positions")
        .and_then(|p| {
            p.get("primary")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    p.get("all")
                        .and_then(|a| a.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                })
        })
        .unwrap_or("");
    let position = normalize_position(position);

    let mut entry = CombineEntry {
        first_name,
        last_name,
        position,
        source: "combine".to_string(),
        year,
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
    };

    if let Some(measurements) = player.get("measurements").and_then(|v| v.as_array()) {
        for m in measurements {
            let key = m.get("measurableKey").and_then(|k| k.as_i64());
            let value = m.get("measurement").and_then(|v| v.as_f64());

            if let (Some(key), Some(value)) = (key, value) {
                // Try numeric key first, fall back to name lookup
                set_measurement_by_key(&mut entry, key, value);
                // If the key wasn't handled by numeric mapping, try name-based
                if !measurable_map.is_empty() {
                    if let Some(name) = measurable_map.get(&key) {
                        set_measurement_by_name(&mut entry, name, value);
                    }
                }
            }
        }
    }

    Some(entry)
}

/// Parse a player from the array format (firstName/lastName fields).
fn parse_array_player(player: &serde_json::Value, year: i32) -> Option<CombineEntry> {
    let first_name = player
        .get("firstName")
        .or_else(|| player.get("first_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    let last_name = player
        .get("lastName")
        .or_else(|| player.get("last_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if first_name.is_empty() && last_name.is_empty() {
        return None;
    }

    let position = player
        .get("position")
        .and_then(|v| {
            v.as_str()
                .map(String::from)
                .or_else(|| v.get("abbreviation").and_then(|a| a.as_str()).map(String::from))
                .or_else(|| v.get("name").and_then(|n| n.as_str()).map(String::from))
        })
        .unwrap_or_default();
    let position = normalize_position(&position);

    let mut entry = CombineEntry {
        first_name,
        last_name,
        position,
        source: "combine".to_string(),
        year,
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
    };

    if let Some(measurements) = player
        .get("measurements")
        .or_else(|| player.get("measurables"))
    {
        if let Some(arr) = measurements.as_array() {
            for m in arr {
                let name = m
                    .get("measurementType")
                    .or_else(|| m.get("name"))
                    .or_else(|| m.get("type"))
                    .and_then(|v| {
                        v.as_str()
                            .map(String::from)
                            .or_else(|| v.get("name").and_then(|n| n.as_str()).map(String::from))
                    });

                let value = m
                    .get("measurement")
                    .or_else(|| m.get("value"))
                    .and_then(|v| v.as_f64());

                if let (Some(name), Some(value)) = (name, value) {
                    set_measurement_by_name(&mut entry, &name, value);
                }
            }
        }
    }

    Some(entry)
}

/// Recursively search for an array of player objects in the JSON.
fn find_players_array(json: &serde_json::Value) -> Option<&Vec<serde_json::Value>> {
    for key in &["players", "results", "searchResults", "prospects"] {
        if let Some(arr) = json.get(key).and_then(|v| v.as_array()) {
            if !arr.is_empty() && looks_like_players(arr) {
                return Some(arr);
            }
        }
    }

    if let Some(obj) = json.as_object() {
        for (_, v) in obj {
            if let Some(arr) = find_players_array(v) {
                return Some(arr);
            }
        }
    }

    None
}

/// Check if an array looks like player data (has firstName/lastName fields).
fn looks_like_players(arr: &[serde_json::Value]) -> bool {
    arr.first().is_some_and(|item| {
        item.get("firstName").is_some()
            || item.get("first_name").is_some()
            || item.get("lastName").is_some()
            || item.get("last_name").is_some()
    })
}

/// Fetch and parse Mockdraftable combine data for a given year.
pub async fn scrape(year: i32) -> Result<CombineData> {
    let url = combine_url(year);
    eprintln!("Fetching Mockdraftable combine data from: {}", url);

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()?;

    let response = client.get(&url).send().await?;
    let html = response.error_for_status()?.text().await?;

    let json = extract_initial_state(&html)?;
    parse_initial_state(&json, year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_initial_state_basic() {
        let html = r#"
        <html>
        <head>
        <script>window.INITIAL_STATE = {"players": [{"firstName": "Cam", "lastName": "Ward"}]}</script>
        </head>
        <body></body>
        </html>
        "#;

        let json = extract_initial_state(html).unwrap();
        assert!(json.get("players").is_some());
        let players = json["players"].as_array().unwrap();
        assert_eq!(players.len(), 1);
        assert_eq!(players[0]["firstName"].as_str().unwrap(), "Cam");
    }

    #[test]
    fn test_extract_initial_state_minified() {
        let html = r#"<script>window.INITIAL_STATE={"data":{"players":[{"firstName":"Test","lastName":"Player"}]}}</script>"#;
        let json = extract_initial_state(html).unwrap();
        assert!(json.get("data").is_some());
    }

    #[test]
    fn test_extract_initial_state_multiple_scripts() {
        let html = r#"
        <script>var foo = "bar";</script>
        <script>window.INITIAL_STATE = {"players": [{"firstName": "Test"}]}</script>
        <script>var baz = "qux";</script>
        "#;

        let json = extract_initial_state(html).unwrap();
        assert!(json.get("players").is_some());
    }

    #[test]
    fn test_extract_initial_state_not_found() {
        let html = "<html><body>No state here</body></html>";
        assert!(extract_initial_state(html).is_err());
    }

    // Tests for the array-based format (backward compat)
    #[test]
    fn test_parse_initial_state_array_format_with_measurements() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
                "players": [
                    {
                        "firstName": "Cam",
                        "lastName": "Ward",
                        "position": "QB",
                        "measurements": [
                            {"measurementType": "40 Yard Dash", "measurement": 4.72},
                            {"measurementType": "Bench Press", "measurement": 18.0},
                            {"measurementType": "Vertical Jump", "measurement": 32.0},
                            {"measurementType": "Broad Jump", "measurement": 108.0},
                            {"measurementType": "3 Cone Drill", "measurement": 7.05},
                            {"measurementType": "20 Yard Shuttle", "measurement": 4.30},
                            {"measurementType": "Arm Length", "measurement": 32.5},
                            {"measurementType": "Hand Size", "measurement": 9.75},
                            {"measurementType": "Wingspan", "measurement": 77.5}
                        ]
                    },
                    {
                        "firstName": "Travis",
                        "lastName": "Hunter",
                        "position": "CB",
                        "measurements": [
                            {"measurementType": "40 Yard Dash", "measurement": 4.38},
                            {"measurementType": "Vertical Jump", "measurement": 40.5}
                        ]
                    }
                ]
            }"#,
        )
        .unwrap();

        let data = parse_initial_state(&json, 2026).unwrap();
        assert_eq!(data.combine_results.len(), 2);

        let cam = &data.combine_results[0];
        assert_eq!(cam.first_name, "Cam");
        assert_eq!(cam.forty_yard_dash, Some(4.72));
        assert_eq!(cam.bench_press, Some(18));
        assert_eq!(cam.vertical_jump, Some(32.0));
        assert_eq!(cam.broad_jump, Some(108));
        assert_eq!(cam.three_cone_drill, Some(7.05));
        assert_eq!(cam.twenty_yard_shuttle, Some(4.30));
        assert_eq!(cam.arm_length, Some(32.5));
        assert_eq!(cam.hand_size, Some(9.75));
        assert_eq!(cam.wingspan, Some(77.5));

        let travis = &data.combine_results[1];
        assert_eq!(travis.forty_yard_dash, Some(4.38));
        assert_eq!(travis.vertical_jump, Some(40.5));
        assert_eq!(travis.bench_press, None);
    }

    #[test]
    fn test_parse_initial_state_nested_players() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
                "data": {
                    "searchResults": {
                        "players": [
                            {
                                "firstName": "Test",
                                "lastName": "Player",
                                "position": "WR",
                                "measurements": []
                            }
                        ]
                    }
                }
            }"#,
        )
        .unwrap();

        let data = parse_initial_state(&json, 2026).unwrap();
        assert_eq!(data.combine_results.len(), 1);
        assert_eq!(data.combine_results[0].first_name, "Test");
    }

    #[test]
    fn test_parse_initial_state_position_as_object() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
                "players": [
                    {
                        "firstName": "Test",
                        "lastName": "Player",
                        "position": {"abbreviation": "DE", "name": "Defensive End"},
                        "measurements": []
                    }
                ]
            }"#,
        )
        .unwrap();

        let data = parse_initial_state(&json, 2026).unwrap();
        assert_eq!(data.combine_results[0].position, "DE");
    }

    #[test]
    fn test_combine_url() {
        assert_eq!(
            combine_url(2026),
            "https://www.mockdraftable.com/search?year=2026&beginYear=2026&endYear=2026&sort=name"
        );
    }

    #[test]
    fn test_parse_initial_state_empty_players_skipped() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
                "players": [
                    {"firstName": "", "lastName": "", "measurements": []},
                    {"firstName": "Valid", "lastName": "Player", "position": "QB", "measurements": []}
                ]
            }"#,
        )
        .unwrap();

        let data = parse_initial_state(&json, 2026).unwrap();
        assert_eq!(data.combine_results.len(), 1);
        assert_eq!(data.combine_results[0].first_name, "Valid");
    }

    #[test]
    fn test_set_measurement_by_name_all_types() {
        let mut entry = CombineEntry {
            first_name: "Test".to_string(),
            last_name: "Player".to_string(),
            position: "QB".to_string(),
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
        };

        set_measurement_by_name(&mut entry, "40 Yard Dash", 4.50);
        set_measurement_by_name(&mut entry, "Bench Press", 22.0);
        set_measurement_by_name(&mut entry, "Vertical Jump", 35.0);
        set_measurement_by_name(&mut entry, "Broad Jump", 120.0);
        set_measurement_by_name(&mut entry, "3 Cone Drill", 7.00);
        set_measurement_by_name(&mut entry, "20 Yard Shuttle", 4.20);
        set_measurement_by_name(&mut entry, "Arm Length", 33.0);
        set_measurement_by_name(&mut entry, "Hand Size", 9.5);
        set_measurement_by_name(&mut entry, "Wingspan", 78.0);
        set_measurement_by_name(&mut entry, "10 Yard Split", 1.55);
        set_measurement_by_name(&mut entry, "20 Yard Split", 2.60);

        assert_eq!(entry.forty_yard_dash, Some(4.50));
        assert_eq!(entry.bench_press, Some(22));
        assert_eq!(entry.vertical_jump, Some(35.0));
        assert_eq!(entry.broad_jump, Some(120));
        assert_eq!(entry.three_cone_drill, Some(7.00));
        assert_eq!(entry.twenty_yard_shuttle, Some(4.20));
        assert_eq!(entry.arm_length, Some(33.0));
        assert_eq!(entry.hand_size, Some(9.5));
        assert_eq!(entry.wingspan, Some(78.0));
        assert_eq!(entry.ten_yard_split, Some(1.55));
        assert_eq!(entry.twenty_yard_split, Some(2.60));
    }

    // Tests for the real Mockdraftable dict-keyed format
    #[test]
    fn test_parse_real_mockdraftable_dict_format() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
                "measurables": {
                    "1": {"id": "height", "key": 1, "name": "Height", "unit": "INCHES"},
                    "2": {"id": "weight", "key": 2, "name": "Weight", "unit": "POUNDS"},
                    "3": {"id": "wingspan", "key": 3, "name": "Wingspan", "unit": "INCHES"},
                    "4": {"id": "arms", "key": 4, "name": "Arm Length", "unit": "INCHES"},
                    "5": {"id": "hands", "key": 5, "name": "Hand Size", "unit": "INCHES"},
                    "6": {"id": "10yd", "key": 6, "name": "10 Yard Split", "unit": "SECONDS"},
                    "7": {"id": "20yd", "key": 7, "name": "20 Yard Split", "unit": "SECONDS"},
                    "8": {"id": "40yd", "key": 8, "name": "40 Yard Dash", "unit": "SECONDS"},
                    "9": {"id": "bench", "key": 9, "name": "Bench Press", "unit": "REPS"},
                    "10": {"id": "vertical", "key": 10, "name": "Vertical Jump", "unit": "INCHES"},
                    "11": {"id": "broad", "key": 11, "name": "Broad Jump", "unit": "INCHES"},
                    "12": {"id": "3cone", "key": 12, "name": "3-Cone Drill", "unit": "SECONDS"},
                    "13": {"id": "20ss", "key": 13, "name": "20 Yard Shuttle", "unit": "SECONDS"}
                },
                "players": {
                    "cam-ward": {
                        "id": "cam-ward",
                        "name": "Cam Ward",
                        "draft": 2026,
                        "key": 1234,
                        "school": "Miami (FL)",
                        "positions": {"primary": "QB", "all": ["ATH", "QB"]},
                        "measurements": [
                            {"measurableKey": 8, "measurement": 4.72, "source": 1},
                            {"measurableKey": 9, "measurement": 18.0, "source": 1},
                            {"measurableKey": 10, "measurement": 32.0, "source": 1},
                            {"measurableKey": 11, "measurement": 108.0, "source": 1},
                            {"measurableKey": 4, "measurement": 32.5, "source": 1},
                            {"measurableKey": 5, "measurement": 9.75, "source": 1},
                            {"measurableKey": 3, "measurement": 77.5, "source": 1}
                        ]
                    },
                    "aj-haulcy": {
                        "id": "aj-haulcy",
                        "name": "AJ Haulcy",
                        "draft": 2026,
                        "key": 5678,
                        "school": "LSU",
                        "positions": {"primary": "S", "all": ["ATH", "S", "DB"]},
                        "measurements": [
                            {"measurableKey": 6, "measurement": 1.62, "source": 1},
                            {"measurableKey": 8, "measurement": 4.52, "source": 1}
                        ]
                    }
                }
            }"#,
        )
        .unwrap();

        let data = parse_initial_state(&json, 2026).unwrap();
        assert_eq!(data.combine_results.len(), 2);
        assert_eq!(data.meta.source, "mockdraftable");

        // Find Cam Ward (dict order is not guaranteed)
        let cam = data
            .combine_results
            .iter()
            .find(|e| e.first_name == "Cam")
            .expect("Cam Ward should be in results");
        assert_eq!(cam.last_name, "Ward");
        assert_eq!(cam.position, "QB");
        assert_eq!(cam.forty_yard_dash, Some(4.72));
        assert_eq!(cam.bench_press, Some(18));
        assert_eq!(cam.vertical_jump, Some(32.0));
        assert_eq!(cam.broad_jump, Some(108));
        assert_eq!(cam.arm_length, Some(32.5));
        assert_eq!(cam.hand_size, Some(9.75));
        assert_eq!(cam.wingspan, Some(77.5));

        // Find AJ Haulcy
        let aj = data
            .combine_results
            .iter()
            .find(|e| e.first_name == "AJ")
            .expect("AJ Haulcy should be in results");
        assert_eq!(aj.last_name, "Haulcy");
        assert_eq!(aj.position, "S");
        assert_eq!(aj.forty_yard_dash, Some(4.52));
        assert_eq!(aj.ten_yard_split, Some(1.62));
        assert_eq!(aj.bench_press, None);
    }

    #[test]
    fn test_parse_dict_format_position_normalization() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
                "players": {
                    "test-player": {
                        "id": "test-player",
                        "name": "Test Player",
                        "positions": {"primary": "OLB", "all": ["OLB"]},
                        "measurements": []
                    }
                }
            }"#,
        )
        .unwrap();

        let data = parse_initial_state(&json, 2026).unwrap();
        assert_eq!(data.combine_results[0].position, "LB"); // OLB -> LB
    }

    #[test]
    fn test_parse_dict_format_empty_name_skipped() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
                "players": {
                    "empty": {"id": "empty", "name": "", "positions": {"primary": "QB"}, "measurements": []},
                    "valid": {"id": "valid", "name": "Valid Player", "positions": {"primary": "QB"}, "measurements": []}
                }
            }"#,
        )
        .unwrap();

        let data = parse_initial_state(&json, 2026).unwrap();
        assert_eq!(data.combine_results.len(), 1);
        assert_eq!(data.combine_results[0].first_name, "Valid");
    }

    #[test]
    fn test_set_measurement_by_key() {
        let mut entry = CombineEntry {
            first_name: "Test".to_string(),
            last_name: "Player".to_string(),
            position: "QB".to_string(),
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
        };

        set_measurement_by_key(&mut entry, 3, 78.0);    // wingspan
        set_measurement_by_key(&mut entry, 4, 33.0);    // arm length
        set_measurement_by_key(&mut entry, 5, 9.5);     // hand size
        set_measurement_by_key(&mut entry, 6, 1.55);    // 10yd split
        set_measurement_by_key(&mut entry, 7, 2.60);    // 20yd split
        set_measurement_by_key(&mut entry, 8, 4.50);    // 40yd dash
        set_measurement_by_key(&mut entry, 9, 22.0);    // bench press
        set_measurement_by_key(&mut entry, 10, 35.0);   // vertical
        set_measurement_by_key(&mut entry, 11, 120.0);  // broad jump
        set_measurement_by_key(&mut entry, 12, 7.00);   // 3-cone
        set_measurement_by_key(&mut entry, 13, 4.20);   // 20yd shuttle

        assert_eq!(entry.wingspan, Some(78.0));
        assert_eq!(entry.arm_length, Some(33.0));
        assert_eq!(entry.hand_size, Some(9.5));
        assert_eq!(entry.ten_yard_split, Some(1.55));
        assert_eq!(entry.twenty_yard_split, Some(2.60));
        assert_eq!(entry.forty_yard_dash, Some(4.50));
        assert_eq!(entry.bench_press, Some(22));
        assert_eq!(entry.vertical_jump, Some(35.0));
        assert_eq!(entry.broad_jump, Some(120));
        assert_eq!(entry.three_cone_drill, Some(7.00));
        assert_eq!(entry.twenty_yard_shuttle, Some(4.20));
    }

    #[test]
    fn test_set_measurement_by_key_ignores_height_weight() {
        let mut entry = CombineEntry {
            first_name: "Test".to_string(),
            last_name: "Player".to_string(),
            position: "QB".to_string(),
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
        };

        set_measurement_by_key(&mut entry, 1, 72.0);  // height — ignored
        set_measurement_by_key(&mut entry, 2, 215.0);  // weight — ignored
        set_measurement_by_key(&mut entry, 14, 11.5);  // 60yd shuttle — ignored

        // All should still be None
        assert!(entry.forty_yard_dash.is_none());
        assert!(entry.bench_press.is_none());
        assert!(entry.arm_length.is_none());
    }
}
