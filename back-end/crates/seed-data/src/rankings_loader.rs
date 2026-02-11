use std::collections::HashMap;

use anyhow::Result;
use domain::models::{Player, ProspectRanking, RankingSource};
use domain::repositories::{
    PlayerRepository, ProspectRankingRepository, RankingSourceRepository,
    ScoutingReportRepository, TeamRepository,
};
use uuid::Uuid;

use crate::grade_generator::create_scouting_report;
use crate::position_mapper::map_position;
use crate::scouting_report_loader::{RankingData, RankingEntry};

#[derive(Debug, Default)]
pub struct RankingsLoadStats {
    pub prospects_matched: usize,
    pub prospects_discovered: usize,
    pub rankings_inserted: usize,
    pub scouting_reports_created: usize,
    pub errors: Vec<String>,
    pub discovered_names: Vec<String>,
}

impl RankingsLoadStats {
    pub fn print_summary(&self) {
        println!("\nRankings Load Summary:");
        println!("  Prospects matched:        {}", self.prospects_matched);
        println!("  New prospects discovered:  {}", self.prospects_discovered);
        println!("  Rankings inserted:         {}", self.rankings_inserted);
        println!(
            "  Scouting reports created:  {}",
            self.scouting_reports_created
        );
        println!("  Errors:                    {}", self.errors.len());

        if !self.discovered_names.is_empty() {
            println!("\nNewly discovered prospects:");
            for name in &self.discovered_names {
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

pub fn load_rankings_dry_run(data: &RankingData) -> Result<RankingsLoadStats> {
    let mut stats = RankingsLoadStats::default();

    println!(
        "[DRY RUN] Would load {} rankings from source '{}'",
        data.rankings.len(),
        data.meta.source
    );

    for entry in &data.rankings {
        println!(
            "[DRY RUN] Rank {}: {} {} ({}, {})",
            entry.rank, entry.first_name, entry.last_name, entry.position, entry.school
        );
        stats.prospects_matched += 1;
        stats.rankings_inserted += 1;
    }

    Ok(stats)
}

pub async fn load_rankings(
    data: &RankingData,
    player_repo: &dyn PlayerRepository,
    team_repo: &dyn TeamRepository,
    ranking_source_repo: &dyn RankingSourceRepository,
    prospect_ranking_repo: &dyn ProspectRankingRepository,
    scouting_report_repo: &dyn ScoutingReportRepository,
) -> Result<RankingsLoadStats> {
    let mut stats = RankingsLoadStats::default();

    println!(
        "Loading rankings from {} ({} prospects)...",
        data.meta.source,
        data.rankings.len()
    );

    // Find or create the ranking source
    let source = find_or_create_source(data, ranking_source_repo).await?;

    // Load all teams for scouting report generation
    let teams = team_repo
        .find_all()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch teams: {}", e))?;

    if teams.is_empty() {
        anyhow::bail!("No teams found in database. Load teams first.");
    }

    // Load existing players and build lookup map
    let players = player_repo
        .find_by_draft_year(data.meta.draft_year)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch players: {}", e))?;

    println!(
        "Found {} existing players for draft year {}",
        players.len(),
        data.meta.draft_year
    );

    let mut player_map: HashMap<(String, String), Player> = players
        .into_iter()
        .map(|p| {
            (
                (p.first_name.to_lowercase(), p.last_name.to_lowercase()),
                p,
            )
        })
        .collect();

    // Parse scraped_at date
    let scraped_at = chrono::NaiveDate::parse_from_str(&data.meta.scraped_at, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Utc::now().date_naive());

    // Delete existing rankings for this source (replace strategy)
    let deleted = prospect_ranking_repo
        .delete_by_source(source.id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete existing rankings: {}", e))?;
    if deleted > 0 {
        println!(
            "Cleared {} existing rankings for source '{}'",
            deleted, source.name
        );
    }

    // Track newly created players for scouting report generation
    let mut new_player_entries: Vec<(Uuid, &RankingEntry)> = Vec::new();
    let mut rankings_to_insert: Vec<ProspectRanking> = Vec::new();

    // Process each ranking entry
    for entry in &data.rankings {
        let lookup_key = (
            entry.first_name.to_lowercase(),
            entry.last_name.to_lowercase(),
        );

        // Match or create player
        let player_id = if let Some(existing) = player_map.get(&lookup_key) {
            stats.prospects_matched += 1;
            existing.id
        } else {
            // Auto-discover: create a new player
            let position = match map_position(&entry.position) {
                Ok(p) => p,
                Err(e) => {
                    let msg = format!(
                        "Skipping {} {} - unknown position '{}': {}",
                        entry.first_name, entry.last_name, entry.position, e
                    );
                    tracing::warn!("{}", msg);
                    stats.errors.push(msg);
                    continue;
                }
            };

            let new_player = Player::new(
                entry.first_name.clone(),
                entry.last_name.clone(),
                position,
                data.meta.draft_year,
            )
            .and_then(|p| {
                if entry.school.trim().is_empty() {
                    Ok(p)
                } else {
                    p.with_college(entry.school.clone())
                }
            })
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create player {} {}: {}",
                    entry.first_name,
                    entry.last_name,
                    e
                )
            })?;

            let player_id = new_player.id;
            player_repo.create(&new_player).await.map_err(|e| {
                anyhow::anyhow!(
                    "Failed to insert player {} {}: {}",
                    entry.first_name,
                    entry.last_name,
                    e
                )
            })?;

            let name = format!(
                "{} {} ({}, {})",
                entry.first_name, entry.last_name, entry.position, entry.school
            );
            println!("  New prospect discovered: {} [created]", name);
            stats.prospects_discovered += 1;
            stats.discovered_names.push(name);

            // Track for scouting report generation
            new_player_entries.push((player_id, entry));

            // Add to our lookup map so we don't create duplicates
            player_map.insert(lookup_key, new_player);

            player_id
        };

        // Collect the prospect ranking for batch insert
        let ranking = ProspectRanking::new(source.id, player_id, entry.rank, scraped_at)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create ranking for {} {}: {}",
                    entry.first_name,
                    entry.last_name,
                    e
                )
            })?;

