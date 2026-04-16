//! Backfill scouting reports for every draft-year player that doesn't yet
//! have one.
//!
//! The auto-pick engine silently skips any player without a scouting report
//! for the drafting team (see `auto_pick.rs`), which means Beast-only
//! prospects (discovered from `the_beast_2026.json` but absent from the
//! tankathon/walterfootball consensus file) are invisible to every team
//! and never get drafted. That made top-graded Brugler prospects appear to
//! "fall" into rounds 2–3 in the simulator.
//!
//! This loader closes the gap: after the consensus `scouting load` step has
//! run, it fills in reports × 32 teams for every unscouted player using the
//! Beast `grade_tier` as the consensus grade (with a neutral floor fallback
//! for players who have neither a rank nor a tier).

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use domain::repositories::{ProspectProfileRepository, ScoutingReportRepository, TeamRepository};

use crate::grade_generator::{create_scouting_report_with_grade, grade_tier_to_consensus_grade};

/// Slug of the Brugler "Beast" source as written by `the_beast_loader`.
const BEAST_SOURCE: &str = "the-beast-2026";

#[derive(Debug, Default)]
pub struct ScoutingBackfillStats {
    pub players_scanned: usize,
    pub players_already_scouted: usize,
    pub players_backfilled: usize,
    pub players_with_beast_tier: usize,
    pub players_with_neutral_grade: usize,
    pub reports_created: usize,
    pub errors: Vec<String>,
}

impl ScoutingBackfillStats {
    pub fn print_summary(&self) {
        println!("\nScouting Backfill Summary:");
        println!("  Players scanned:              {}", self.players_scanned);
        println!(
            "  Already scouted (skipped):    {}",
            self.players_already_scouted
        );
        println!(
            "  Players backfilled:           {}",
            self.players_backfilled
        );
        println!(
            "    - with Beast grade tier:    {}",
            self.players_with_beast_tier
        );
        println!(
            "    - with neutral fallback:    {}",
            self.players_with_neutral_grade
        );
        println!("  Reports created:              {}", self.reports_created);
        println!("  Errors:                       {}", self.errors.len());
        if !self.errors.is_empty() {
            for err in self.errors.iter().take(10) {
                println!("    - {}", err);
            }
            if self.errors.len() > 10 {
                println!("    ... and {} more", self.errors.len() - 10);
            }
        }
    }
}

/// One row from the players table needed for report generation.
struct PlayerRow {
    id: Uuid,
    first_name: String,
    last_name: String,
}

/// Backfill scouting reports for every player in `draft_year` that has no
/// existing scouting reports.
///
/// For each unscouted player we create a report for every team using
/// `grade_tier_to_consensus_grade(tier)` when the player has a Beast
/// profile with a grade tier, or a neutral floor (3.0) otherwise. The
/// per-team variation pipeline (`generate_team_grade`, fit grade, concern
/// flags) is identical to the one used by the consensus `scouting load`
/// step so the resulting reports are indistinguishable in shape.
pub async fn backfill_scouting_reports(
    pool: &PgPool,
    draft_year: i32,
    team_repo: &dyn TeamRepository,
    profile_repo: &dyn ProspectProfileRepository,
    scouting_report_repo: &dyn ScoutingReportRepository,
) -> Result<ScoutingBackfillStats> {
    let mut stats = ScoutingBackfillStats::default();

    // 1. Pull every player for this draft year (id + name).
    let players: Vec<PlayerRow> = sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT id, first_name, last_name FROM players WHERE draft_year = $1",
    )
    .bind(draft_year)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        anyhow::anyhow!(
            "Failed to fetch players for draft year {}: {}",
            draft_year,
            e
        )
    })?
    .into_iter()
    .map(|(id, first_name, last_name)| PlayerRow {
        id,
        first_name,
        last_name,
    })
    .collect();
    stats.players_scanned = players.len();

    if players.is_empty() {
        println!(
            "No players found for draft year {}; nothing to backfill.",
            draft_year
        );
        return Ok(stats);
    }

    // 2. Load every team so we can fan out reports per player.
    let teams = team_repo
        .find_all()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch teams: {}", e))?;
    if teams.is_empty() {
        anyhow::bail!("No teams found; load teams before running backfill.");
    }

    // 3. Find which players already have at least one scouting report. One
    //    query using a DISTINCT projection beats N `find_by_player_id`.
    let scouted_player_ids: HashSet<Uuid> = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT sr.player_id FROM scouting_reports sr \
         JOIN players p ON p.id = sr.player_id \
         WHERE p.draft_year = $1",
    )
    .bind(draft_year)
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to fetch scouted player ids: {}", e))?
    .into_iter()
    .collect();

    // 4. Pre-load every Beast profile once so the tier lookup is O(1).
    let beast_tier_by_player: HashMap<Uuid, String> =
        match profile_repo.find_by_source(BEAST_SOURCE).await {
            Ok(profiles) => profiles
                .into_iter()
                .filter_map(|p| p.grade_tier.map(|t| (p.player_id, t)))
                .collect(),
            Err(e) => {
                // Not fatal — fall back to neutral grades for everyone.
                tracing::warn!(
                    "Failed to load Beast profiles for backfill ({}); neutral grades only",
                    e
                );
                HashMap::new()
            }
        };

    // 5. For each player without a report, create reports × all teams.
    for player in &players {
        if scouted_player_ids.contains(&player.id) {
            stats.players_already_scouted += 1;
            continue;
        }

        let (consensus_grade, has_tier) = match beast_tier_by_player.get(&player.id) {
            Some(tier) => (grade_tier_to_consensus_grade(tier), true),
            None => (3.0, false),
        };
        if has_tier {
            stats.players_with_beast_tier += 1;
        } else {
            stats.players_with_neutral_grade += 1;
        }

        for team in &teams {
            let report = match create_scouting_report_with_grade(
                player.id,
                team.id,
                &team.abbreviation,
                &player.first_name,
                &player.last_name,
                consensus_grade,
            ) {
                Ok(r) => r,
                Err(e) => {
                    stats.errors.push(format!(
                        "Failed to build report for {} {} / {}: {}",
                        player.first_name, player.last_name, team.abbreviation, e
                    ));
                    continue;
                }
            };

            if let Err(e) = scouting_report_repo.create(&report).await {
                stats.errors.push(format!(
                    "Failed to insert report for {} {} / {}: {}",
                    player.first_name, player.last_name, team.abbreviation, e
                ));
                continue;
            }
            stats.reports_created += 1;
        }
        stats.players_backfilled += 1;
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_print_summary_does_not_panic_on_empty_errors() {
        let stats = ScoutingBackfillStats::default();
        stats.print_summary();
    }

    #[test]
    fn stats_print_summary_truncates_large_error_lists() {
        let mut stats = ScoutingBackfillStats::default();
        for i in 0..25 {
            stats.errors.push(format!("error {}", i));
        }
        // Must not panic; truncation message is visually verified.
        stats.print_summary();
    }
}
