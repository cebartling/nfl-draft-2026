use std::time::Duration;

use anyhow::{Context, Result};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

use crate::team_name_mapper;

/// HTTP request timeout for scraping operations
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftOrderMeta {
    pub version: String,
    pub last_updated: String,
    pub sources: Vec<String>,
    /// Indicates the origin of this data: "template" or "tankathon"
    pub source: String,
    pub draft_year: i32,
    pub total_rounds: i32,
    pub total_picks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftOrderEntry {
    pub round: i32,
    pub pick_in_round: i32,
    pub overall_pick: i32,
    pub team_abbreviation: String,
    pub original_team_abbreviation: String,
    pub is_compensatory: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftOrderData {
    pub meta: DraftOrderMeta,
    pub draft_order: Vec<DraftOrderEntry>,
}

/// Fetch the Tankathon full draft page HTML
pub async fn fetch_tankathon_html(year: i32) -> Result<String> {
    let url = if year == 2026 {
        "https://www.tankathon.com/nfl/full_draft".to_string()
    } else {
        format!("https://www.tankathon.com/nfl/full_draft/{}", year)
    };

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

/// Parse a round title like "1st Round", "2nd Round", "3rd Round", "4th Round" into a number.
fn parse_round_number(text: &str) -> Option<i32> {
    let trimmed = text.trim().to_lowercase();
    // Extract leading digits from ordinal text like "1st", "2nd", "3rd", "4th"
    let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Extract the team abbreviation slug from a Tankathon SVG logo URL.
///
/// Expected URL patterns:
/// - `/img/nfl/lv.svg` → `"lv"`
/// - `https://www.tankathon.com/img/nfl/atl.svg` → `"atl"`
fn extract_abbr_from_svg_url(src: &str) -> Option<&str> {
    // Find the last path segment and strip the .svg extension
    let filename = src.rsplit('/').next()?;
    filename.strip_suffix(".svg")
}

/// Parse all `<tr>` rows within a single round container element.
///
/// Returns a vec of `(pick_number, team_abbr, original_team_abbr, is_compensatory)`.
fn parse_round_picks(round_element: &ElementRef) -> Vec<(i32, String, String, bool)> {
    let row_sel = Selector::parse("table > tbody > tr").unwrap();
    let pick_num_sel = Selector::parse("td.pick-number").unwrap();
    let team_logo_sel = Selector::parse("td div.team-link img.logo-thumb").unwrap();
    let trade_logo_sel = Selector::parse("td div.trade img.logo-thumb").unwrap();

    let mut picks = Vec::new();

    for row in round_element.select(&row_sel) {
        // Extract pick number (take only leading digits since comp picks have extra text)
        let pick_number = match row.select(&pick_num_sel).next() {
            Some(td) => {
                let text: String = td.text().collect::<Vec<_>>().join("");
                let digits: String = text
                    .trim()
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                match digits.parse::<i32>() {
                    Ok(n) => n,
                    Err(_) => continue,
                }
            }
            None => continue,
        };

        // Check if compensatory: span.primary with data-balloon containing "Compensatory"
        let is_compensatory = row
            .select(&pick_num_sel)
            .next()
            .map(|td| {
                let comp_sel = Selector::parse("span.primary[data-balloon]").unwrap();
                td.select(&comp_sel).any(|span| {
                    span.value()
                        .attr("data-balloon")
                        .map(|v| v.to_lowercase().contains("compensatory"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        // Extract team abbreviation from logo URL
        let team_abbr = match row.select(&team_logo_sel).next() {
            Some(img) => match img.value().attr("src") {
                Some(src) => match extract_abbr_from_svg_url(src) {
                    Some(slug) => team_name_mapper::normalize_svg_abbreviation(slug),
                    None => {
                        eprintln!(
                            "WARNING: Could not extract team abbreviation from logo URL: {}",
                            src
                        );
                        continue;
                    }
                },
                None => continue,
            },
            None => continue,
        };

        // Extract original team from trade div (if present), otherwise same as team
        let original_team_abbr = row
            .select(&trade_logo_sel)
            .next()
            .and_then(|img| img.value().attr("src"))
            .and_then(extract_abbr_from_svg_url)
            .map(team_name_mapper::normalize_svg_abbreviation)
            .unwrap_or_else(|| team_abbr.clone());

        picks.push((pick_number, team_abbr, original_team_abbr, is_compensatory));
    }

    picks
}

/// Parse all round containers from the Tankathon full draft page and build `DraftOrderEntry` items.
fn parse_full_draft_rounds(document: &Html) -> Vec<DraftOrderEntry> {
    let round_sel = Selector::parse("div.full-draft-round").unwrap();
    let title_sel = Selector::parse("div.round-title").unwrap();

    let mut entries = Vec::new();
    let mut overall_pick = 1;

    for round_div in document.select(&round_sel) {
        // Extract round number from title
        let round_number = match round_div.select(&title_sel).next() {
            Some(title_el) => {
                let text: String = title_el.text().collect();
                match parse_round_number(&text) {
                    Some(n) => n,
                    None => {
                        eprintln!("Could not parse round number from: {:?}", text.trim());
                        continue;
                    }
                }
            }
            None => {
                eprintln!("Round container missing round-title div");
                continue;
            }
        };

        let picks = parse_round_picks(&round_div);

        for (pick_in_round_idx, (_pick_number, team_abbr, original_team_abbr, is_comp)) in
            picks.into_iter().enumerate()
        {
            // Build notes
            let is_traded = team_abbr != original_team_abbr;
            let mut notes_parts = Vec::new();
            if is_traded {
                notes_parts.push(format!("From {}", original_team_abbr));
            }
            if is_comp {
                notes_parts.push("Compensatory pick".to_string());
            }
            let notes = if notes_parts.is_empty() {
                None
            } else {
                Some(notes_parts.join("; "))
            };

            entries.push(DraftOrderEntry {
                round: round_number,
                pick_in_round: (pick_in_round_idx + 1) as i32,
                overall_pick,
                team_abbreviation: team_abbr,
                original_team_abbreviation: original_team_abbr,
                is_compensatory: is_comp,
                notes,
            });
            overall_pick += 1;
        }
    }

    entries
}

/// Log diagnostic info about the HTML structure when parsing fails.
fn log_diagnostic_info(document: &Html) {
    eprintln!("WARNING: Could not extract draft picks from Tankathon HTML.");
    eprintln!("The site structure may have changed. Dumping diagnostic info...");

    let diagnostic_selectors = [
        ("div.full-draft-round", "round containers"),
        ("div.round-title", "round titles"),
        ("td.pick-number", "pick number cells"),
        ("img.logo-thumb", "team logos"),
        ("div.trade", "trade indicators"),
    ];

    for (sel_str, label) in &diagnostic_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            let count = document.select(&sel).count();
            eprintln!("  {}: {} found", label, count);
        }
    }

    // Dump a text sample for debugging
    if let Ok(sel) = Selector::parse("body") {
        if let Some(body) = document.select(&sel).next() {
            let text: String = body.text().collect::<Vec<_>>().join(" ");
            let sample = &text[..text.len().min(1000)];
            eprintln!("  Body text sample: {}", sample);
        }
    }

    eprintln!("Consider using the generated template and editing it manually.");
}

/// Parse Tankathon HTML to extract draft order entries.
///
/// Parses the full draft page structure:
/// - `div.full-draft-round` containers (one per round)
/// - `div.round-title` for round numbers
/// - `table > tbody > tr` rows for individual picks
/// - Team abbreviations extracted from SVG logo URLs
/// - Trade info from `div.trade` elements
/// - Compensatory picks from `span.primary[data-balloon]` attributes
///
/// Returns empty draft_order if the HTML structure doesn't match, allowing the
/// caller to fall back to `generate_template_draft_order()`.
pub fn parse_tankathon_html(html: &str, year: i32) -> Result<DraftOrderData> {
    let document = Html::parse_document(html);
    let entries = parse_full_draft_rounds(&document);

    if entries.is_empty() {
        log_diagnostic_info(&document);
    } else {
        eprintln!(
            "Successfully parsed {} picks from Tankathon HTML",
            entries.len()
        );
        // A full 7-round NFL draft has ~224 base picks + compensatory picks.
        // Warn if the count seems suspiciously low, which may indicate partial parsing.
        if entries.len() < 200 {
            eprintln!(
                "WARNING: Only {} picks parsed (expected 220+). Some rounds may not have been extracted.",
                entries.len()
            );
        }
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let max_round = entries
        .iter()
        .map(|e: &DraftOrderEntry| e.round)
        .max()
        .unwrap_or(7);

    let source = if entries.is_empty() {
        "template"
    } else {
        "tankathon"
    };

    let data = DraftOrderData {
        meta: DraftOrderMeta {
            version: "1.0.0".to_string(),
            last_updated: today,
            sources: vec!["Tankathon.com".to_string()],
            source: source.to_string(),
            draft_year: year,
            total_rounds: max_round,
            total_picks: entries.len(),
        },
        draft_order: entries,
    };

    Ok(data)
}

/// Generate a template draft order based on team season standings data.
/// This creates a realistic-looking draft order when scraping fails.
pub fn generate_template_draft_order(year: i32) -> DraftOrderData {
    // Default order based on typical 2026 mock drafts
    // This is a template — edit the JSON file to match the actual draft order
    let teams_by_round1_order = vec![
        "TEN", "CLE", "NYG", "NE", "JAX", "LV", "NYJ", "CAR", "NO", "CHI", "SF", "DAL", "MIA",
        "IND", "ATL", "ARI", "CIN", "SEA", "TB", "DEN", "PIT", "LAC", "GB", "HOU", "MIN", "LAR",
        "BAL", "WAS", "BUF", "DET", "PHI", "KC",
    ];

    let mut entries = Vec::new();
    let mut overall_pick = 1;

    for round in 1..=7 {
        for (pick_in_round, &team) in teams_by_round1_order.iter().enumerate() {
            entries.push(DraftOrderEntry {
                round,
                pick_in_round: (pick_in_round + 1) as i32,
                overall_pick,
                team_abbreviation: team.to_string(),
                original_team_abbreviation: team.to_string(),
                is_compensatory: false,
                notes: None,
            });
            overall_pick += 1;
        }

        // Add compensatory picks for rounds 3-7
        if round >= 3 {
            let comp_teams: Vec<&str> = match round {
                3 => vec!["NE", "SF", "LAR", "KC"],
                4 => vec!["DAL", "CHI", "BAL", "MIN"],
                5 => vec!["NYG", "CLE", "NO", "DET"],
                6 => vec!["JAX", "LV", "TB", "HOU"],
                7 => vec!["CAR", "CIN", "SEA", "PHI"],
                _ => vec![],
            };

            for &team in &comp_teams {
                entries.push(DraftOrderEntry {
                    round,
                    pick_in_round: (teams_by_round1_order.len()
                        + entries
                            .iter()
                            .filter(|e| e.round == round && e.is_compensatory)
                            .count()
                        + 1) as i32,
                    overall_pick,
                    team_abbreviation: team.to_string(),
                    original_team_abbreviation: team.to_string(),
                    is_compensatory: true,
                    notes: Some("Compensatory pick".to_string()),
                });
                overall_pick += 1;
            }
        }
    }

    // Fix pick_in_round for compensatory picks (recalculate sequentially per round)
    let mut current_round = 0;
    let mut pick_in_current_round = 0;
    for entry in entries.iter_mut() {
        if entry.round != current_round {
            current_round = entry.round;
            pick_in_current_round = 0;
        }
        pick_in_current_round += 1;
        entry.pick_in_round = pick_in_current_round;
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    DraftOrderData {
        meta: DraftOrderMeta {
            version: "1.0.0".to_string(),
            last_updated: today,
            sources: vec!["Template (edit manually)".to_string()],
            source: "template".to_string(),
            draft_year: year,
            total_rounds: 7,
            total_picks: entries.len(),
        },
        draft_order: entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_template_has_correct_structure() {
        let data = generate_template_draft_order(2026);

        assert_eq!(data.meta.draft_year, 2026);
        assert_eq!(data.meta.total_rounds, 7);
        assert!(data.draft_order.len() > 224); // More than 32*7 due to comp picks
        assert_eq!(data.meta.total_picks, data.draft_order.len());

        // First pick should be overall 1
        assert_eq!(data.draft_order[0].overall_pick, 1);
        assert_eq!(data.draft_order[0].round, 1);
        assert_eq!(data.draft_order[0].pick_in_round, 1);

        // Last pick overall should equal total
        let last = data.draft_order.last().unwrap();
        assert_eq!(last.overall_pick as usize, data.draft_order.len());
    }

    #[test]
    fn test_template_has_compensatory_picks() {
        let data = generate_template_draft_order(2026);

        let comp_picks: Vec<_> = data
            .draft_order
            .iter()
            .filter(|e| e.is_compensatory)
            .collect();

        assert!(!comp_picks.is_empty());

        // No comp picks in rounds 1-2
        assert!(comp_picks.iter().all(|e| e.round >= 3));
    }

    #[test]
    fn test_template_overall_picks_sequential() {
        let data = generate_template_draft_order(2026);

        for (i, entry) in data.draft_order.iter().enumerate() {
            assert_eq!(
                entry.overall_pick,
                (i + 1) as i32,
                "Overall pick mismatch at index {}: expected {}, got {}",
                i,
                i + 1,
                entry.overall_pick
            );
        }
    }

    #[test]
    fn test_template_pick_in_round_sequential() {
        let data = generate_template_draft_order(2026);

        let mut current_round = 0;
        let mut expected_pick = 0;

        for entry in &data.draft_order {
            if entry.round != current_round {
                current_round = entry.round;
                expected_pick = 1;
            } else {
                expected_pick += 1;
            }

            assert_eq!(
                entry.pick_in_round, expected_pick,
                "Pick in round mismatch for round {}, overall {}",
                entry.round, entry.overall_pick
            );
        }
    }

    #[test]
    fn test_all_team_abbreviations_valid() {
        let valid = vec![
            "ARI", "ATL", "BAL", "BUF", "CAR", "CHI", "CIN", "CLE", "DAL", "DEN", "DET", "GB",
            "HOU", "IND", "JAX", "KC", "LAC", "LAR", "LV", "MIA", "MIN", "NE", "NO", "NYG", "NYJ",
            "PHI", "PIT", "SEA", "SF", "TB", "TEN", "WAS",
        ];

        let data = generate_template_draft_order(2026);

        for entry in &data.draft_order {
            assert!(
                valid.contains(&entry.team_abbreviation.as_str()),
                "Invalid team abbreviation: {}",
                entry.team_abbreviation
            );
            assert!(
                valid.contains(&entry.original_team_abbreviation.as_str()),
                "Invalid original team abbreviation: {}",
                entry.original_team_abbreviation
            );
        }
    }

    // --- Helper function tests ---

    #[test]
    fn test_parse_round_number() {
        assert_eq!(parse_round_number("1st Round"), Some(1));
        assert_eq!(parse_round_number("2nd Round"), Some(2));
        assert_eq!(parse_round_number("3rd Round"), Some(3));
        assert_eq!(parse_round_number("4th Round"), Some(4));
        assert_eq!(parse_round_number("7th Round"), Some(7));
        assert_eq!(parse_round_number("  1st Round  "), Some(1));
        assert_eq!(parse_round_number("No number here"), None);
        assert_eq!(parse_round_number(""), None);
    }

    #[test]
    fn test_extract_abbr_from_svg_url() {
        assert_eq!(extract_abbr_from_svg_url("/img/nfl/lv.svg"), Some("lv"));
        assert_eq!(
            extract_abbr_from_svg_url("https://www.tankathon.com/img/nfl/atl.svg"),
            Some("atl")
        );
        assert_eq!(extract_abbr_from_svg_url("/img/nfl/nyj.svg"), Some("nyj"));
        assert_eq!(extract_abbr_from_svg_url("/img/nfl/logo.png"), None);
        assert_eq!(extract_abbr_from_svg_url(""), None);
    }

    // --- HTML parsing tests ---

    fn make_round_html(round_title: &str, rows: &str) -> String {
        format!(
            r#"<div class="full-draft-round full-draft-round-nfl">
                <div class="round-title">{}</div>
                <table><tbody>{}</tbody></table>
            </div>"#,
            round_title, rows
        )
    }

    fn make_pick_row(pick_num: i32, team_slug: &str) -> String {
        format!(
            r#"<tr>
                <td class="pick-number">{}</td>
                <td>
                    <div class="team-link"><a href=""><img class="logo-thumb" src="/img/nfl/{}.svg"></a></div>
                </td>
            </tr>"#,
            pick_num, team_slug
        )
    }

    fn make_traded_pick_row(pick_num: i32, team_slug: &str, original_slug: &str) -> String {
        format!(
            r#"<tr>
                <td class="pick-number">{}</td>
                <td>
                    <div class="team-link"><a href=""><img class="logo-thumb" src="/img/nfl/{}.svg"></a></div>
                    <div class="trade"><a href=""><img class="logo-thumb" src="/img/nfl/{}.svg"></a></div>
                </td>
            </tr>"#,
            pick_num, team_slug, original_slug
        )
    }

    fn make_comp_pick_row(pick_num: i32, team_slug: &str) -> String {
        format!(
            r#"<tr>
                <td class="pick-number">{}
                    <span class="primary" data-balloon="Compensatory pick">C</span>
                </td>
                <td>
                    <div class="team-link"><a href=""><img class="logo-thumb" src="/img/nfl/{}.svg"></a></div>
                </td>
            </tr>"#,
            pick_num, team_slug
        )
    }

    #[test]
    fn test_parse_basic_round() {
        let rows = [
            make_pick_row(1, "ten"),
            make_pick_row(2, "cle"),
            make_pick_row(3, "nyg"),
        ]
        .join("\n");
        let html = format!(
            "<html><body>{}</body></html>",
            make_round_html("1st Round", &rows)
        );

        let data = parse_tankathon_html(&html, 2026).unwrap();
        assert_eq!(data.draft_order.len(), 3);
        assert_eq!(data.meta.source, "tankathon");
        assert_eq!(data.meta.total_picks, 3);

        assert_eq!(data.draft_order[0].round, 1);
        assert_eq!(data.draft_order[0].pick_in_round, 1);
        assert_eq!(data.draft_order[0].overall_pick, 1);
        assert_eq!(data.draft_order[0].team_abbreviation, "TEN");
        assert_eq!(data.draft_order[0].original_team_abbreviation, "TEN");
        assert!(!data.draft_order[0].is_compensatory);
        assert!(data.draft_order[0].notes.is_none());

        assert_eq!(data.draft_order[2].team_abbreviation, "NYG");
        assert_eq!(data.draft_order[2].overall_pick, 3);
    }

    #[test]
    fn test_parse_traded_pick() {
        let rows = make_traded_pick_row(5, "lv", "atl");
        let html = format!(
            "<html><body>{}</body></html>",
            make_round_html("1st Round", &rows)
        );

        let data = parse_tankathon_html(&html, 2026).unwrap();
        assert_eq!(data.draft_order.len(), 1);

        let entry = &data.draft_order[0];
        assert_eq!(entry.team_abbreviation, "LV");
        assert_eq!(entry.original_team_abbreviation, "ATL");
        assert!(!entry.is_compensatory);
        assert_eq!(entry.notes.as_deref(), Some("From ATL"));
    }

    #[test]
    fn test_parse_compensatory_pick() {
        let rows = make_comp_pick_row(33, "ne");
        let html = format!(
            "<html><body>{}</body></html>",
            make_round_html("3rd Round", &rows)
        );

        let data = parse_tankathon_html(&html, 2026).unwrap();
        assert_eq!(data.draft_order.len(), 1);

        let entry = &data.draft_order[0];
        assert_eq!(entry.team_abbreviation, "NE");
        assert_eq!(entry.original_team_abbreviation, "NE");
        assert!(entry.is_compensatory);
        assert_eq!(entry.notes.as_deref(), Some("Compensatory pick"));
    }

    #[test]
    fn test_parse_traded_compensatory_pick() {
        let rows = format!(
            r#"<tr>
                <td class="pick-number">35
                    <span class="primary" data-balloon="Compensatory pick">C</span>
                </td>
                <td>
                    <div class="team-link"><a href=""><img class="logo-thumb" src="/img/nfl/dal.svg"></a></div>
                    <div class="trade"><a href=""><img class="logo-thumb" src="/img/nfl/sf.svg"></a></div>
                </td>
            </tr>"#
        );
        let html = format!(
            "<html><body>{}</body></html>",
            make_round_html("3rd Round", &rows)
        );

        let data = parse_tankathon_html(&html, 2026).unwrap();
        assert_eq!(data.draft_order.len(), 1);

        let entry = &data.draft_order[0];
        assert_eq!(entry.team_abbreviation, "DAL");
        assert_eq!(entry.original_team_abbreviation, "SF");
        assert!(entry.is_compensatory);
        assert_eq!(entry.notes.as_deref(), Some("From SF; Compensatory pick"));
    }

    #[test]
    fn test_parse_multiple_rounds() {
        let round1_rows = [make_pick_row(1, "ten"), make_pick_row(2, "cle")].join("\n");
        // Use realistic overall pick numbers (33, 34) like Tankathon does
        let round2_rows = [make_pick_row(33, "cle"), make_pick_row(34, "ten")].join("\n");
        let html = format!(
            "<html><body>{}{}</body></html>",
            make_round_html("1st Round", &round1_rows),
            make_round_html("2nd Round", &round2_rows)
        );

        let data = parse_tankathon_html(&html, 2026).unwrap();
        assert_eq!(data.draft_order.len(), 4);
        assert_eq!(data.meta.total_rounds, 2);

        // Round 1
        assert_eq!(data.draft_order[0].round, 1);
        assert_eq!(data.draft_order[0].overall_pick, 1);
        assert_eq!(data.draft_order[0].pick_in_round, 1);
        assert_eq!(data.draft_order[1].round, 1);
        assert_eq!(data.draft_order[1].overall_pick, 2);
        assert_eq!(data.draft_order[1].pick_in_round, 2);

        // Round 2: pick_in_round must be 1-based per round, NOT overall numbers
        assert_eq!(data.draft_order[2].round, 2);
        assert_eq!(data.draft_order[2].overall_pick, 3);
        assert_eq!(data.draft_order[2].pick_in_round, 1);
        assert_eq!(data.draft_order[3].round, 2);
        assert_eq!(data.draft_order[3].overall_pick, 4);
        assert_eq!(data.draft_order[3].pick_in_round, 2);
    }

    #[test]
    fn test_parse_empty_html() {
        let html = "<html><body><p>Nothing here</p></body></html>";
        let data = parse_tankathon_html(html, 2026).unwrap();
        assert!(data.draft_order.is_empty());
        assert_eq!(data.meta.source, "template");
        assert_eq!(data.meta.total_picks, 0);
    }

    #[test]
    fn test_parse_wsh_normalization() {
        let rows = make_pick_row(28, "wsh");
        let html = format!(
            "<html><body>{}</body></html>",
            make_round_html("1st Round", &rows)
        );

        let data = parse_tankathon_html(&html, 2026).unwrap();
        assert_eq!(data.draft_order[0].team_abbreviation, "WAS");
    }

    #[test]
    fn test_parse_jac_normalization() {
        let rows = make_pick_row(5, "jac");
        let html = format!(
            "<html><body>{}</body></html>",
            make_round_html("1st Round", &rows)
        );

        let data = parse_tankathon_html(&html, 2026).unwrap();
        assert_eq!(data.draft_order[0].team_abbreviation, "JAX");
    }
}
