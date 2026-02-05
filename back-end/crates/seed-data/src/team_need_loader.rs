use anyhow::Result;
use domain::models::TeamNeed;
use domain::repositories::{TeamNeedRepository, TeamRepository};
use serde::Deserialize;

use crate::position_mapper::map_position;

#[derive(Debug, Deserialize)]
pub struct TeamNeedData {
    pub meta: TeamNeedMetaData,
    pub team_needs: Vec<TeamNeedEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TeamNeedMetaData {
    pub version: String,
    pub last_updated: String,
    pub sources: Vec<String>,
    pub total_teams: usize,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct TeamNeedEntry {
    pub team_abbreviation: String,
    pub needs: Vec<PositionalNeed>,
}

#[derive(Debug, Deserialize)]
pub struct PositionalNeed {
    pub position: String,
    pub priority: i32,
}

#[derive(Debug, Default)]
pub struct TeamNeedLoadStats {
    pub teams_processed: usize,
    pub needs_created: usize,
    pub teams_skipped: usize,
    pub errors: Vec<String>,
}

impl TeamNeedLoadStats {
    pub fn print_summary(&self) {
        println!("\nLoad Summary:");
        println!("  Teams processed: {}", self.teams_processed);
        println!("  Needs created:   {}", self.needs_created);
        println!("  Teams skipped:   {}", self.teams_skipped);
        println!("  Errors:          {}", self.errors.len());
        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }
    }
}

pub fn parse_team_need_file(file_path: &str) -> Result<TeamNeedData> {
    let content = std::fs::read_to_string(file_path)?;
    let data: TeamNeedData = serde_json::from_str(&content)?;
    Ok(data)
}

/// Maximum number of consecutive failures before aborting.
const MAX_CONSECUTIVE_FAILURES: usize = 5;

pub fn load_team_needs_dry_run(data: &TeamNeedData) -> Result<TeamNeedLoadStats> {
    let mut stats = TeamNeedLoadStats::default();

    for entry in &data.team_needs {
        let mut needs_for_team = 0;

        for need in &entry.needs {
            match map_position(&need.position) {
                Ok(position) => {
                    println!(
                        "[DRY RUN] Would insert: {} - {:?} (priority {})",
                        entry.team_abbreviation, position, need.priority
                    );
                    needs_for_team += 1;
                }
                Err(e) => {
                    let msg = format!(
                        "Invalid position '{}' for {}: {}",
                        need.position, entry.team_abbreviation, e
                    );
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                }
            }
        }

        if needs_for_team > 0 {
            stats.teams_processed += 1;
            stats.needs_created += needs_for_team;
        }
    }

    Ok(stats)
}

pub async fn load_team_needs(
    data: &TeamNeedData,
    team_repo: &dyn TeamRepository,
    team_need_repo: &dyn TeamNeedRepository,
) -> Result<TeamNeedLoadStats> {
    let mut stats = TeamNeedLoadStats::default();
    let mut consecutive_failures: usize = 0;

    for entry in &data.team_needs {
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

        // Delete existing needs for this team (fresh load)
        if let Err(e) = team_need_repo.delete_by_team_id(team.id).await {
            let msg = format!(
                "Failed to clear existing needs for {}: {}",
                entry.team_abbreviation, e
            );
            tracing::error!("{}", msg);
            stats.errors.push(msg);
            stats.teams_skipped += 1;
            continue;
        }

        let mut needs_created = 0;
        let mut team_had_error = false;

        for need in &entry.needs {
            let position = match map_position(&need.position) {
                Ok(p) => p,
                Err(e) => {
                    let msg = format!(
                        "Invalid position '{}' for {}: {}",
                        need.position, entry.team_abbreviation, e
                    );
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                    team_had_error = true;
                    continue;
                }
            };

            let team_need = match TeamNeed::new(team.id, position, need.priority) {
                Ok(tn) => tn,
                Err(e) => {
                    let msg = format!(
                        "Failed to create team need for {} {:?}: {}",
                        entry.team_abbreviation, position, e
                    );
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                    team_had_error = true;
                    continue;
                }
            };

            match team_need_repo.create(&team_need).await {
                Ok(_) => {
                    tracing::info!(
                        "Inserted: {} - {:?} (priority {})",
                        entry.team_abbreviation,
                        position,
                        need.priority
                    );
                    needs_created += 1;
                }
                Err(e) => {
                    let msg = format!(
                        "Failed to insert need for {} {:?}: {}",
                        entry.team_abbreviation, position, e
                    );
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                    team_had_error = true;
                }
            }
        }

        if needs_created > 0 {
            stats.teams_processed += 1;
            stats.needs_created += needs_created;
            consecutive_failures = 0;
        }

        if team_had_error && needs_created == 0 {
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
                "sources": ["Test"],
                "total_teams": 2,
                "description": "Test data"
            },
            "team_needs": [
                {
                    "team_abbreviation": "DAL",
                    "needs": [
                        { "position": "QB", "priority": 1 },
                        { "position": "OT", "priority": 2 },
                        { "position": "EDGE", "priority": 3 }
                    ]
                },
                {
                    "team_abbreviation": "BUF",
                    "needs": [
                        { "position": "CB", "priority": 1 },
                        { "position": "WR", "priority": 2 }
                    ]
                }
            ]
        }"#
    }

    #[test]
    fn test_parse_json() {
        let data: TeamNeedData = serde_json::from_str(sample_json()).unwrap();
        assert_eq!(data.meta.total_teams, 2);
        assert_eq!(data.team_needs.len(), 2);
        assert_eq!(data.team_needs[0].team_abbreviation, "DAL");
        assert_eq!(data.team_needs[0].needs.len(), 3);
        assert_eq!(data.team_needs[0].needs[0].position, "QB");
        assert_eq!(data.team_needs[0].needs[0].priority, 1);
    }

    #[test]
    fn test_dry_run() {
        let data: TeamNeedData = serde_json::from_str(sample_json()).unwrap();
        let stats = load_team_needs_dry_run(&data).unwrap();
        assert_eq!(stats.teams_processed, 2);
        assert_eq!(stats.needs_created, 5);
        assert_eq!(stats.teams_skipped, 0);
        assert!(stats.errors.is_empty());
    }

    #[test]
    fn test_dry_run_with_invalid_position() {
        let json = r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-04",
                "sources": ["Test"],
                "total_teams": 1,
                "description": "Test data"
            },
            "team_needs": [
                {
                    "team_abbreviation": "DAL",
                    "needs": [
                        { "position": "QB", "priority": 1 },
                        { "position": "INVALID", "priority": 2 }
                    ]
                }
            ]
        }"#;

        let data: TeamNeedData = serde_json::from_str(json).unwrap();
        let stats = load_team_needs_dry_run(&data).unwrap();
        assert_eq!(stats.teams_processed, 1);
        assert_eq!(stats.needs_created, 1);
        assert_eq!(stats.errors.len(), 1);
        assert!(stats.errors[0].contains("INVALID"));
    }

    #[test]
    fn test_edge_maps_to_de() {
        let json = r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-04",
                "sources": ["Test"],
                "total_teams": 1,
                "description": "Test data"
            },
            "team_needs": [
                {
                    "team_abbreviation": "DAL",
                    "needs": [
                        { "position": "EDGE", "priority": 1 }
                    ]
                }
            ]
        }"#;

        let data: TeamNeedData = serde_json::from_str(json).unwrap();
        let stats = load_team_needs_dry_run(&data).unwrap();
        assert_eq!(stats.teams_processed, 1);
        assert_eq!(stats.needs_created, 1);
        assert!(stats.errors.is_empty());
    }
}
