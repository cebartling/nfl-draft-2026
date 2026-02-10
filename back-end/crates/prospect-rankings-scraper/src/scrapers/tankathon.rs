use std::time::Duration;

use anyhow::{Context, Result};
use scraper::{Html, Selector};

use crate::models::{RankingData, RankingEntry, RankingMeta};

/// HTTP request timeout for scraping operations
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// Fetch the Tankathon big board HTML
pub async fn fetch_html(year: i32) -> Result<String> {
    let url = if year == 2026 {
        "https://www.tankathon.com/nfl/big_board".to_string()
    } else {
        format!("https://www.tankathon.com/nfl/big_board/{}", year)
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

/// Parse Tankathon big board HTML to extract prospect rankings.
///
/// The HTML structure uses div-based rows. This parser attempts multiple
/// CSS selector strategies and falls back gracefully.
pub fn parse_html(html: &str, year: i32) -> Result<RankingData> {
    let document = Html::parse_document(html);
    let mut entries = Vec::new();

    // Try multiple selector strategies for the big board
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
                    break;
                }
            }
        }
    }

    // If structured selectors didn't work, try text-based extraction
    if entries.is_empty() {
        eprintln!("No prospect rows found with standard selectors.");
        eprintln!(
            "The site structure may have changed. Use --template to generate a template file."
        );
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    Ok(RankingData {
        meta: RankingMeta {
            version: "1.0.0".to_string(),
            source: "tankathon".to_string(),
            source_url: "https://www.tankathon.com/nfl/big_board".to_string(),
            draft_year: year,
            scraped_at: today,
            total_prospects: entries.len(),
        },
        rankings: entries,
    })
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
}
