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

/// Map Mockdraftable measurement type names to CombineEntry field setters.
fn set_measurement(entry: &mut CombineEntry, measurement_name: &str, value: f64) {
    match measurement_name.to_lowercase().as_str() {
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
        _ => {
            // Unknown measurement type — skip
        }
    }
}

/// Parse the INITIAL_STATE JSON into CombineData.
///
/// Mockdraftable's INITIAL_STATE typically contains a `players` array where each
/// player has `measurements` with typed measurement entries.
pub fn parse_initial_state(json: &serde_json::Value, year: i32) -> Result<CombineData> {
    let mut entries = Vec::new();

    // Navigate to the players list — structure varies, so try common paths
    let players = find_players_array(json)
        .context("Could not locate players array in INITIAL_STATE")?;

    for player in players {
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
            continue;
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

        // Extract measurements
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
                        set_measurement(&mut entry, &name, value);
                    }
                }
            }
        }

        entries.push(entry);
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

/// Recursively search for an array of player objects in the JSON.
fn find_players_array(json: &serde_json::Value) -> Option<&Vec<serde_json::Value>> {
    // Direct lookup for common keys
    for key in &["players", "results", "searchResults", "prospects"] {
        if let Some(arr) = json.get(key).and_then(|v| v.as_array()) {
            if !arr.is_empty() && looks_like_players(arr) {
                return Some(arr);
            }
        }
    }

    // Recurse into object values
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

    #[test]
    fn test_parse_initial_state_with_measurements() {
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
        assert_eq!(data.combine_results[0].position, "DE"); // DE -> DE
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
    fn test_set_measurement_all_types() {
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

        set_measurement(&mut entry, "40 Yard Dash", 4.50);
        set_measurement(&mut entry, "Bench Press", 22.0);
        set_measurement(&mut entry, "Vertical Jump", 35.0);
        set_measurement(&mut entry, "Broad Jump", 120.0);
        set_measurement(&mut entry, "3 Cone Drill", 7.00);
        set_measurement(&mut entry, "20 Yard Shuttle", 4.20);
        set_measurement(&mut entry, "Arm Length", 33.0);
        set_measurement(&mut entry, "Hand Size", 9.5);
        set_measurement(&mut entry, "Wingspan", 78.0);
        set_measurement(&mut entry, "10 Yard Split", 1.55);
        set_measurement(&mut entry, "20 Yard Split", 2.60);

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
}
