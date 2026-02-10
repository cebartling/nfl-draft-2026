use std::time::Duration;

use anyhow::{Context, Result};
use scraper::{Html, Selector};

use crate::models::{RankingData, RankingEntry, RankingMeta};

/// HTTP request timeout for scraping operations
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// Build the Tankathon big board URL for a given year
fn big_board_url(year: i32) -> String {
    if year == 2026 {
        "https://www.tankathon.com/nfl/big_board".to_string()
    } else {
        format!("https://www.tankathon.com/nfl/big_board/{}", year)
    }
}

/// Fetch the Tankathon big board HTML
pub async fn fetch_html(year: i32) -> Result<String> {
    let url = big_board_url(year);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .timeout(HTTP_TIMEOUT)
        .build()?;

    let response = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Tankathon returned status {} for {}",
            response.status(),
            url
        );
    }

    response
        .text()
        .await
        .with_context(|| "Failed to read response body")
}

/// Parse Tankathon big board HTML to extract prospect rankings.
///
/// Supports two DOM structures:
/// 1. **New SPA structure** (2025+): `div.mock-row.nfl` with child divs
///    for pick number, name, and school/position.
/// 2. **Legacy structure**: Various `div.big-board-row`, `div.prospect-row`,
///    etc. with text-based extraction.
///
/// Also attempts to extract embedded JSON data from `<script>` tags
/// (e.g. `__NEXT_DATA__`) which some SPAs inject into the initial HTML.
pub fn parse_html(html: &str, year: i32) -> Result<RankingData> {
    let document = Html::parse_document(html);

    // Strategy 1: Try the new SPA DOM structure (div.mock-row.nfl)
    let entries = parse_mock_rows(&document);
    if !entries.is_empty() {
        eprintln!(
            "Parsed {} prospects using mock-row selectors",
            entries.len()
        );
        return Ok(build_ranking_data(entries, year));
    }

    // Strategy 2: Try embedded JSON extraction (__NEXT_DATA__, __NUXT__, inline JSON)
    let entries = extract_embedded_json(html);
    if !entries.is_empty() {
        eprintln!(
            "Parsed {} prospects from embedded JSON data",
            entries.len()
        );
        return Ok(build_ranking_data(entries, year));
    }

    // Strategy 3: Legacy CSS selectors with text-based parsing
    let entries = parse_legacy_rows(&document);
    if !entries.is_empty() {
        eprintln!(
            "Parsed {} prospects using legacy selectors",
            entries.len()
        );
        return Ok(build_ranking_data(entries, year));
    }

    eprintln!("No prospect rows found with any strategy.");
    eprintln!("The site structure may have changed. Use --template or --browser to generate data.");

    Ok(build_ranking_data(vec![], year))
}

