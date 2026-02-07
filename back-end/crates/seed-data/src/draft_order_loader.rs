use anyhow::Result;
use domain::models::{Draft, DraftPick, DraftStatus};
use domain::repositories::{DraftPickRepository, DraftRepository, TeamRepository};
use serde::Deserialize;

use crate::{COMPENSATORY_ROUND_MAX, COMPENSATORY_ROUND_MIN, MAX_DRAFT_ROUND};

#[derive(Debug, Deserialize)]
pub struct DraftOrderData {
    pub meta: DraftOrderMeta,
    pub draft_order: Vec<DraftOrderEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DraftOrderMeta {
    pub version: String,
    pub last_updated: String,
    pub sources: Vec<String>,
    /// Origin of draft order data: "template" or "tankathon"
    #[serde(default)]
    pub source: Option<String>,
    pub draft_year: i32,
    pub total_rounds: i32,
    pub total_picks: usize,
}

#[derive(Debug, Deserialize)]
pub struct DraftOrderEntry {
    pub round: i32,
    pub pick_in_round: i32,
    pub overall_pick: i32,
    pub team_abbreviation: String,
    pub original_team_abbreviation: String,
    pub is_compensatory: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Default)]
pub struct DraftOrderLoadStats {
    pub picks_processed: usize,
    pub picks_created: usize,
    pub teams_skipped: usize,
    pub draft_created: bool,
    pub draft_reused: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl DraftOrderLoadStats {
    pub fn print_summary(&self) {
        println!("\nLoad Summary:");
        println!("  Picks processed: {}", self.picks_processed);
        println!("  Picks created:   {}", self.picks_created);
        println!("  Teams skipped:   {}", self.teams_skipped);
        if self.draft_created {
            println!("  Draft: created new");
        } else if self.draft_reused {
            println!("  Draft: reused existing (picks replaced)");
        }
        if !self.warnings.is_empty() {
            println!("\nWarnings ({}):", self.warnings.len());
            for warning in &self.warnings {
                println!("  [WARN] {}", warning);
            }
        }
        println!("  Errors:          {}", self.errors.len());
        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }
    }
}

