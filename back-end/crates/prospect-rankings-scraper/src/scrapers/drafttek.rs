use std::time::Duration;

use anyhow::{Context, Result};
use scraper::{Html, Selector};

use crate::models::{RankingData, RankingEntry, RankingMeta};

/// HTTP request timeout for scraping operations
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// DraftTek paginates at 100 prospects per page
const PROSPECTS_PER_PAGE: usize = 100;

/// Fetch a single page of DraftTek big board HTML
async fn fetch_page(year: i32, page: usize) -> Result<String> {
    let url = if page == 1 {
        format!(
            "https://www.drafttek.com/{}-NFL-Draft-Big-Board/Top-NFL-Draft-Prospects-{}.asp",
            year, year
        )
    } else {
        format!(
            "https://www.drafttek.com/{}-NFL-Draft-Big-Board/Top-NFL-Draft-Prospects-{}-Page-{}.asp",
            year, year, page
        )
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
        anyhow::bail!("DraftTek returned status {} for {}", response.status(), url);
    }

    response
        .text()
        .await
        .with_context(|| "Failed to read response body")
}

/// Fetch and parse DraftTek big board, up to max_prospects.
pub async fn fetch_rankings(year: i32, max_prospects: usize) -> Result<RankingData> {
    let max_pages = max_prospects.div_ceil(PROSPECTS_PER_PAGE);
    let mut all_entries = Vec::new();

    for page in 1..=max_pages {
        eprintln!("Fetching DraftTek page {}...", page);

        match fetch_page(year, page).await {
            Ok(html) => {
                let page_entries = parse_page_html(&html, all_entries.len());
                eprintln!("  Found {} prospects on page {}", page_entries.len(), page);

                if page_entries.is_empty() {
                    eprintln!("  No prospects found on page {}, stopping.", page);
                    break;
                }

                all_entries.extend(page_entries);

                if all_entries.len() >= max_prospects {
                    all_entries.truncate(max_prospects);
                    break;
                }

                // Be polite between requests
                if page < max_pages {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
            Err(e) => {
                eprintln!("  Failed to fetch page {}: {}", page, e);
                if page == 1 {
                    return Err(e);
                }
                // For subsequent pages, stop but keep what we have
                break;
            }
        }
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    Ok(RankingData {
        meta: RankingMeta {
            version: "1.0.0".to_string(),
            source: "drafttek".to_string(),
            source_url: format!(
                "https://www.drafttek.com/{}-NFL-Draft-Big-Board/Top-NFL-Draft-Prospects-{}.asp",
                year, year
            ),
            draft_year: year,
            scraped_at: today,
            total_prospects: all_entries.len(),
        },
        rankings: all_entries,
    })
}

/// Parse a single DraftTek HTML page.
fn parse_page_html(html: &str, rank_offset: usize) -> Vec<RankingEntry> {
    let document = Html::parse_document(html);
    let mut entries = Vec::new();

    // DraftTek uses table rows for prospects
    let table_selectors = [
        "table.bpa tr",
        "table tr.pointed",
        "table[class*='draft'] tr",
        "table tr",
    ];

    for selector_str in &table_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let rows: Vec<_> = document.select(&selector).collect();

            if rows.len() < 2 {
                continue;
            }

            // Parse cell selector once
            let td_selector =
                Selector::parse("td").unwrap_or_else(|_| Selector::parse("*").unwrap());

            for row in &rows {
                let cells: Vec<String> = row
                    .select(&td_selector)
                    .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .collect();

                if let Some(entry) = parse_drafttek_row(&cells, rank_offset + entries.len()) {
                    entries.push(entry);
                }
            }

            if !entries.is_empty() {
                break;
            }
        }
    }

    if entries.is_empty() {
        eprintln!("Could not extract prospects from DraftTek HTML.");
        eprintln!("The site structure may have changed.");
    }

    entries
}

/// Parse a DraftTek table row into a RankingEntry.
/// DraftTek columns typically: Rank | Player Name | College | Position | Height | Weight | Class
fn parse_drafttek_row(cells: &[String], fallback_rank: usize) -> Option<RankingEntry> {
    if cells.len() < 4 {
        return None;
    }

    // Try rank from first column
    let rank = cells[0]
        .trim()
        .replace('.', "")
        .parse::<i32>()
        .unwrap_or((fallback_rank + 1) as i32);

    if rank < 1 {
        return None;
    }

    // Player name (second column typically)
    let name = cells[1].trim();
    if name.is_empty() {
        return None;
    }

    let name_parts: Vec<&str> = name.split_whitespace().collect();
    if name_parts.is_empty() {
        return None;
    }

    let (first_name, last_name) = if name_parts.len() >= 2 {
        (name_parts[0].to_string(), name_parts[1..].join(" "))
    } else {
        (name_parts[0].to_string(), String::new())
    };

    // College (third column)
    let school = cells[2].trim().to_string();

    // Position (fourth column)
    let position = cells[3].trim().to_uppercase();
    if position.is_empty() {
        return None;
    }

    // Filter out header rows
    let known_positions = [
        "QB", "RB", "WR", "TE", "OT", "OG", "C", "DE", "EDGE", "DT", "LB", "CB", "S", "K", "P",
        "OLB", "ILB", "NT", "FS", "SS", "T", "G", "HB",
    ];
    if !known_positions.contains(&position.as_str()) {
        return None;
    }

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
    fn test_parse_drafttek_row_standard() {
        let cells = vec![
            "1".to_string(),
            "Fernando Mendoza".to_string(),
            "Indiana".to_string(),
            "QB".to_string(),
            "6-3".to_string(),
            "215".to_string(),
        ];
        let entry = parse_drafttek_row(&cells, 0).unwrap();
        assert_eq!(entry.rank, 1);
        assert_eq!(entry.first_name, "Fernando");
        assert_eq!(entry.last_name, "Mendoza");
        assert_eq!(entry.position, "QB");
        assert_eq!(entry.school, "Indiana");
    }

    #[test]
    fn test_parse_drafttek_row_with_dot_rank() {
        let cells = vec![
            "5.".to_string(),
            "Travis Hunter".to_string(),
            "Colorado".to_string(),
            "CB".to_string(),
        ];
        let entry = parse_drafttek_row(&cells, 4).unwrap();
        assert_eq!(entry.rank, 5);
        assert_eq!(entry.position, "CB");
    }

    #[test]
    fn test_parse_drafttek_row_too_few_columns() {
        let cells = vec!["1".to_string(), "Fernando Mendoza".to_string()];
        assert!(parse_drafttek_row(&cells, 0).is_none());
    }

    #[test]
    fn test_parse_drafttek_row_header_row() {
        let cells = vec![
            "Rank".to_string(),
            "Player".to_string(),
            "College".to_string(),
            "Position".to_string(),
        ];
        // "Rank" doesn't parse as i32 -> rank=1, "POSITION" not in known_positions
        assert!(parse_drafttek_row(&cells, 0).is_none());
    }

    #[test]
    fn test_parse_drafttek_row_empty_name() {
        let cells = vec![
            "1".to_string(),
            "".to_string(),
            "Indiana".to_string(),
            "QB".to_string(),
        ];
        assert!(parse_drafttek_row(&cells, 0).is_none());
    }
}
