use std::collections::HashMap;

use anyhow::Result;
use domain::models::{FeldmanFreak, Player};
use domain::repositories::{FeldmanFreakRepository, PlayerRepository};
use serde::Deserialize;

use crate::rankings_loader::normalize_name;

#[derive(Debug, Deserialize)]
pub struct FreaksData {
    pub meta: FreaksMeta,
    pub freaks: Vec<FreakEntry>,
}

#[derive(Debug, Deserialize)]
pub struct FreaksMeta {
    pub year: i32,
    pub source: String,
    pub article_url: String,
}

#[derive(Debug, Deserialize)]
pub struct FreakEntry {
    pub rank: i32,
    pub first_name: String,
    pub last_name: String,
    pub college: String,
    pub position: String,
    pub description: String,
}

pub fn parse_freaks_file(path: &str) -> Result<FreaksData> {
    let content = std::fs::read_to_string(path)?;
    parse_freaks_json(&content)
}

pub fn parse_freaks_json(json: &str) -> Result<FreaksData> {
    let data: FreaksData = serde_json::from_str(json)?;
    Ok(data)
}

#[derive(Debug, Default)]
pub struct FreaksLoadStats {
    pub matched: usize,
    pub unmatched: usize,
    pub inserted: usize,
    pub errors: Vec<String>,
    pub unmatched_names: Vec<String>,
}

impl FreaksLoadStats {
    pub fn print_summary(&self) {
        println!("\nFeldman Freaks Load Summary:");
        println!("  Players matched:    {}", self.matched);
        println!("  Players unmatched:  {}", self.unmatched);
        println!("  Freaks inserted:    {}", self.inserted);
        println!("  Errors:             {}", self.errors.len());

        if !self.unmatched_names.is_empty() {
            println!("\nUnmatched players (not found in database):");
            for name in &self.unmatched_names {
                println!("  - {}", name);
            }
        }

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }
    }
}

pub fn load_freaks_dry_run(data: &FreaksData) -> Result<FreaksLoadStats> {
    let mut stats = FreaksLoadStats::default();

    println!(
        "[DRY RUN] Would load {} freaks for year {} from '{}'",
        data.freaks.len(),
        data.meta.year,
        data.meta.source
    );

    for entry in &data.freaks {
        println!(
            "[DRY RUN] Rank {}: {} {} ({}, {})",
            entry.rank, entry.first_name, entry.last_name, entry.position, entry.college
        );
        stats.matched += 1;
        stats.inserted += 1;
    }

    Ok(stats)
}

pub async fn load_freaks(
    data: &FreaksData,
    player_repo: &dyn PlayerRepository,
    freak_repo: &dyn FeldmanFreakRepository,
) -> Result<FreaksLoadStats> {
    let mut stats = FreaksLoadStats::default();

    println!(
        "Loading {} Feldman Freaks for year {}...",
        data.freaks.len(),
        data.meta.year
    );

    // Load existing players and build lookup map by normalized name
    let players = player_repo
        .find_by_draft_year(data.meta.year)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch players: {}", e))?;

    println!(
        "Found {} existing players for draft year {}",
        players.len(),
        data.meta.year
    );

    let player_map: HashMap<(String, String), &Player> = players
        .iter()
        .map(|p| {
            (
                (normalize_name(&p.first_name), normalize_name(&p.last_name)),
                p,
            )
        })
        .collect();

    // Delete existing freaks for this year before inserting
    let deleted = freak_repo
        .delete_by_year(data.meta.year)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete existing freaks: {}", e))?;

    if deleted > 0 {
        println!(
            "Cleared {} existing Feldman Freaks for year {}",
            deleted, data.meta.year
        );
    }

    // Match and insert each freak
    for entry in &data.freaks {
        let lookup_key = (
            normalize_name(&entry.first_name),
            normalize_name(&entry.last_name),
        );

        let player = match player_map.get(&lookup_key) {
            Some(p) => {
                stats.matched += 1;
                *p
            }
            None => {
                let name = format!(
                    "{} {} ({}, {})",
                    entry.first_name, entry.last_name, entry.position, entry.college
                );
                tracing::warn!("No matching player for Freak #{}: {}", entry.rank, name);
                stats.unmatched += 1;
                stats.unmatched_names.push(name);
                continue;
            }
        };

        let freak = FeldmanFreak::new(
            player.id,
            data.meta.year,
            entry.rank,
            entry.description.clone(),
        )
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to create FeldmanFreak for {} {}: {}",
                entry.first_name,
                entry.last_name,
                e
            )
        })?;

        let freak = if !data.meta.article_url.is_empty() {
            freak
                .with_article_url(data.meta.article_url.clone())
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to set article URL for {} {}: {}",
                        entry.first_name,
                        entry.last_name,
                        e
                    )
                })?
        } else {
            freak
        };

        freak_repo.create(&freak).await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to insert Feldman Freak for {} {}: {}",
                entry.first_name,
                entry.last_name,
                e
            )
        })?;

        stats.inserted += 1;
    }

    println!(
        "  Matched {} players, inserted {} freaks",
        stats.matched, stats.inserted
    );

    Ok(stats)
}

