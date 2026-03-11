use anyhow::{Context, Result};
use scraper::{Html, Selector};

use crate::models::{CombineData, CombineEntry, CombineMeta, normalize_position};

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const TIMEOUT_SECS: u64 = 30;

/// Build the Pro Football Reference combine URL for a given year.
pub fn combine_url(year: i32) -> String {
    format!(
        "https://www.pro-football-reference.com/draft/{}-combine.htm",
        year
    )
}

/// Parse a PFR height string like "6-2" into total inches (74).
fn parse_height(s: &str) -> Option<i32> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() == 2 {
        let feet: i32 = parts[0].trim().parse().ok()?;
        let inches: i32 = parts[1].trim().parse().ok()?;
        Some(feet * 12 + inches)
    } else {
        None
    }
}

/// Parse a time value (e.g., "4.48") into f64, returning None for empty/dash.
fn parse_time(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return None;
    }
    trimmed.parse::<f64>().ok()
}

/// Parse an integer measurement (e.g., "25" for bench press reps).
fn parse_int_measurement(s: &str) -> Option<i32> {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return None;
    }
    trimmed.parse::<i32>().ok()
}

/// Split a full name into (first, last), handling suffixes like Jr., III.
fn split_name(full_name: &str) -> (String, String) {
    let parts: Vec<&str> = full_name.split_whitespace().collect();
    if parts.len() <= 1 {
        return (full_name.to_string(), String::new());
    }

    let first = parts[0].to_string();
    let last = parts[1..].join(" ");
    (first, last)
}

/// Parse PFR combine HTML into CombineData.
///
/// PFR uses `<table id="combine">` with columns:
/// Player, Pos, School, Ht, Wt, 40yd, Vertical, Bench, Broad Jump, 3Cone, Shuttle
pub fn parse_html(html: &str, year: i32) -> Result<CombineData> {
    let document = Html::parse_document(html);

    let table_selector =
        Selector::parse("table#combine").expect("valid selector");
    let row_selector = Selector::parse("tbody tr:not(.thead)").expect("valid selector");
    let td_selector = Selector::parse("td").expect("valid selector");
    let th_selector = Selector::parse("th").expect("valid selector");

    let table = document
        .select(&table_selector)
        .next()
        .context("Could not find table#combine in PFR HTML")?;

    let mut entries = Vec::new();

    for row in table.select(&row_selector) {
        // Skip over-header rows (class="over_header") and spacer rows
        let class = row.value().attr("class").unwrap_or("");
        if class.contains("over_header") || class.contains("spacer") {
            continue;
        }

        // Player name is in the <th> element with data-stat="player"
        let player_name = row
            .select(&th_selector)
            .find(|th| th.value().attr("data-stat") == Some("player"))
            .map(|th| th.text().collect::<String>())
            .unwrap_or_default();

        let player_name = player_name.trim();
        if player_name.is_empty() {
            continue;
        }

        let (first_name, last_name) = split_name(player_name);

        // Collect td cells into a map by data-stat attribute
        let cells: std::collections::HashMap<&str, String> = row
            .select(&td_selector)
            .filter_map(|td| {
                let stat = td.value().attr("data-stat")?;
                let text = td.text().collect::<String>();
                Some((stat, text.trim().to_string()))
            })
            .collect();

        let position = cells.get("pos").map(|s| s.as_str()).unwrap_or("");
        let position = normalize_position(position);

        let _height = cells
            .get("height")
            .and_then(|s| parse_height(s));
        let _weight = cells
            .get("weight")
            .and_then(|s| parse_int_measurement(s));

        let entry = CombineEntry {
            first_name,
            last_name,
            position,
            source: "combine".to_string(),
            year,
            forty_yard_dash: cells.get("forty_yd").and_then(|s| parse_time(s)),
            bench_press: cells.get("bench_reps").and_then(|s| parse_int_measurement(s)),
            vertical_jump: cells.get("vertical").and_then(|s| parse_time(s)),
            broad_jump: cells.get("broad_jump").and_then(|s| parse_int_measurement(s)),
            three_cone_drill: cells.get("cone").and_then(|s| parse_time(s)),
            twenty_yard_shuttle: cells.get("shuttle").and_then(|s| parse_time(s)),
            arm_length: cells.get("arm_length").and_then(|s| parse_time(s)),
            hand_size: cells.get("hand_size").and_then(|s| parse_time(s)),
            wingspan: cells.get("wingspan").and_then(|s| parse_time(s)),
            ten_yard_split: cells.get("ten_yd").and_then(|s| parse_time(s)),
            twenty_yard_split: cells.get("twenty_yd").and_then(|s| parse_time(s)),
        };

        entries.push(entry);
    }

    let entry_count = entries.len();

    Ok(CombineData {
        meta: CombineMeta {
            source: "pro_football_reference".to_string(),
            description: format!("{} NFL Combine results from Pro Football Reference", year),
            year,
            generated_at: chrono::Utc::now().to_rfc3339(),
            player_count: entry_count,
            entry_count,
        },
        combine_results: entries,
    })
}