pub fn parse_draft_order_file(file_path: &str) -> Result<DraftOrderData> {
    let content = std::fs::read_to_string(file_path)?;
    let data: DraftOrderData = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn parse_draft_order_json(json: &str) -> Result<DraftOrderData> {
    let data: DraftOrderData = serde_json::from_str(json)?;
    Ok(data)
}

pub fn load_draft_order_dry_run(data: &DraftOrderData) -> Result<DraftOrderLoadStats> {
    let mut stats = DraftOrderLoadStats::default();

    for entry in &data.draft_order {
        // Validate round
        if entry.round < 1 || entry.round > MAX_DRAFT_ROUND {
            let msg = format!(
                "Invalid round {} for overall pick {}",
                entry.round, entry.overall_pick
            );
            tracing::error!("{}", msg);
            stats.errors.push(msg);
            continue;
        }

        // Validate pick numbers
        if entry.pick_in_round < 1 {
            let msg = format!(
                "Invalid pick_in_round {} for overall pick {}",
                entry.pick_in_round, entry.overall_pick
            );
            tracing::error!("{}", msg);
            stats.errors.push(msg);
            continue;
        }

        if entry.overall_pick < 1 {
            let msg = format!("Invalid overall_pick {}", entry.overall_pick);
            tracing::error!("{}", msg);
            stats.errors.push(msg);
            continue;
        }

        // Validate compensatory picks only in rounds 3-7
        if entry.is_compensatory
            && (entry.round < COMPENSATORY_ROUND_MIN || entry.round > COMPENSATORY_ROUND_MAX)
        {
            let msg = format!(
                "Compensatory pick in round {} (overall {}): compensatory picks only allowed in rounds 3-7",
                entry.round, entry.overall_pick
            );
            tracing::error!("{}", msg);
            stats.errors.push(msg);
            continue;
        }

        let traded = if entry.team_abbreviation != entry.original_team_abbreviation {
            format!(" (from {})", entry.original_team_abbreviation)
        } else {
            String::new()
        };

        let comp = if entry.is_compensatory { " [COMP]" } else { "" };

        println!(
            "[DRY RUN] Pick {}: Round {} Pick {} - {}{}{}",
            entry.overall_pick,
            entry.round,
            entry.pick_in_round,
            entry.team_abbreviation,
            traded,
            comp,
        );

        stats.picks_processed += 1;
        stats.picks_created += 1;
    }

    stats.draft_created = true;
    Ok(stats)
}

/// Maximum number of consecutive failures before aborting.
const MAX_CONSECUTIVE_FAILURES: usize = 5;

pub async fn load_draft_order(
    data: &DraftOrderData,
    team_repo: &dyn TeamRepository,
    draft_repo: &dyn DraftRepository,
    pick_repo: &dyn DraftPickRepository,
) -> Result<DraftOrderLoadStats> {
    let mut stats = DraftOrderLoadStats::default();
    let mut consecutive_failures: usize = 0;
    let year = data.meta.draft_year;
    let rounds = data.meta.total_rounds;

    // Check if a NotStarted realistic draft already exists for this year to reuse
    let draft = match draft_repo.find_by_year(year).await {
        Ok(existing_drafts) => {
            // Look for a NotStarted realistic draft to reuse
            let reusable = existing_drafts
                .into_iter()
                .find(|d| d.status == DraftStatus::NotStarted && d.is_realistic());

            if let Some(existing) = reusable {
                // Delete existing picks and reuse draft
                tracing::info!(
                    "Found existing NotStarted realistic draft for year {}. Replacing picks.",
                    year
                );
                if let Err(e) = pick_repo.delete_by_draft_id(existing.id).await {
                    let msg = format!("Failed to delete existing picks: {}", e);
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                    return Ok(stats);
                }
                stats.draft_reused = true;
                existing
            } else {
                // Create new realistic draft
                let draft = Draft::new_realistic(year, rounds)
                    .map_err(|e| anyhow::anyhow!("Failed to create draft: {}", e))?;
                let created = draft_repo
                    .create(&draft)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to persist draft: {}", e))?;
                tracing::info!("Created new realistic draft for year {}", year);
                stats.draft_created = true;
                created
            }
        }
        Err(e) => {
            let msg = format!("Failed to check for existing draft: {}", e);
            tracing::error!("{}", msg);
            stats.errors.push(msg);
            return Ok(stats);
        }
    };

    // Build all picks first, then batch insert
    let mut picks = Vec::new();

    for entry in &data.draft_order {
        // Look up team by abbreviation
        let team = match team_repo
            .find_by_abbreviation(&entry.team_abbreviation)
            .await
        {
            Ok(Some(t)) => t,
            Ok(None) => {
                let msg = format!(
                    "Team not found: {} (pick {}). Ensure teams are loaded first.",
                    entry.team_abbreviation, entry.overall_pick
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

        // Look up original team if different
        let original_team_id = if entry.original_team_abbreviation != entry.team_abbreviation {
            match team_repo
                .find_by_abbreviation(&entry.original_team_abbreviation)
                .await
            {
                Ok(Some(t)) => Some(t.id),
                Ok(None) => {
                    let msg = format!(
                        "Original team not found: {} (pick {}). Trade metadata will be missing.",
                        entry.original_team_abbreviation, entry.overall_pick
                    );
                    tracing::warn!("{}", msg);
                    stats.warnings.push(msg);
                    None
                }
                Err(e) => {
                    let msg = format!(
                        "Failed to lookup original team {}: {}. Trade metadata will be missing.",
                        entry.original_team_abbreviation, e
                    );
                    tracing::warn!("{}", msg);
                    stats.warnings.push(msg);
                    None
                }
            }
        } else {
            None
        };

        // Create pick
        let pick = match DraftPick::new_realistic(
            draft.id,
            entry.round,
            entry.pick_in_round,
            entry.overall_pick,
            team.id,
            original_team_id,
            entry.is_compensatory,
            entry.notes.clone(),
        ) {
            Ok(p) => p,
            Err(e) => {
                let msg = format!(
                    "Failed to create pick {} for {}: {}",
                    entry.overall_pick, entry.team_abbreviation, e
                );
                tracing::error!("{}", msg);
                stats.errors.push(msg);
                continue;
            }
        };

        picks.push(pick);
        stats.picks_processed += 1;
        consecutive_failures = 0;
    }

    // Batch insert all picks
    if !picks.is_empty() {
        match pick_repo.create_many(&picks).await {
            Ok(created) => {
                stats.picks_created = created.len();
                tracing::info!("Created {} draft picks for year {}", created.len(), year);
            }
            Err(e) => {
                let msg = format!("Failed to batch insert picks: {}", e);
                tracing::error!("{}", msg);
                stats.errors.push(msg);
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
                "last_updated": "2026-02-06",
                "sources": ["Test"],
                "draft_year": 2026,
                "total_rounds": 7,
                "total_picks": 3
            },
            "draft_order": [
                {
                    "round": 1,
                    "pick_in_round": 1,
                    "overall_pick": 1,
                    "team_abbreviation": "TEN",
                    "original_team_abbreviation": "TEN",
                    "is_compensatory": false,
                    "notes": null
                },
                {
                    "round": 1,
                    "pick_in_round": 2,
                    "overall_pick": 2,
                    "team_abbreviation": "DAL",
                    "original_team_abbreviation": "GB",
                    "is_compensatory": false,
                    "notes": "From Green Bay"
                },
                {
                    "round": 3,
                    "pick_in_round": 33,
                    "overall_pick": 97,
                    "team_abbreviation": "NE",
                    "original_team_abbreviation": "NE",
                    "is_compensatory": true,
                    "notes": "Compensatory pick"
                }
            ]
        }"#
    }

    #[test]
    fn test_parse_json() {
        let data: DraftOrderData = serde_json::from_str(sample_json()).unwrap();
        assert_eq!(data.meta.draft_year, 2026);
        assert_eq!(data.meta.total_rounds, 7);
        assert_eq!(data.meta.total_picks, 3);
        assert_eq!(data.draft_order.len(), 3);
        assert_eq!(data.draft_order[0].team_abbreviation, "TEN");
        assert!(!data.draft_order[0].is_compensatory);
        assert_eq!(
            data.draft_order[1].notes,
            Some("From Green Bay".to_string())
        );
        assert!(data.draft_order[2].is_compensatory);
    }

    #[test]
    fn test_dry_run() {
        let data: DraftOrderData = serde_json::from_str(sample_json()).unwrap();
        let stats = load_draft_order_dry_run(&data).unwrap();
        assert_eq!(stats.picks_processed, 3);
        assert_eq!(stats.picks_created, 3);
        assert_eq!(stats.teams_skipped, 0);
        assert!(stats.errors.is_empty());
        assert!(stats.draft_created);
    }

    #[test]
    fn test_dry_run_invalid_round() {
        let json = r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-06",
                "sources": ["Test"],
                "draft_year": 2026,
                "total_rounds": 7,
                "total_picks": 1
            },
            "draft_order": [
                {
                    "round": 0,
                    "pick_in_round": 1,
                    "overall_pick": 1,
                    "team_abbreviation": "TEN",
                    "original_team_abbreviation": "TEN",
                    "is_compensatory": false,
                    "notes": null
                }
            ]
        }"#;

        let data: DraftOrderData = serde_json::from_str(json).unwrap();
        let stats = load_draft_order_dry_run(&data).unwrap();
        assert_eq!(stats.picks_processed, 0);
        assert_eq!(stats.errors.len(), 1);
        assert!(stats.errors[0].contains("Invalid round"));
    }

    #[test]
    fn test_dry_run_compensatory_in_round_1() {
        let json = r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-06",
                "sources": ["Test"],
                "draft_year": 2026,
                "total_rounds": 7,
                "total_picks": 1
            },
            "draft_order": [
                {
                    "round": 1,
                    "pick_in_round": 1,
                    "overall_pick": 1,
                    "team_abbreviation": "TEN",
                    "original_team_abbreviation": "TEN",
                    "is_compensatory": true,
                    "notes": null
                }
            ]
        }"#;

        let data: DraftOrderData = serde_json::from_str(json).unwrap();
        let stats = load_draft_order_dry_run(&data).unwrap();
        assert_eq!(stats.picks_processed, 0);
        assert_eq!(stats.errors.len(), 1);
        assert!(stats.errors[0].contains("Compensatory pick in round 1"));
    }

    #[test]
    fn test_dry_run_traded_pick_display() {
        let json = r#"{
            "meta": {
                "version": "1.0.0",
                "last_updated": "2026-02-06",
                "sources": ["Test"],
                "draft_year": 2026,
                "total_rounds": 1,
                "total_picks": 1
            },
            "draft_order": [
                {
                    "round": 1,
                    "pick_in_round": 1,
                    "overall_pick": 1,
                    "team_abbreviation": "DAL",
                    "original_team_abbreviation": "GB",
                    "is_compensatory": false,
                    "notes": "From Green Bay via trade"
                }
            ]
        }"#;

        let data: DraftOrderData = serde_json::from_str(json).unwrap();
        let stats = load_draft_order_dry_run(&data).unwrap();
        assert_eq!(stats.picks_processed, 1);
        assert!(stats.errors.is_empty());
    }
}
