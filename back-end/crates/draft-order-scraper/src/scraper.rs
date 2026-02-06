use anyhow::{Context, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

// Used when parsing actual Tankathon HTML
#[allow(unused_imports)]
use crate::team_name_mapper;

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftOrderMeta {
    pub version: String,
    pub last_updated: String,
    pub sources: Vec<String>,
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

/// Parse Tankathon HTML to extract draft order entries.
///
/// Tankathon uses a table-based layout with rows containing pick info.
/// The structure varies but generally has pick number, team name, and trade info.
pub fn parse_tankathon_html(html: &str, year: i32) -> Result<DraftOrderData> {
    let document = Html::parse_document(html);
    let entries = Vec::new();

    // Tankathon uses div-based pick rows. Try multiple selector strategies.
    let pick_selectors = [
        "div.pick-row",
        "tr.pick-row",
        "div[class*='pick']",
        "table.draft-board tr",
    ];

    let mut found_picks = false;

    for selector_str in &pick_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let elements: Vec<_> = document.select(&selector).collect();
            if !elements.is_empty() {
                eprintln!(
                    "Found {} elements with selector '{}'",
                    elements.len(),
                    selector_str
                );
                found_picks = true;

                for element in &elements {
                    let text: String = element.text().collect::<Vec<_>>().join(" ");
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        eprintln!("  Row text: {}", &text[..text.len().min(120)]);
                    }
                }
                break;
            }
        }
    }

    if !found_picks {
        // Fallback: try to extract any useful data from the page
        // Look for numbered pick patterns in the text
        eprintln!(
            "No pick rows found with standard selectors. Attempting text-based extraction..."
        );

        // Try to find all links or spans that look like team names
        if let Ok(selector) = Selector::parse("a, span, td, div") {
            let all_text: String = document
                .select(&selector)
                .flat_map(|el| el.text())
                .collect::<Vec<_>>()
                .join("\n");

            // Log a sample for debugging
            let sample = &all_text[..all_text.len().min(2000)];
            eprintln!("Page text sample:\n{}", sample);
        }
    }

    // If scraping didn't produce results, return empty data with a note
    if entries.is_empty() {
        eprintln!("WARNING: Could not extract draft picks from Tankathon HTML.");
        eprintln!("The site structure may have changed. The output file will need manual editing.");
        eprintln!("Consider using the generated template and editing it manually.");
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let max_round = entries
        .iter()
        .map(|e: &DraftOrderEntry| e.round)
        .max()
        .unwrap_or(7);

    let data = DraftOrderData {
        meta: DraftOrderMeta {
            version: "1.0.0".to_string(),
            last_updated: today,
            sources: vec!["Tankathon.com".to_string()],
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
}