        rankings_to_insert.push(ranking);
    }

    // Batch insert all rankings
    let inserted = prospect_ranking_repo
        .create_batch(&rankings_to_insert)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert rankings batch: {}", e))?;
    stats.rankings_inserted = inserted;

    // Generate scouting reports for newly discovered prospects
    if !new_player_entries.is_empty() {
        println!(
            "\nGenerating scouting reports for {} new prospects ({} teams each)...",
            new_player_entries.len(),
            teams.len()
        );

        for (player_id, entry) in &new_player_entries {
            for team in &teams {
                let report = match create_scouting_report(
                    *player_id,
                    team.id,
                    &team.abbreviation,
                    &entry.first_name,
                    &entry.last_name,
                    entry.rank,
                ) {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to create scouting report for {} {} / {}: {}",
                            entry.first_name,
                            entry.last_name,
                            team.abbreviation,
                            e
                        );
                        continue;
                    }
                };

                scouting_report_repo.create(&report).await.map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to insert scouting report for {} {} / {}: {}",
                        entry.first_name,
                        entry.last_name,
                        team.abbreviation,
                        e
                    )
                })?;

                stats.scouting_reports_created += 1;
            }
        }

        println!(
            "  Generated {} scouting reports for {} new prospects",
            stats.scouting_reports_created,
            new_player_entries.len()
        );
    }

    println!(
        "  Matched {} prospects to existing players",
        stats.prospects_matched
    );
    println!(
        "  Inserted {} prospect rankings",
        stats.rankings_inserted
    );

    Ok(stats)
}

async fn find_or_create_source(
    data: &RankingData,
    ranking_source_repo: &dyn RankingSourceRepository,
) -> Result<RankingSource> {
    match ranking_source_repo
        .find_by_name(&data.meta.source)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to look up ranking source: {}", e))?
    {
        Some(s) => {
            println!("Found existing ranking source: {} ({})", s.name, s.id);
            Ok(s)
        }
        None => {
            println!("Creating new ranking source: {}", data.meta.source);
            let mut new_source = RankingSource::new(data.meta.source.clone())?;
            if let Ok(s) = new_source.clone().with_url(data.meta.source_url.clone()) {
                new_source = s;
            }
            ranking_source_repo
                .create(&new_source)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create ranking source: {}", e))
        }
    }
}

pub async fn clear_rankings(
    source_name: &str,
    ranking_source_repo: &dyn RankingSourceRepository,
    prospect_ranking_repo: &dyn ProspectRankingRepository,
) -> Result<u64> {
    let source = ranking_source_repo
        .find_by_name(source_name)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to look up ranking source: {}", e))?;

    match source {
        Some(s) => {
            let deleted = prospect_ranking_repo
                .delete_by_source(s.id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to delete rankings: {}", e))?;
            Ok(deleted)
        }
        None => {
            println!("No ranking source found with name '{}'", source_name);
            Ok(0)
        }
    }
}