/// Parse the new SPA DOM structure: `div.mock-row.nfl`
///
/// Expected structure:
/// ```text
/// div#big-board > div.mock-rows > div.mock-row.nfl (×N)
///   ├── div.mock-row-pick-number  → "1"
///   ├── div.mock-row-player > a
///   │   ├── div.mock-row-name            → "Arvell Reese"
///   │   └── div.mock-row-school-position → "LB | Ohio State"
/// ```
fn parse_mock_rows(document: &Html) -> Vec<RankingEntry> {
    // Scope to #big-board to avoid duplicate by-school/conference sections
    let row_selector = match Selector::parse("#big-board div.mock-row.nfl") {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let pick_selector = Selector::parse("div.mock-row-pick-number").ok();
    let name_selector = Selector::parse("div.mock-row-name").ok();
    let school_pos_selector = Selector::parse("div.mock-row-school-position").ok();

    let mut entries = Vec::new();

    for (i, row) in document.select(&row_selector).enumerate() {
        let rank = pick_selector
            .as_ref()
            .and_then(|s| row.select(s).next())
            .and_then(|el| el.text().collect::<String>().trim().parse::<i32>().ok())
            .unwrap_or((i + 1) as i32);

        let full_name = name_selector
            .as_ref()
            .and_then(|s| row.select(s).next())
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let school_pos_text = school_pos_selector
            .as_ref()
            .and_then(|s| row.select(s).next())
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if full_name.is_empty() {
            continue;
        }

        let name_parts: Vec<&str> = full_name.split_whitespace().collect();
        let first_name = name_parts.first().unwrap_or(&"").to_string();
        let last_name = name_parts[1..].join(" ");

        // Parse "Position | School" format
        let sp_parts: Vec<&str> = school_pos_text.split('|').map(|s| s.trim()).collect();
        let position = sp_parts.first().unwrap_or(&"").to_uppercase();
        let school = sp_parts.get(1).unwrap_or(&"").to_string();

        // Filter out future draft class entries (rank looks like a year: 2027, 2028, etc.)
        if rank >= 2025 {
            continue;
        }

        entries.push(RankingEntry {
            rank,
            first_name,
            last_name,
            position,
            school,
        });
    }

    entries
}

/// Try to extract prospect data from embedded JSON in `<script>` tags.
///
/// Many modern SPAs (Next.js, Nuxt, etc.) embed initial state in the HTML
/// as `__NEXT_DATA__`, `__NUXT__`, or similar globals.
fn extract_embedded_json(html: &str) -> Vec<RankingEntry> {
    // Look for __NEXT_DATA__ JSON blob
    if let Some(start) = html.find("__NEXT_DATA__") {
        if let Some(json_start) = html[start..].find('{') {
            let json_str = &html[start + json_start..];
            if let Some(end) = find_matching_brace(json_str) {
                let json_slice = &json_str[..=end];
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_slice) {
                    let entries = extract_prospects_from_json(&value);
                    if !entries.is_empty() {
                        return entries;
                    }
                }
            }
        }
    }

    // Look for other common embedded data patterns
    for pattern in &["__NUXT__", "window.__data", "window.__INITIAL_STATE__"] {
        if let Some(start) = html.find(pattern) {
            if let Some(json_start) = html[start..].find('{') {
                let json_str = &html[start + json_start..];
                if let Some(end) = find_matching_brace(json_str) {
                    let json_slice = &json_str[..=end];
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_slice) {
                        let entries = extract_prospects_from_json(&value);
                        if !entries.is_empty() {
                            return entries;
                        }
                    }
                }
            }
        }
    }

    vec![]
}

/// Find the index of the matching closing brace for a JSON string starting with `{`.
fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape_next = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Recursively search a JSON value for arrays that look like prospect data.
fn extract_prospects_from_json(value: &serde_json::Value) -> Vec<RankingEntry> {
    // If this is an array, check if its elements look like prospects
    if let Some(arr) = value.as_array() {
        if arr.len() >= 20 {
            let mut entries = Vec::new();
            for (i, item) in arr.iter().enumerate() {
                if let Some(entry) = try_parse_prospect_json(item, (i + 1) as i32) {
                    entries.push(entry);
                }
            }
            // If we got at least half the array as prospects, use it
            if entries.len() >= arr.len() / 2 {
                return entries;
            }
        }
    }

    // Recurse into objects
    if let Some(obj) = value.as_object() {
        for val in obj.values() {
            let entries = extract_prospects_from_json(val);
            if !entries.is_empty() {
                return entries;
            }
        }
    }

    // Recurse into arrays
    if let Some(arr) = value.as_array() {
        for item in arr {
            let entries = extract_prospects_from_json(item);
            if !entries.is_empty() {
                return entries;
            }
        }
    }

    vec![]
}

