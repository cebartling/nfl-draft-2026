use anyhow::Result;
use domain::models::{PlayoffResult, TeamSeason};
use domain::repositories::{TeamRepository, TeamSeasonRepository};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TeamSeasonData {
    pub meta: TeamSeasonMetaData,
    pub team_seasons: Vec<TeamSeasonEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TeamSeasonMetaData {
    pub version: String,
    pub last_updated: String,
    pub sources: Vec<String>,
    pub season_year: i32,
    pub total_teams: usize,
}

#[derive(Debug, Deserialize)]
pub struct TeamSeasonEntry {
    pub team_abbreviation: String,
    pub wins: i32,
    pub losses: i32,
    pub ties: i32,
    pub playoff_result: Option<String>,
    pub draft_position: Option<i32>,
}

#[derive(Debug, Default)]
pub struct TeamSeasonLoadStats {
    pub seasons_processed: usize,
    pub seasons_created: usize,
    pub seasons_updated: usize,
    pub teams_skipped: usize,
    pub errors: Vec<String>,
}

impl TeamSeasonLoadStats {
    pub fn print_summary(&self) {
        println!("\nLoad Summary:");
        println!("  Seasons processed: {}", self.seasons_processed);
        println!("  Seasons created:   {}", self.seasons_created);
        println!("  Seasons updated:   {}", self.seasons_updated);
        println!("  Teams skipped:     {}", self.teams_skipped);
        println!("  Errors:            {}", self.errors.len());
        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }
    }
}

