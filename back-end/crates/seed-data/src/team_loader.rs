use anyhow::{anyhow, Result};
use domain::models::{Conference, Division, Team};
use domain::repositories::TeamRepository;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TeamData {
    pub meta: TeamMetaData,
    pub teams: Vec<TeamEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TeamMetaData {
    pub version: String,
    pub last_updated: String,
    pub sources: Vec<String>,
    pub total_teams: usize,
}

#[derive(Debug, Deserialize)]
pub struct TeamEntry {
    pub name: String,
    pub abbreviation: String,
    pub city: String,
    pub conference: String,
    pub division: String,
}

impl TeamEntry {
    pub fn to_domain(&self) -> Result<Team> {
        let conference = map_conference(&self.conference)?;
        let division = map_division(&self.division)?;

        let team = Team::new(
            self.name.clone(),
            self.abbreviation.clone(),
            self.city.clone(),
            conference,
            division,
        )?;

        Ok(team)
    }
}

pub fn map_conference(conference: &str) -> Result<Conference> {
    match conference.to_uppercase().as_str() {
        "AFC" => Ok(Conference::AFC),
        "NFC" => Ok(Conference::NFC),
        _ => Err(anyhow!("Unknown conference: {}", conference)),
    }
}

pub fn map_division(division: &str) -> Result<Division> {
    match division {
        "AFC East" => Ok(Division::AFCEast),
        "AFC North" => Ok(Division::AFCNorth),
        "AFC South" => Ok(Division::AFCSouth),
        "AFC West" => Ok(Division::AFCWest),
        "NFC East" => Ok(Division::NFCEast),
        "NFC North" => Ok(Division::NFCNorth),
        "NFC South" => Ok(Division::NFCSouth),
        "NFC West" => Ok(Division::NFCWest),
        _ => Err(anyhow!("Unknown division: {}", division)),
    }
}

#[derive(Debug, Default)]
pub struct TeamLoadStats {
    pub success: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl TeamLoadStats {
    pub fn print_summary(&self) {
        println!("\nLoad Summary:");
        println!("  Succeeded: {}", self.success);
        println!("  Skipped:   {}", self.skipped);
        println!("  Errors:    {}", self.errors.len());
        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }
    }
}

pub fn parse_team_file(file_path: &str) -> Result<TeamData> {
    let content = std::fs::read_to_string(file_path)?;
    let data: TeamData = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn parse_team_json(json: &str) -> Result<TeamData> {
    let data: TeamData = serde_json::from_str(json)?;
    Ok(data)
}

/// Maximum number of consecutive failures before aborting.
const MAX_CONSECUTIVE_FAILURES: usize = 5;

pub fn load_teams_dry_run(data: &TeamData) -> Result<TeamLoadStats> {
    let mut stats = TeamLoadStats::default();
    let mut consecutive_failures: usize = 0;

    for entry in &data.teams {
        match entry.to_domain() {
            Ok(_) => {
                println!(
                    "[DRY RUN] Would insert: {} ({}) - {} {}",
                    entry.name, entry.abbreviation, entry.conference, entry.division
                );
                stats.success += 1;
                consecutive_failures = 0;
            }
            Err(e) => {
                let msg = format!(
                    "Validation failed for {} ({}): {}",
                    entry.name, entry.abbreviation, e
                );
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                consecutive_failures += 1;

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    let abort_msg = format!(
                        "Aborting: {} consecutive failures detected. This may indicate a systematic problem (e.g., schema mismatch).",
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

pub async fn load_teams(data: &TeamData, repo: &dyn TeamRepository) -> Result<TeamLoadStats> {
    let mut stats = TeamLoadStats::default();
    let mut consecutive_failures: usize = 0;

    for entry in &data.teams {
        // Check if team already exists by abbreviation (UNIQUE constraint)
        match repo.find_by_abbreviation(&entry.abbreviation).await {
            Ok(Some(existing)) => {
                tracing::warn!(
                    "Skipping {} ({}): team already exists with id {}",
                    entry.name,
                    entry.abbreviation,
                    existing.id
                );
                stats.skipped += 1;
                consecutive_failures = 0;
                continue;
            }
            Ok(None) => {
                // Team doesn't exist, proceed with creation
            }
            Err(e) => {
                let msg = format!(
                    "Failed to check existing team {} ({}): {}",
                    entry.name, entry.abbreviation, e
                );
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                consecutive_failures += 1;

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    let abort_msg = format!(
                        "Aborting: {} consecutive failures detected. This may indicate a systematic problem (e.g., database down).",
                        consecutive_failures
                    );
                    tracing::error!("{}", abort_msg);
                    stats.errors.push(abort_msg);
                    break;
                }
                continue;
            }
        }

        match entry.to_domain() {
            Ok(team) => match repo.create(&team).await {
                Ok(_) => {
                    tracing::info!(
                        "Inserted: {} ({}) - {} {}",
                        entry.name,
                        entry.abbreviation,
                        entry.conference,
                        entry.division
                    );
                    stats.success += 1;
                    consecutive_failures = 0;
                }
                Err(e) => {
                    let msg = format!(
                        "Failed to insert {} ({}): {}",
                        entry.name, entry.abbreviation, e
                    );
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                    consecutive_failures += 1;
                }
            },
            Err(e) => {
                let msg = format!(
                    "Validation failed for {} ({}): {}",
                    entry.name, entry.abbreviation, e
                );
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                consecutive_failures += 1;
            }
        }

        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            let abort_msg = format!(
                "Aborting: {} consecutive failures detected. This may indicate a systematic problem (e.g., database down, schema mismatch).",
                consecutive_failures
            );
            tracing::error!("{}", abort_msg);
            stats.errors.push(abort_msg);
            break;
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
                "total_teams": 2
            },
            "teams": [
                {
                    "name": "Dallas Cowboys",
                    "abbreviation": "DAL",
                    "city": "Arlington",
                    "conference": "NFC",
                    "division": "NFC East"
                },
                {
                    "name": "Buffalo Bills",
                    "abbreviation": "BUF",
                    "city": "Buffalo",
                    "conference": "AFC",
                    "division": "AFC East"
                }
            ]
        }"#
    }

    #[test]
    fn test_parse_json() {
        let data: TeamData = serde_json::from_str(sample_json()).unwrap();
        assert_eq!(data.meta.total_teams, 2);
        assert_eq!(data.teams.len(), 2);
        assert_eq!(data.teams[0].name, "Dallas Cowboys");
        assert_eq!(data.teams[0].abbreviation, "DAL");
    }

    #[test]
    fn test_team_entry_to_domain() {
        let entry = TeamEntry {
            name: "Dallas Cowboys".to_string(),
            abbreviation: "DAL".to_string(),
            city: "Arlington".to_string(),
            conference: "NFC".to_string(),
            division: "NFC East".to_string(),
        };

        let team = entry.to_domain().unwrap();
        assert_eq!(team.name, "Dallas Cowboys");
        assert_eq!(team.abbreviation, "DAL");
        assert_eq!(team.city, "Arlington");
        assert_eq!(team.conference, Conference::NFC);
        assert_eq!(team.division, Division::NFCEast);
    }

    #[test]
    fn test_map_conference() {
        assert_eq!(map_conference("AFC").unwrap(), Conference::AFC);
        assert_eq!(map_conference("NFC").unwrap(), Conference::NFC);
        assert_eq!(map_conference("afc").unwrap(), Conference::AFC);
        assert!(map_conference("XFL").is_err());
    }

    #[test]
    fn test_map_division() {
        assert_eq!(map_division("AFC East").unwrap(), Division::AFCEast);
        assert_eq!(map_division("AFC North").unwrap(), Division::AFCNorth);
        assert_eq!(map_division("AFC South").unwrap(), Division::AFCSouth);
        assert_eq!(map_division("AFC West").unwrap(), Division::AFCWest);
        assert_eq!(map_division("NFC East").unwrap(), Division::NFCEast);
        assert_eq!(map_division("NFC North").unwrap(), Division::NFCNorth);
        assert_eq!(map_division("NFC South").unwrap(), Division::NFCSouth);
        assert_eq!(map_division("NFC West").unwrap(), Division::NFCWest);
        assert!(map_division("Invalid Division").is_err());
    }

    #[test]
    fn test_team_entry_invalid_conference() {
        let entry = TeamEntry {
            name: "Test Team".to_string(),
            abbreviation: "TST".to_string(),
            city: "Test City".to_string(),
            conference: "XFL".to_string(),
            division: "NFC East".to_string(),
        };

        assert!(entry.to_domain().is_err());
    }

    #[test]
    fn test_team_entry_invalid_division() {
        let entry = TeamEntry {
            name: "Test Team".to_string(),
            abbreviation: "TST".to_string(),
            city: "Test City".to_string(),
            conference: "NFC".to_string(),
            division: "Invalid Division".to_string(),
        };

        assert!(entry.to_domain().is_err());
    }

    #[test]
    fn test_team_entry_mismatched_conference_division() {
        // NFC team with AFC division should fail domain validation
        let entry = TeamEntry {
            name: "Test Team".to_string(),
            abbreviation: "TST".to_string(),
            city: "Test City".to_string(),
            conference: "NFC".to_string(),
            division: "AFC East".to_string(),
        };

        assert!(entry.to_domain().is_err());
    }

    #[test]
    fn test_dry_run() {
        let data: TeamData = serde_json::from_str(sample_json()).unwrap();
        let stats = load_teams_dry_run(&data).unwrap();
        assert_eq!(stats.success, 2);
        assert_eq!(stats.skipped, 0);
        assert!(stats.errors.is_empty());
    }
}