/// Fetch and parse PFR combine data for a given year.
pub async fn scrape(year: i32) -> Result<CombineData> {
    let url = combine_url(year);
    eprintln!("Fetching PFR combine data from: {}", url);

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()?;

    let response = client.get(&url).send().await?;

    let status = response.status();
    if status == reqwest::StatusCode::FORBIDDEN {
        anyhow::bail!(
            "PFR returned 403 Forbidden. Try using --browser flag for Playwright-based scraping."
        );
    }

    let html = response
        .error_for_status()?
        .text()
        .await?;

    parse_html(&html, year)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pfr_html() -> &'static str {
        r#"
        <html>
        <body>
        <table id="combine" class="sortable stats_table">
        <thead>
            <tr><th>Player</th><th>Pos</th><th>School</th><th>Ht</th><th>Wt</th><th>40yd</th><th>Vertical</th><th>Bench</th><th>Broad Jump</th><th>3Cone</th><th>Shuttle</th></tr>
        </thead>
        <tbody>
            <tr>
                <th data-stat="player">Cam Ward</th>
                <td data-stat="pos">QB</td>
                <td data-stat="school_name">Miami (FL)</td>
                <td data-stat="height">6-2</td>
                <td data-stat="weight">210</td>
                <td data-stat="forty_yd">4.72</td>
                <td data-stat="vertical">32.0</td>
                <td data-stat="bench_reps">18</td>
                <td data-stat="broad_jump">108</td>
                <td data-stat="cone">7.05</td>
                <td data-stat="shuttle">4.30</td>
                <td data-stat="arm_length">32.5</td>
                <td data-stat="hand_size">9.75</td>
                <td data-stat="wingspan">77.5</td>
                <td data-stat="ten_yd">1.65</td>
                <td data-stat="twenty_yd">2.72</td>
            </tr>
            <tr>
                <th data-stat="player">Travis Hunter</th>
                <td data-stat="pos">CB</td>
                <td data-stat="school_name">Colorado</td>
                <td data-stat="height">6-1</td>
                <td data-stat="weight">185</td>
                <td data-stat="forty_yd">4.38</td>
                <td data-stat="vertical">40.5</td>
                <td data-stat="bench_reps">-</td>
                <td data-stat="broad_jump">130</td>
                <td data-stat="cone"></td>
                <td data-stat="shuttle">4.05</td>
                <td data-stat="arm_length"></td>
                <td data-stat="hand_size"></td>
                <td data-stat="wingspan"></td>
                <td data-stat="ten_yd">1.50</td>
                <td data-stat="twenty_yd"></td>
            </tr>
            <tr>
                <th data-stat="player">Shedeur Sanders</th>
                <td data-stat="pos">QB</td>
                <td data-stat="school_name">Colorado</td>
                <td data-stat="height">6-2</td>
                <td data-stat="weight">215</td>
                <td data-stat="forty_yd">4.79</td>
                <td data-stat="vertical">31.5</td>
                <td data-stat="bench_reps">15</td>
                <td data-stat="broad_jump">105</td>
                <td data-stat="cone">7.12</td>
                <td data-stat="shuttle">4.35</td>
                <td data-stat="arm_length">33.0</td>
                <td data-stat="hand_size">9.50</td>
                <td data-stat="wingspan">78.0</td>
                <td data-stat="ten_yd">1.67</td>
                <td data-stat="twenty_yd">2.76</td>
            </tr>
        </tbody>
        </table>
        </body>
        </html>
        "#
    }

    #[test]
    fn test_parse_html_basic() {
        let data = parse_html(sample_pfr_html(), 2026).unwrap();
        assert_eq!(data.combine_results.len(), 3);
        assert_eq!(data.meta.source, "pro_football_reference");
        assert_eq!(data.meta.year, 2026);
    }

    #[test]
    fn test_parse_html_player_names() {
        let data = parse_html(sample_pfr_html(), 2026).unwrap();
        assert_eq!(data.combine_results[0].first_name, "Cam");
        assert_eq!(data.combine_results[0].last_name, "Ward");
        assert_eq!(data.combine_results[1].first_name, "Travis");
        assert_eq!(data.combine_results[1].last_name, "Hunter");
    }

    #[test]
    fn test_parse_html_measurables() {
        let data = parse_html(sample_pfr_html(), 2026).unwrap();
        let cam = &data.combine_results[0];
        assert_eq!(cam.forty_yard_dash, Some(4.72));
        assert_eq!(cam.bench_press, Some(18));
        assert_eq!(cam.vertical_jump, Some(32.0));
        assert_eq!(cam.broad_jump, Some(108));
        assert_eq!(cam.three_cone_drill, Some(7.05));
        assert_eq!(cam.twenty_yard_shuttle, Some(4.30));
    }

    #[test]
    fn test_parse_html_missing_values() {
        let data = parse_html(sample_pfr_html(), 2026).unwrap();
        let travis = &data.combine_results[1];
        // Dash and empty should both be None
        assert_eq!(travis.bench_press, None);
        assert_eq!(travis.three_cone_drill, None);
        assert_eq!(travis.arm_length, None);
        assert_eq!(travis.twenty_yard_split, None);
        // But present values should still be there
        assert_eq!(travis.forty_yard_dash, Some(4.38));
        assert_eq!(travis.twenty_yard_shuttle, Some(4.05));
    }

    #[test]
    fn test_parse_html_position_normalization() {
        let html = r#"
        <html><body>
        <table id="combine"><tbody>
            <tr>
                <th data-stat="player">John Smith</th>
                <td data-stat="pos">DE</td>
                <td data-stat="forty_yd">4.65</td>
            </tr>
            <tr>
                <th data-stat="player">Jim Jones</th>
                <td data-stat="pos">ILB</td>
                <td data-stat="forty_yd">4.55</td>
            </tr>
            <tr>
                <th data-stat="player">Bob Brown</th>
                <td data-stat="pos">FS</td>
                <td data-stat="forty_yd">4.45</td>
            </tr>
            <tr>
                <th data-stat="player">Mike Wilson</th>
                <td data-stat="pos">NT</td>
                <td data-stat="forty_yd">5.10</td>
            </tr>
            <tr>
                <th data-stat="player">Tom Davis</th>
                <td data-stat="pos">OG</td>
                <td data-stat="forty_yd">5.20</td>
            </tr>
        </tbody></table>
        </body></html>
        "#;

        let data = parse_html(html, 2026).unwrap();
        assert_eq!(data.combine_results[0].position, "EDGE"); // DE -> EDGE
        assert_eq!(data.combine_results[1].position, "LB"); // ILB -> LB
        assert_eq!(data.combine_results[2].position, "S"); // FS -> S
        assert_eq!(data.combine_results[3].position, "DT"); // NT -> DT
        assert_eq!(data.combine_results[4].position, "IOL"); // OG -> IOL
    }

    #[test]
    fn test_parse_html_name_with_suffix() {
        let html = r#"
        <html><body>
        <table id="combine"><tbody>
            <tr>
                <th data-stat="player">Marvin Harrison Jr.</th>
                <td data-stat="pos">WR</td>
                <td data-stat="forty_yd">4.40</td>
            </tr>
            <tr>
                <th data-stat="player">Will Lee III</th>
                <td data-stat="pos">CB</td>
                <td data-stat="forty_yd">4.35</td>
            </tr>
        </tbody></table>
        </body></html>
        "#;

        let data = parse_html(html, 2026).unwrap();
        assert_eq!(data.combine_results[0].first_name, "Marvin");
        assert_eq!(data.combine_results[0].last_name, "Harrison Jr.");
        assert_eq!(data.combine_results[1].first_name, "Will");
        assert_eq!(data.combine_results[1].last_name, "Lee III");
    }

    #[test]
    fn test_parse_height() {
        assert_eq!(parse_height("6-2"), Some(74));
        assert_eq!(parse_height("5-11"), Some(71));
        assert_eq!(parse_height("6-0"), Some(72));
        assert_eq!(parse_height(""), None);
        assert_eq!(parse_height("-"), None);
    }

    #[test]
    fn test_parse_time_values() {
        assert_eq!(parse_time("4.48"), Some(4.48));
        assert_eq!(parse_time(""), None);
        assert_eq!(parse_time("-"), None);
        assert_eq!(parse_time("  4.72  "), Some(4.72));
    }

    #[test]
    fn test_combine_url() {
        assert_eq!(
            combine_url(2026),
            "https://www.pro-football-reference.com/draft/2026-combine.htm"
        );
        assert_eq!(
            combine_url(2025),
            "https://www.pro-football-reference.com/draft/2025-combine.htm"
        );
    }

    #[test]
    fn test_parse_html_skips_spacer_rows() {
        let html = r#"
        <html><body>
        <table id="combine"><tbody>
            <tr class="over_header">
                <th>Section</th>
            </tr>
            <tr>
                <th data-stat="player">Cam Ward</th>
                <td data-stat="pos">QB</td>
                <td data-stat="forty_yd">4.72</td>
            </tr>
            <tr class="spacer">
                <td colspan="20"></td>
            </tr>
            <tr class="thead">
                <th>Player</th>
            </tr>
            <tr>
                <th data-stat="player">Travis Hunter</th>
                <td data-stat="pos">CB</td>
                <td data-stat="forty_yd">4.38</td>
            </tr>
        </tbody></table>
        </body></html>
        "#;

        let data = parse_html(html, 2026).unwrap();
        assert_eq!(data.combine_results.len(), 2);
    }
}
