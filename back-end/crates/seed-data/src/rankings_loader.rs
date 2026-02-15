use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, NaiveDate};
use domain::models::{Player, ProspectRanking, RankingSource};
use domain::repositories::{
    PlayerRepository, ProspectRankingRepository, RankingSourceRepository, ScoutingReportRepository,
    TeamRepository,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::grade_generator::create_scouting_report;
use crate::position_mapper::map_position;
use crate::scouting_report_loader::{RankingData, RankingEntry};

/// Normalize a name component for matching by stripping periods and collapsing whitespace.
/// This handles variations like "C.J." vs "CJ", "Jr." vs "Jr", "L.T." vs "LT".
fn normalize_name(name: &str) -> String {
    name.replace('.', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

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
    pool: &PgPool,
    player_repo: &dyn PlayerRepository,
    team_repo: &dyn TeamRepository,
    ranking_source_repo: &dyn RankingSourceRepository,
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
                (normalize_name(&p.first_name), normalize_name(&p.last_name)),
                p,
            )
        })
        .collect();

    // Parse scraped_at date
    let scraped_at =
        chrono::NaiveDate::parse_from_str(&data.meta.scraped_at, "%Y-%m-%d").map_err(|e| {
            anyhow::anyhow!("Invalid scraped_at date '{}': {}", data.meta.scraped_at, e)
        })?;

    // Track newly created players for scouting report generation
    let mut new_player_entries: Vec<(Uuid, &RankingEntry)> = Vec::new();
    let mut rankings_to_insert: Vec<ProspectRanking> = Vec::new();

    // Process each ranking entry
    for entry in &data.rankings {
        let lookup_key = (
            normalize_name(&entry.first_name),
            normalize_name(&entry.last_name),
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
        let ranking =
            ProspectRanking::new(source.id, player_id, entry.rank, scraped_at).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create ranking for {} {}: {}",
                    entry.first_name,
                    entry.last_name,
                    e
                )
            })?;

        rankings_to_insert.push(ranking);
    }

    // Delete old + insert new rankings in a transaction (replace strategy)
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to begin transaction: {}", e))?;

    let delete_result = sqlx::query("DELETE FROM prospect_rankings WHERE ranking_source_id = $1")
        .bind(source.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete existing rankings: {}", e))?;

    if delete_result.rows_affected() > 0 {
        println!(
            "Cleared {} existing rankings for source '{}'",
            delete_result.rows_affected(),
            source.name
        );
    }

    if !rankings_to_insert.is_empty() {
        let ids: Vec<Uuid> = rankings_to_insert.iter().map(|r| r.id).collect();
        let source_ids: Vec<Uuid> = rankings_to_insert
            .iter()
            .map(|r| r.ranking_source_id)
            .collect();
        let player_ids: Vec<Uuid> = rankings_to_insert.iter().map(|r| r.player_id).collect();
        let ranks: Vec<i32> = rankings_to_insert.iter().map(|r| r.rank).collect();
        let scraped_dates: Vec<NaiveDate> =
            rankings_to_insert.iter().map(|r| r.scraped_at).collect();
        let created_dates: Vec<DateTime<chrono::Utc>> =
            rankings_to_insert.iter().map(|r| r.created_at).collect();

        let insert_result = sqlx::query(
            r#"
            INSERT INTO prospect_rankings (id, ranking_source_id, player_id, rank, scraped_at, created_at)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[], $4::int4[], $5::date[], $6::timestamptz[])
            "#,
        )
        .bind(&ids)
        .bind(&source_ids)
        .bind(&player_ids)
        .bind(&ranks)
        .bind(&scraped_dates)
        .bind(&created_dates)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert rankings batch: {}", e))?;

        stats.rankings_inserted = insert_result.rows_affected() as usize;
    }

    tx.commit()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to commit rankings transaction: {}", e))?;

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
    println!("  Inserted {} prospect rankings", stats.rankings_inserted);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name_strips_periods() {
        assert_eq!(normalize_name("C.J."), "cj");
        assert_eq!(normalize_name("L.T."), "lt");
        assert_eq!(normalize_name("K.C."), "kc");
        assert_eq!(normalize_name("T.J."), "tj");
    }

    #[test]
    fn test_normalize_name_handles_suffix() {
        assert_eq!(normalize_name("Jr."), "jr");
        assert_eq!(normalize_name("Jr"), "jr");
        assert_eq!(normalize_name("III"), "iii");
    }

    #[test]
    fn test_normalize_name_plain_names() {
        assert_eq!(normalize_name("Fernando"), "fernando");
        assert_eq!(normalize_name("Mendoza"), "mendoza");
    }

    #[test]
    fn test_normalize_name_collapses_whitespace() {
        assert_eq!(normalize_name("R. Mason"), "r mason");
        assert_eq!(normalize_name("R Mason"), "r mason");
    }

    #[test]
    fn test_normalize_name_matches_duplicates() {
        // C.J. Allen vs CJ Allen
        assert_eq!(normalize_name("C.J."), normalize_name("CJ"));
        // Harold Perkins Jr vs Harold Perkins Jr.
        assert_eq!(normalize_name("Perkins Jr"), normalize_name("Perkins Jr."));
        // L.T. vs LT
        assert_eq!(normalize_name("L.T."), normalize_name("LT"));
        // R. Mason vs R Mason
        assert_eq!(normalize_name("R."), normalize_name("R"));
    }
}