/// Try to interpret a JSON object as a prospect entry.
fn try_parse_prospect_json(value: &serde_json::Value, fallback_rank: i32) -> Option<RankingEntry> {
    let obj = value.as_object()?;

    // Look for name fields (various conventions)
    let name = obj
        .get("name")
        .or_else(|| obj.get("player_name"))
        .or_else(|| obj.get("playerName"))
        .and_then(|v| v.as_str())?;

    let position = obj
        .get("position")
        .or_else(|| obj.get("pos"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let school = obj
        .get("school")
        .or_else(|| obj.get("college"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let rank = obj
        .get("rank")
        .or_else(|| obj.get("overall_rank"))
        .and_then(|v| v.as_i64())
        .map(|r| r as i32)
        .unwrap_or(fallback_rank);

    let name_parts: Vec<&str> = name.split_whitespace().collect();
    if name_parts.is_empty() {
        return None;
    }

    let first_name = name_parts[0].to_string();
    let last_name = name_parts[1..].join(" ");

    Some(RankingEntry {
        rank,
        first_name,
        last_name,
        position: position.to_uppercase(),
        school: school.to_string(),
    })
}

/// Legacy text-based parsing using older CSS selectors.
fn parse_legacy_rows(document: &Html) -> Vec<RankingEntry> {
    let row_selectors = [
        "div.big-board-row",
        "div.prospect-row",
        "tr.prospect-row",
        "div[class*='prospect']",
        "div[class*='board'] div[class*='row']",
    ];

    for selector_str in &row_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let elements: Vec<_> = document.select(&selector).collect();
            if !elements.is_empty() {
                eprintln!(
                    "Found {} elements with selector '{}'",
                    elements.len(),
                    selector_str
                );

                let mut entries = Vec::new();
                for (i, element) in elements.iter().enumerate() {
                    let text: String = element.text().collect::<Vec<_>>().join(" ");
                    let text = text.trim().to_string();

                    if text.is_empty() {
                        continue;
                    }

                    if let Some(entry) = parse_prospect_text(&text, (i + 1) as i32) {
                        entries.push(entry);
                    }
                }

                if !entries.is_empty() {
                    return entries;
                }
            }
        }
    }

    vec![]
}

/// Attempt to parse a prospect from a text row.
/// Expected format varies but typically: "Rank Name Position School"
fn parse_prospect_text(text: &str, fallback_rank: i32) -> Option<RankingEntry> {
    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.len() < 4 {
        return None;
    }

    // Try to extract rank from first token
    let (rank, name_start) = match parts[0].parse::<i32>() {
        Ok(r) => (r, 1),
        Err(_) => (fallback_rank, 0),
    };

    // Try to find the position (known NFL positions)
    let positions = [
        "QB", "RB", "WR", "TE", "OT", "OG", "C", "DE", "EDGE", "DT", "LB", "CB", "S", "K", "P",
        "OLB", "ILB", "NT", "FS", "SS", "T", "G",
    ];

    let mut pos_idx = None;
    for (i, part) in parts.iter().enumerate().skip(name_start) {
        if positions.contains(&part.to_uppercase().as_str()) {
            pos_idx = Some(i);
            break;
        }
    }

    let pos_idx = pos_idx?;

    // Name is between name_start and pos_idx
    if pos_idx <= name_start {
        return None;
    }

    let name_parts = &parts[name_start..pos_idx];
    let (first_name, last_name) = if name_parts.len() >= 2 {
        (name_parts[0].to_string(), name_parts[1..].join(" "))
    } else if name_parts.len() == 1 {
        (name_parts[0].to_string(), String::new())
    } else {
        return None;
    };

    let position = parts[pos_idx].to_uppercase();
    let school = if pos_idx + 1 < parts.len() {
        parts[pos_idx + 1..].join(" ")
    } else {
        String::new()
    };

    Some(RankingEntry {
        rank,
        first_name,
        last_name,
        position,
        school,
    })
}

fn build_ranking_data(entries: Vec<RankingEntry>, year: i32) -> RankingData {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    RankingData {
        meta: RankingMeta {
            version: "1.0.0".to_string(),
            source: "tankathon".to_string(),
            source_url: big_board_url(year),
            draft_year: year,
            scraped_at: today,
            total_prospects: entries.len(),
        },
        rankings: entries,
    }
}

/// Scrape Tankathon using an external Playwright script.
///
/// Shells out to `scrape-rankings.mjs` which handles JS-rendered pages.
/// Returns the parsed RankingData from the output file.
pub fn scrape_with_browser(year: i32, output: &str) -> Result<RankingData> {
    // Look for the script relative to the binary or in common locations
    let script_candidates = [
        std::path::PathBuf::from("scripts/scrape-rankings.mjs"),
        std::path::PathBuf::from("back-end/scripts/scrape-rankings.mjs"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("../scripts/scrape-rankings.mjs")))
            .unwrap_or_default(),
    ];

    let script_path = script_candidates
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Could not find scrape-rankings.mjs. Expected in: scripts/scrape-rankings.mjs"
            )
        })?;

    eprintln!("Running Playwright scraper: {}", script_path.display());

    let status = std::process::Command::new("node")
        .arg(script_path)
        .arg("--year")
        .arg(year.to_string())
        .arg("--output")
        .arg(output)
        .status()
        .with_context(|| "Failed to execute node. Is Node.js installed?")?;

    if !status.success() {
        anyhow::bail!(
            "Playwright scraper exited with status {}",
            status.code().unwrap_or(-1)
        );
    }

    let json = std::fs::read_to_string(output)
        .with_context(|| format!("Failed to read scraper output: {}", output))?;

    serde_json::from_str(&json).with_context(|| "Failed to parse scraper output JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prospect_text_standard() {
        let entry = parse_prospect_text("1 Fernando Mendoza QB Indiana", 1).unwrap();
        assert_eq!(entry.rank, 1);
        assert_eq!(entry.first_name, "Fernando");
        assert_eq!(entry.last_name, "Mendoza");
        assert_eq!(entry.position, "QB");
        assert_eq!(entry.school, "Indiana");
    }

    #[test]
    fn test_parse_prospect_text_no_rank() {
        let entry = parse_prospect_text("Caleb Downs S Ohio State", 2).unwrap();
        assert_eq!(entry.rank, 2);
        assert_eq!(entry.first_name, "Caleb");
        assert_eq!(entry.last_name, "Downs");
        assert_eq!(entry.position, "S");
        assert_eq!(entry.school, "Ohio State");
    }

    #[test]
    fn test_parse_prospect_text_edge_position() {
        let entry = parse_prospect_text("3 Mykel Williams EDGE Georgia", 3).unwrap();
        assert_eq!(entry.rank, 3);
        assert_eq!(entry.position, "EDGE");
        assert_eq!(entry.school, "Georgia");
    }

    #[test]
    fn test_parse_prospect_text_too_short() {
        assert!(parse_prospect_text("QB Indiana", 1).is_none());
    }

    #[test]
    fn test_parse_prospect_text_no_position() {
        assert!(parse_prospect_text("1 John Smith University of Something", 1).is_none());
    }

    #[test]
    fn test_parse_mock_row_html() {
        let html = r#"
        <div id="big-board">
          <div class="mock-rows">
            <div class="mock-row nfl">
              <div class="mock-row-pick-number">1</div>
              <div class="mock-row-player">
                <a class="primary-hover">
                  <div class="mock-row-name">Arvell Reese</div>
                  <div class="mock-row-school-position">LB | Ohio State</div>
                </a>
              </div>
            </div>
            <div class="mock-row nfl">
              <div class="mock-row-pick-number">2</div>
              <div class="mock-row-player">
                <a class="primary-hover">
                  <div class="mock-row-name">Rueben Bain Jr.</div>
                  <div class="mock-row-school-position">EDGE | Miami</div>
                </a>
              </div>
            </div>
          </div>
        </div>
        "#;

        let data = parse_html(html, 2026).unwrap();
        assert_eq!(data.rankings.len(), 2);
        assert_eq!(data.rankings[0].rank, 1);
        assert_eq!(data.rankings[0].first_name, "Arvell");
        assert_eq!(data.rankings[0].last_name, "Reese");
        assert_eq!(data.rankings[0].position, "LB");
        assert_eq!(data.rankings[0].school, "Ohio State");
        assert_eq!(data.rankings[1].rank, 2);
        assert_eq!(data.rankings[1].first_name, "Rueben");
        assert_eq!(data.rankings[1].last_name, "Bain Jr.");
        assert_eq!(data.rankings[1].position, "EDGE");
        assert_eq!(data.rankings[1].school, "Miami");
    }

    #[test]
    fn test_find_matching_brace() {
        assert_eq!(find_matching_brace(r#"{"a": 1}"#), Some(7));
        assert_eq!(find_matching_brace(r#"{"a": {"b": 2}}"#), Some(14));
        assert_eq!(find_matching_brace(r#"{"a": "}"}"#), Some(9));
        assert_eq!(find_matching_brace(r#"{"a": "\\}"}"#), Some(11));
        assert!(find_matching_brace(r#"{"a": 1"#).is_none());
    }

    #[test]
    fn test_extract_embedded_json_next_data() {
        let html = r#"
        <script id="__NEXT_DATA__" type="application/json">
        __NEXT_DATA__ = {"props":{"pageProps":{"prospects":[
            {"name":"Arvell Reese","position":"LB","school":"Ohio State","rank":1},
            {"name":"Rueben Bain Jr.","position":"EDGE","school":"Miami","rank":2},
            {"name":"Caleb Downs","position":"S","school":"Ohio State","rank":3},
            {"name":"Fernando Mendoza","position":"QB","school":"Indiana","rank":4},
            {"name":"David Bailey","position":"EDGE","school":"Texas Tech","rank":5},
            {"name":"Francis Mauigoa","position":"OT","school":"Miami","rank":6},
            {"name":"Carnell Tate","position":"WR","school":"Ohio State","rank":7},
            {"name":"Spencer Fano","position":"OT","school":"Utah","rank":8},
            {"name":"Jeremiyah Love","position":"RB","school":"Notre Dame","rank":9},
            {"name":"Jordyn Tyson","position":"WR","school":"Arizona State","rank":10},
            {"name":"Player 11","position":"CB","school":"Alabama","rank":11},
            {"name":"Player 12","position":"DT","school":"Georgia","rank":12},
            {"name":"Player 13","position":"S","school":"Michigan","rank":13},
            {"name":"Player 14","position":"WR","school":"USC","rank":14},
            {"name":"Player 15","position":"OT","school":"Penn State","rank":15},
            {"name":"Player 16","position":"EDGE","school":"Oregon","rank":16},
            {"name":"Player 17","position":"LB","school":"Clemson","rank":17},
            {"name":"Player 18","position":"CB","school":"Texas","rank":18},
            {"name":"Player 19","position":"QB","school":"LSU","rank":19},
            {"name":"Player 20","position":"RB","school":"Florida","rank":20}
        ]}}}
        </script>
        "#;

        let entries = extract_embedded_json(html);
        assert_eq!(entries.len(), 20);
        assert_eq!(entries[0].first_name, "Arvell");
        assert_eq!(entries[0].last_name, "Reese");
        assert_eq!(entries[0].position, "LB");
        assert_eq!(entries[0].rank, 1);
    }

    #[test]
    fn test_build_ranking_data() {
        let entries = vec![RankingEntry {
            rank: 1,
            first_name: "Test".to_string(),
            last_name: "Player".to_string(),
            position: "QB".to_string(),
            school: "Test U".to_string(),
        }];
        let data = build_ranking_data(entries, 2026);
        assert_eq!(data.meta.source, "tankathon");
        assert_eq!(data.meta.draft_year, 2026);
        assert_eq!(data.meta.total_prospects, 1);
        assert_eq!(data.rankings.len(), 1);
    }
}
