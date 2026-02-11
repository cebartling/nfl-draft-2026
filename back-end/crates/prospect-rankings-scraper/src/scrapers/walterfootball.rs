use std::time::Duration;

use anyhow::{Context, Result};
use scraper::{Html, Selector};

use crate::models::{RankingData, RankingEntry, RankingMeta};

/// HTTP request timeout for scraping operations
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// Fetch the Walter Football big board HTML
pub async fn fetch_html(year: i32) -> Result<String> {
    let url = format!(
        "https://walterfootball.com/nfldraftbigboard{}.php",
        year
    );

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
            "Walter Football returned status {} for {}",
            response.status(),
            url
        );
    }

    response
        .text()
        .await
        .with_context(|| "Failed to read response body")
}

/// Parse Walter Football big board HTML into RankingData.
///
/// Walter Football uses a pattern of:
///   <strong>N.</strong>
///   <strong><a href="...">Player Name</a>, Position, School.</strong>
///
/// or sometimes combined:
///   <strong>N. <a href="...">Player Name</a>, Position, School.</strong>
pub fn parse_html(html: &str, year: i32) -> Result<RankingData> {
    let document = Html::parse_document(html);
    let strong_selector = Selector::parse("strong").unwrap();

    let mut entries = Vec::new();
    let mut current_rank: Option<i32> = None;

    for element in document.select(&strong_selector) {
        let text = element.text().collect::<String>();
        let trimmed = text.trim();

        // Check if this is a rank number like "1." or "42."
        if let Some(rank) = parse_rank_number(trimmed) {
            // Check if the rest of the prospect info is in this same element
            // e.g. "1. " followed by an <a> tag child
            if let Some(entry) = try_parse_combined_element(&element, rank) {
                entries.push(entry);
                current_rank = None;
            } else {
                current_rank = Some(rank);
            }
            continue;
        }

        // If we have a pending rank, this strong element should contain the prospect info
        if let Some(rank) = current_rank {
            if let Some(entry) = try_parse_prospect_element(&element, rank) {
                entries.push(entry);
            }
            current_rank = None;
        }
    }

    // Re-number entries if we missed some ranks (fallback)
    for (i, entry) in entries.iter_mut().enumerate() {
        if entry.rank == 0 {
            entry.rank = (i + 1) as i32;
        }
    }

    let total = entries.len();

    Ok(RankingData {
        meta: RankingMeta {
            version: "1.0.0".to_string(),
            source: "Walter Football".to_string(),
            source_url: format!(
                "https://walterfootball.com/nfldraftbigboard{}.php",
                year
            ),
            draft_year: year,
            scraped_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            total_prospects: total,
        },
        rankings: entries,
    })
}

/// Parse a rank number from text like "1." or "42."
fn parse_rank_number(text: &str) -> Option<i32> {
    let text = text.trim();
    if text.ends_with('.') {
        let num_part = text.trim_end_matches('.');
        // Must be purely numeric
        if num_part.chars().all(|c| c.is_ascii_digit()) && !num_part.is_empty() {
            return num_part.parse::<i32>().ok();
        }
    }
    None
}

/// Try to parse a combined element like "<strong>1. <a>Name</a>, Pos, School.</strong>"
fn try_parse_combined_element(
    element: &scraper::ElementRef,
    rank: i32,
) -> Option<RankingEntry> {
    // Look for an <a> tag child
    let a_selector = Selector::parse("a").unwrap();
    let anchor = element.select(&a_selector).next()?;
    let name = anchor.text().collect::<String>().trim().to_string();

    if name.is_empty() {
        return None;
    }

    // Get the full inner text and find the part after the name
    let full_text = element.text().collect::<String>();

    // Find the position and school after the name
    // Pattern: "N. Name, Position, School."
    let after_name = full_text.split(&name).nth(1)?;
    parse_position_school(after_name, rank, &name)
}

/// Try to parse a prospect element like "<strong><a>Name</a>, Pos, School.</strong>"
fn try_parse_prospect_element(
    element: &scraper::ElementRef,
    rank: i32,
) -> Option<RankingEntry> {
    let a_selector = Selector::parse("a").unwrap();
    let anchor = element.select(&a_selector).next()?;
    let name = anchor.text().collect::<String>().trim().to_string();

    if name.is_empty() {
        return None;
    }

    let full_text = element.text().collect::<String>();
    let after_name = full_text.split(&name).nth(1)?;
    parse_position_school(after_name, rank, &name)
}

/// Parse ", Position, School." from text after the player name
fn parse_position_school(text: &str, rank: i32, full_name: &str) -> Option<RankingEntry> {
    // Expected format: ", Position, School." or ", Position, School"
    let text = text.trim().trim_start_matches(',').trim();

    // Split on commas to get [Position, School]
    let parts: Vec<&str> = text.splitn(2, ',').collect();
    if parts.len() < 2 {
        return None;
    }

    let raw_position = parts[0].trim().to_string();
    let school = parts[1]
        .trim()
        .trim_end_matches('.')
        .trim()
        .to_string();

    if raw_position.is_empty() || school.is_empty() {
        return None;
    }

    let position = normalize_position(&raw_position);

    let (first_name, last_name) = split_name(full_name)?;

    Some(RankingEntry {
        rank,
        first_name,
        last_name,
        position,
        school,
    })
}