pub fn parse_team_season_file(file_path: &str) -> Result<TeamSeasonData> {
    let content = std::fs::read_to_string(file_path)?;
    let data: TeamSeasonData = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn parse_team_season_json(json: &str) -> Result<TeamSeasonData> {
    let data: TeamSeasonData = serde_json::from_str(json)?;
    Ok(data)
}

fn parse_playoff_result(s: &str) -> Result<PlayoffResult, String> {
    s.parse()
        .map_err(|_| format!("Invalid playoff result: {}", s))
}

/// Maximum number of consecutive failures before aborting.
const MAX_CONSECUTIVE_FAILURES: usize = 5;

pub fn load_team_seasons_dry_run(data: &TeamSeasonData) -> Result<TeamSeasonLoadStats> {
    let mut stats = TeamSeasonLoadStats::default();

    for entry in &data.team_seasons {
        // Validate playoff result if present
        if let Some(ref pr) = entry.playoff_result {
            if let Err(e) = parse_playoff_result(pr) {
                let msg = format!(
                    "Invalid playoff result '{}' for {}: {}",
                    pr, entry.team_abbreviation, e
                );
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                continue;
            }
        }

        // Validate the record
        if entry.wins < 0
            || entry.wins > 17
            || entry.losses < 0
            || entry.losses > 17
            || entry.ties < 0
            || entry.ties > 17
        {
            let msg = format!(
                "Invalid record for {}: {}-{}-{}",
                entry.team_abbreviation, entry.wins, entry.losses, entry.ties
            );
            tracing::error!("{}", msg);
            stats.errors.push(msg);
            continue;
        }

        if entry.wins + entry.losses + entry.ties > 17 {
            let msg = format!(
                "Total games exceed 17 for {}: {}-{}-{}",
                entry.team_abbreviation, entry.wins, entry.losses, entry.ties
            );
            tracing::error!("{}", msg);
            stats.errors.push(msg);
            continue;
        }

        // Validate draft position
        if let Some(pos) = entry.draft_position {
            if !(1..=32).contains(&pos) {
                let msg = format!(
                    "Invalid draft position {} for {}",
                    pos, entry.team_abbreviation
                );
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                continue;
            }
        }

        println!(
            "[DRY RUN] Would upsert: {} - {}-{}-{} (draft position: {:?})",
            entry.team_abbreviation, entry.wins, entry.losses, entry.ties, entry.draft_position
        );
        stats.seasons_processed += 1;
        stats.seasons_created += 1;
    }

    Ok(stats)
}

pub async fn load_team_seasons(
    data: &TeamSeasonData,
    team_repo: &dyn TeamRepository,
    team_season_repo: &dyn TeamSeasonRepository,
) -> Result<TeamSeasonLoadStats> {
    let mut stats = TeamSeasonLoadStats::default();
    let mut consecutive_failures: usize = 0;
    let season_year = data.meta.season_year;

    for entry in &data.team_seasons {
        // Look up the team by abbreviation
        let team = match team_repo
            .find_by_abbreviation(&entry.team_abbreviation)
            .await
        {
            Ok(Some(t)) => t,
            Ok(None) => {
                let msg = format!(
                    "Team not found: {} - ensure teams are loaded first",
                    entry.team_abbreviation
                );
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                stats.teams_skipped += 1;
                consecutive_failures += 1;

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    let abort_msg = format!(
                        "Aborting: {} consecutive failures. Are teams loaded?",
                        consecutive_failures
                    );
                    tracing::error!("{}", abort_msg);
                    stats.errors.push(abort_msg);
                    break;
                }
                continue;
            }
            Err(e) => {
                let msg = format!("Failed to lookup team {}: {}", entry.team_abbreviation, e);
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                stats.teams_skipped += 1;
                consecutive_failures += 1;

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    let abort_msg = format!(
                        "Aborting: {} consecutive failures. Database issue?",
                        consecutive_failures
                    );
                    tracing::error!("{}", abort_msg);
                    stats.errors.push(abort_msg);
                    break;
                }
                continue;
            }
        };

        // Parse playoff result
        let playoff_result = match &entry.playoff_result {
            Some(pr) => match parse_playoff_result(pr) {
                Ok(result) => Some(result),
                Err(e) => {
                    let msg = format!(
                        "Invalid playoff result '{}' for {}: {}",
                        pr, entry.team_abbreviation, e
                    );
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                    stats.teams_skipped += 1;
                    continue;
                }
            },
            None => None,
        };

        // Check if season already exists
        let existing = team_season_repo
            .find_by_team_and_year(team.id, season_year)
            .await;
        let is_update = matches!(existing, Ok(Some(_)));

        // Create team season
        let team_season = match TeamSeason::new(
            team.id,
            season_year,
            entry.wins,
            entry.losses,
            entry.ties,
            playoff_result,
            entry.draft_position,
        ) {
            Ok(ts) => ts,
            Err(e) => {
                let msg = format!(
                    "Failed to create team season for {}: {}",
                    entry.team_abbreviation, e
                );
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                stats.teams_skipped += 1;
                continue;
            }
        };

        match team_season_repo.upsert(&team_season).await {
            Ok(_) => {
                if is_update {
                    tracing::info!(
                        "Updated: {} - {}-{}-{} (draft position: {:?})",
                        entry.team_abbreviation,
                        entry.wins,
                        entry.losses,
                        entry.ties,
                        entry.draft_position
                    );
                    stats.seasons_updated += 1;
                } else {
                    tracing::info!(
                        "Inserted: {} - {}-{}-{} (draft position: {:?})",
                        entry.team_abbreviation,
                        entry.wins,
                        entry.losses,
                        entry.ties,
                        entry.draft_position
                    );
                    stats.seasons_created += 1;
                }
                stats.seasons_processed += 1;
                consecutive_failures = 0;
            }
            Err(e) => {
                let msg = format!(
                    "Failed to upsert season for {}: {}",
                    entry.team_abbreviation, e
                );
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                stats.teams_skipped += 1;
                consecutive_failures += 1;

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    let abort_msg = format!(
                        "Aborting: {} consecutive failures. This may indicate a systematic problem.",
                        consecutive_failures
                    );
                    tracing::error!("{}", abort_msg);
                    stats.errors.push(abort_msg);
                    break;
                }
            }
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> &'static str {
        r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-04",
                "sources": ["NFL.com"],
                "season_year": 2025,
                "total_teams": 2
            },
            "team_seasons": [
                {
                    "team_abbreviation": "TEN",
                    "wins": 3,
                    "losses": 14,
                    "ties": 0,
                    "playoff_result": "MissedPlayoffs",
                    "draft_position": 1
                },
                {
                    "team_abbreviation": "PHI",
                    "wins": 14,
                    "losses": 3,
                    "ties": 0,
                    "playoff_result": "SuperBowlWin",
                    "draft_position": 32
                }
            ]
        }"#
    }

    #[test]
    fn test_parse_json() {
        let data: TeamSeasonData = serde_json::from_str(sample_json()).unwrap();
        assert_eq!(data.meta.season_year, 2025);
        assert_eq!(data.meta.total_teams, 2);
        assert_eq!(data.team_seasons.len(), 2);
        assert_eq!(data.team_seasons[0].team_abbreviation, "TEN");
        assert_eq!(data.team_seasons[0].wins, 3);
        assert_eq!(data.team_seasons[0].losses, 14);
        assert_eq!(data.team_seasons[0].ties, 0);
        assert_eq!(
            data.team_seasons[0].playoff_result,
            Some("MissedPlayoffs".to_string())
        );
        assert_eq!(data.team_seasons[0].draft_position, Some(1));
    }

    #[test]
    fn test_dry_run() {
        let data: TeamSeasonData = serde_json::from_str(sample_json()).unwrap();
        let stats = load_team_seasons_dry_run(&data).unwrap();
        assert_eq!(stats.seasons_processed, 2);
        assert_eq!(stats.seasons_created, 2);
        assert_eq!(stats.teams_skipped, 0);
        assert!(stats.errors.is_empty());
    }

    #[test]
    fn test_dry_run_with_invalid_playoff_result() {
        let json = r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-04",
                "sources": ["NFL.com"],
                "season_year": 2025,
                "total_teams": 1
            },
            "team_seasons": [
                {
                    "team_abbreviation": "TEN",
                    "wins": 3,
                    "losses": 14,
                    "ties": 0,
                    "playoff_result": "Invalid",
                    "draft_position": 1
                }
            ]
        }"#;

        let data: TeamSeasonData = serde_json::from_str(json).unwrap();
        let stats = load_team_seasons_dry_run(&data).unwrap();
        assert_eq!(stats.seasons_processed, 0);
        assert_eq!(stats.errors.len(), 1);
        assert!(stats.errors[0].contains("Invalid"));
    }

    #[test]
    fn test_dry_run_with_invalid_record() {
        let json = r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-04",
                "sources": ["NFL.com"],
                "season_year": 2025,
                "total_teams": 1
            },
            "team_seasons": [
                {
                    "team_abbreviation": "TEN",
                    "wins": 18,
                    "losses": 0,
                    "ties": 0,
                    "playoff_result": "MissedPlayoffs",
                    "draft_position": 1
                }
            ]
        }"#;

        let data: TeamSeasonData = serde_json::from_str(json).unwrap();
        let stats = load_team_seasons_dry_run(&data).unwrap();
        assert_eq!(stats.seasons_processed, 0);
        assert_eq!(stats.errors.len(), 1);
        assert!(stats.errors[0].contains("Invalid record"));
    }

    #[test]
    fn test_dry_run_with_too_many_games() {
        let json = r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-04",
                "sources": ["NFL.com"],
                "season_year": 2025,
                "total_teams": 1
            },
            "team_seasons": [
                {
                    "team_abbreviation": "TEN",
                    "wins": 10,
                    "losses": 5,
                    "ties": 5,
                    "playoff_result": null,
                    "draft_position": 1
                }
            ]
        }"#;

        let data: TeamSeasonData = serde_json::from_str(json).unwrap();
        let stats = load_team_seasons_dry_run(&data).unwrap();
        assert_eq!(stats.seasons_processed, 0);
        assert_eq!(stats.errors.len(), 1);
        assert!(stats.errors[0].contains("exceed 17"));
    }

    #[test]
    fn test_dry_run_with_invalid_draft_position() {
        let json = r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-04",
                "sources": ["NFL.com"],
                "season_year": 2025,
                "total_teams": 1
            },
            "team_seasons": [
                {
                    "team_abbreviation": "TEN",
                    "wins": 3,
                    "losses": 14,
                    "ties": 0,
                    "playoff_result": null,
                    "draft_position": 33
                }
            ]
        }"#;

        let data: TeamSeasonData = serde_json::from_str(json).unwrap();
        let stats = load_team_seasons_dry_run(&data).unwrap();
        assert_eq!(stats.seasons_processed, 0);
        assert_eq!(stats.errors.len(), 1);
        assert!(stats.errors[0].contains("Invalid draft position"));
    }

    #[test]
    fn test_parse_playoff_result_valid() {
        assert!(matches!(
            parse_playoff_result("MissedPlayoffs"),
            Ok(PlayoffResult::MissedPlayoffs)
        ));
        assert!(matches!(
            parse_playoff_result("WildCard"),
            Ok(PlayoffResult::WildCard)
        ));
        assert!(matches!(
            parse_playoff_result("Divisional"),
            Ok(PlayoffResult::Divisional)
        ));
        assert!(matches!(
            parse_playoff_result("Conference"),
            Ok(PlayoffResult::Conference)
        ));
        assert!(matches!(
            parse_playoff_result("SuperBowlLoss"),
            Ok(PlayoffResult::SuperBowlLoss)
        ));
        assert!(matches!(
            parse_playoff_result("SuperBowlWin"),
            Ok(PlayoffResult::SuperBowlWin)
        ));
    }

    #[test]
    fn test_parse_playoff_result_invalid() {
        assert!(parse_playoff_result("Invalid").is_err());
    }
}
