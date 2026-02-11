use std::collections::HashMap;

use anyhow::Result;
use domain::models::{Player, ProspectRanking, RankingSource};
use domain::repositories::{PlayerRepository, RankingSourceRepository, TeamRepository};
use uuid::Uuid;

use crate::grade_generator::create_scouting_report;
use crate::position_mapper::map_position;
use crate::scouting_report_loader::{RankingData, RankingEntry};

/// Convert a Position enum to its string representation for SQL binding
fn position_to_str(pos: &domain::models::Position) -> &'static str {
    use domain::models::Position;
    match pos {
        Position::QB => "QB",
        Position::RB => "RB",
        Position::WR => "WR",
        Position::TE => "TE",
        Position::OT => "OT",
        Position::OG => "OG",
        Position::C => "C",
        Position::DE => "DE",
        Position::DT => "DT",
        Position::LB => "LB",
        Position::CB => "CB",
        Position::S => "S",
        Position::K => "K",
        Position::P => "P",
    }
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
    player_repo: &dyn PlayerRepository,
    team_repo: &dyn TeamRepository,
    ranking_source_repo: &dyn RankingSourceRepository,
    pool: &sqlx::PgPool,
) -> Result<RankingsLoadStats> {
    let mut stats = RankingsLoadStats::default();

    println!(
        "Loading rankings from {} ({} prospects)...",
        data.meta.source,
        data.rankings.len()
    );

    // Find or create the ranking source
    let source = match ranking_source_repo
        .find_by_name(&data.meta.source)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to look up ranking source: {}", e))?
    {
        Some(s) => {
            println!("Found existing ranking source: {} ({})", s.name, s.id);
            s
        }
        None => {
            println!("Creating new ranking source: {}", data.meta.source);
            let new_source = RankingSource::new(data.meta.source.clone())?;
            let new_source = match new_source.with_url(data.meta.source_url.clone()) {
                Ok(s) => s,
                Err(_) => RankingSource::new(data.meta.source.clone())?,
            };
            ranking_source_repo
                .create(&new_source)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create ranking source: {}", e))?
        }
    };

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

    // Start a transaction
    let mut tx = pool.begin().await?;

    // Delete existing rankings for this source (replace strategy)
    let deleted = sqlx::query("DELETE FROM prospect_rankings WHERE ranking_source_id = $1")
        .bind(source.id)
        .execute(&mut *tx)
        .await?;
    if deleted.rows_affected() > 0 {
        println!(
            "Cleared {} existing rankings for source '{}'",
            deleted.rows_affected(),
            source.name
        );
    }

    // Track newly created players for scouting report generation
    let mut new_player_entries: Vec<(Uuid, &RankingEntry)> = Vec::new();

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

            // Insert the player into the database
            let player_id = new_player.id;
            sqlx::query(
                "INSERT INTO players \
                 (id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year, draft_eligible, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
            )
            .bind(new_player.id)
            .bind(&new_player.first_name)
            .bind(&new_player.last_name)
            .bind(position_to_str(&new_player.position))
            .bind(&new_player.college)
            .bind(new_player.height_inches)
            .bind(new_player.weight_pounds)
            .bind(new_player.draft_year)
            .bind(new_player.draft_eligible)
            .bind(new_player.created_at)
            .bind(new_player.updated_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
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

        // Create the prospect ranking
        let ranking = ProspectRanking::new(source.id, player_id, entry.rank, scraped_at)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create ranking for {} {}: {}",
                    entry.first_name,
                    entry.last_name,
                    e
                )
            })?;

        sqlx::query(
            "INSERT INTO prospect_rankings (id, ranking_source_id, player_id, rank, scraped_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(ranking.id)
        .bind(ranking.ranking_source_id)
        .bind(ranking.player_id)
        .bind(ranking.rank)
        .bind(ranking.scraped_at)
        .bind(ranking.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to insert ranking for {} {}: {}",
                entry.first_name,
                entry.last_name,
                e
            )
        })?;

        stats.rankings_inserted += 1;
    }

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

                let fit_grade_str = report.fit_grade.map(|g| g.as_str().to_string());
                sqlx::query(
                    "INSERT INTO scouting_reports \
                     (id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
                )
                .bind(report.id)
                .bind(report.player_id)
                .bind(report.team_id)
                .bind(report.grade)
                .bind(&report.notes)
                .bind(&fit_grade_str)
                .bind(report.injury_concern)
                .bind(report.character_concern)
                .bind(report.created_at)
                .bind(report.updated_at)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
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

    tx.commit().await?;

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

pub async fn clear_rankings(
    source_name: &str,
    ranking_source_repo: &dyn RankingSourceRepository,
    pool: &sqlx::PgPool,
) -> Result<u64> {
    let source = ranking_source_repo
        .find_by_name(source_name)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to look up ranking source: {}", e))?;

    match source {
        Some(s) => {
            let result = sqlx::query("DELETE FROM prospect_rankings WHERE ranking_source_id = $1")
                .bind(s.id)
                .execute(pool)
                .await?;
            Ok(result.rows_affected())
        }
        None => {
            println!("No ranking source found with name '{}'", source_name);
            Ok(0)
        }
    }
}