/// Split a full name into first and last name
fn split_name(full_name: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = full_name.trim().splitn(2, ' ').collect();
    if parts.len() < 2 {
        return None;
    }
    Some((parts[0].to_string(), parts[1].to_string()))
}

/// Normalize Walter Football position abbreviations to our canonical set
fn normalize_position(pos: &str) -> String {
    match pos.to_uppercase().as_str() {
        // Handle slash positions (take the first one)
        s if s.contains('/') => {
            let first = s.split('/').next().unwrap_or(s);
            normalize_single_position(first).to_string()
        }
        s => normalize_single_position(s).to_string(),
    }
}

fn normalize_single_position(pos: &str) -> &str {
    match pos.trim() {
        "OLB" | "ILB" | "MLB" => "LB",
        "G" => "OG",
        "T" => "OT",
        "NT" => "DT",
        "EDGE" => "DE",
        "HB" => "RB",
        "SS" | "FS" => "S",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rank_number() {
        assert_eq!(parse_rank_number("1."), Some(1));
        assert_eq!(parse_rank_number("42."), Some(42));
        assert_eq!(parse_rank_number("100."), Some(100));
        assert_eq!(parse_rank_number("abc."), None);
        assert_eq!(parse_rank_number("1"), None);
        assert_eq!(parse_rank_number(""), None);
        assert_eq!(parse_rank_number("1.5."), None);
    }

    #[test]
    fn test_split_name() {
        assert_eq!(
            split_name("Fernando Mendoza"),
            Some(("Fernando".to_string(), "Mendoza".to_string()))
        );
        assert_eq!(
            split_name("Harold Perkins Jr"),
            Some(("Harold".to_string(), "Perkins Jr".to_string()))
        );
        assert_eq!(
            split_name("R. Mason Thomas"),
            Some(("R.".to_string(), "Mason Thomas".to_string()))
        );
        assert_eq!(split_name("OneWord"), None);
    }

    #[test]
    fn test_normalize_position() {
        assert_eq!(normalize_position("QB"), "QB");
        assert_eq!(normalize_position("OLB"), "LB");
        assert_eq!(normalize_position("G"), "OG");
        assert_eq!(normalize_position("OT/G"), "OT");
        assert_eq!(normalize_position("CB/S"), "CB");
        assert_eq!(normalize_position("EDGE"), "DE");
    }

    #[test]
    fn test_parse_position_school() {
        let result = parse_position_school(", QB, Indiana.", 1, "Fernando Mendoza");
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.rank, 1);
        assert_eq!(entry.first_name, "Fernando");
        assert_eq!(entry.last_name, "Mendoza");
        assert_eq!(entry.position, "QB");
        assert_eq!(entry.school, "Indiana");
    }

    #[test]
    fn test_parse_position_school_with_olb() {
        let result = parse_position_school(", OLB, Oklahoma.", 24, "R. Mason Thomas");
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.position, "LB");
        assert_eq!(entry.first_name, "R.");
        assert_eq!(entry.last_name, "Mason Thomas");
    }

    #[test]
    fn test_parse_html_basic() {
        let html = r#"
        <html><body>
        <strong>1.</strong>
        <strong><a href="/scout.php">Fernando Mendoza</a>, QB, Indiana.</strong>
        <strong>2.</strong>
        <strong><a href="/scout.php">Jeremiyah Love</a>, RB, Notre Dame.</strong>
        </body></html>
        "#;

        let data = parse_html(html, 2026).unwrap();
        assert_eq!(data.rankings.len(), 2);
        assert_eq!(data.rankings[0].rank, 1);
        assert_eq!(data.rankings[0].first_name, "Fernando");
        assert_eq!(data.rankings[0].last_name, "Mendoza");
        assert_eq!(data.rankings[0].position, "QB");
        assert_eq!(data.rankings[0].school, "Indiana");
        assert_eq!(data.rankings[1].rank, 2);
        assert_eq!(data.rankings[1].first_name, "Jeremiyah");
        assert_eq!(data.rankings[1].last_name, "Love");
    }

    #[test]
    fn test_parse_html_with_position_normalization() {
        let html = r#"
        <html><body>
        <strong>1.</strong>
        <strong><a href="/scout.php">Test Player</a>, OLB, Oklahoma.</strong>
        <strong>2.</strong>
        <strong><a href="/scout.php">Another Player</a>, G, Penn State.</strong>
        </body></html>
        "#;

        let data = parse_html(html, 2026).unwrap();
        assert_eq!(data.rankings[0].position, "LB");
        assert_eq!(data.rankings[1].position, "OG");
    }
}