pub async fn clear_freaks(year: i32, freak_repo: &dyn FeldmanFreakRepository) -> Result<u64> {
    let deleted = freak_repo
        .delete_by_year(year)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete freaks: {}", e))?;
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> &'static str {
        r#"{
            "meta": {
                "year": 2026,
                "source": "Bruce Feldman's Freaks List",
                "article_url": "https://example.com/freaks"
            },
            "freaks": [
                {
                    "rank": 1,
                    "first_name": "Kenyon",
                    "last_name": "Sadiq",
                    "college": "Oregon",
                    "position": "TE",
                    "description": "Vertical jumped 41.5 inches"
                },
                {
                    "rank": 2,
                    "first_name": "Sonny",
                    "last_name": "Styles",
                    "college": "Ohio State",
                    "position": "LB",
                    "description": "Broad jumped 11 feet"
                }
            ]
        }"#
    }

    #[test]
    fn test_parse_freaks_json() {
        let data = parse_freaks_json(sample_json()).unwrap();
        assert_eq!(data.meta.year, 2026);
        assert_eq!(data.meta.source, "Bruce Feldman's Freaks List");
        assert_eq!(
            data.meta.article_url,
            "https://example.com/freaks"
        );
        assert_eq!(data.freaks.len(), 2);
        assert_eq!(data.freaks[0].rank, 1);
        assert_eq!(data.freaks[0].first_name, "Kenyon");
        assert_eq!(data.freaks[0].last_name, "Sadiq");
        assert_eq!(data.freaks[0].college, "Oregon");
        assert_eq!(data.freaks[0].position, "TE");
        assert_eq!(data.freaks[1].rank, 2);
        assert_eq!(data.freaks[1].first_name, "Sonny");
    }

    #[test]
    fn test_parse_freaks_json_invalid() {
        let result = parse_freaks_json("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_freaks_json_missing_field() {
        let json = r#"{
            "meta": { "year": 2026, "source": "Test" },
            "freaks": []
        }"#;
        // Missing article_url in meta
        let result = parse_freaks_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_dry_run() {
        let data = parse_freaks_json(sample_json()).unwrap();
        let stats = load_freaks_dry_run(&data).unwrap();
        assert_eq!(stats.matched, 2);
        assert_eq!(stats.inserted, 2);
        assert_eq!(stats.unmatched, 0);
        assert!(stats.errors.is_empty());
        assert!(stats.unmatched_names.is_empty());
    }

    #[test]
    fn test_dry_run_empty_freaks() {
        let json = r#"{
            "meta": {
                "year": 2026,
                "source": "Test",
                "article_url": "https://example.com"
            },
            "freaks": []
        }"#;
        let data = parse_freaks_json(json).unwrap();
        let stats = load_freaks_dry_run(&data).unwrap();
        assert_eq!(stats.matched, 0);
        assert_eq!(stats.inserted, 0);
    }

    #[test]
    fn test_freaks_load_stats_default() {
        let stats = FreaksLoadStats::default();
        assert_eq!(stats.matched, 0);
        assert_eq!(stats.unmatched, 0);
        assert_eq!(stats.inserted, 0);
        assert!(stats.errors.is_empty());
        assert!(stats.unmatched_names.is_empty());
    }
}
