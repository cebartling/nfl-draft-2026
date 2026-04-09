//! Loader for the JSON output of the TS/Bun "The Beast 2026" PDF scraper.
//!
//! Pipeline:
//!   1. Read JSON file → BeastFile struct
//!   2. Upsert ranking_sources row for "the-beast-2026"
//!   3. For each prospect:
//!      a. Find existing player by normalized name OR auto-create
//!      b. Upsert prospect_profile (carries grade tier, prose, strengths, weaknesses, ...)
//!      c. Upsert combine_results rows (combine + pro_day, when present)
//!      d. Insert prospect_ranking row when overall_rank is set (Top 100)
//!
//! Mirrors the upsert/discover pattern from `rankings_loader.rs`.

use std::collections::HashMap;

use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use domain::models::{
    CombineResults, CombineSource, Player, ProspectProfile, ProspectRanking, RankingSource,
};
use domain::repositories::{
    CombineResultsRepository, PlayerRepository, ProspectProfileRepository,
    ProspectRankingRepository, RankingSourceRepository,
};

use crate::position_mapper::map_position;
use crate::rankings_loader::normalize_name;

// ============================================================================
// JSON file shape (mirrors scrapers/src/types/the-beast.ts BeastDataSchema)
// ============================================================================

#[derive(Debug, Deserialize, Serialize)]
pub struct BeastFile {
    pub meta: BeastMeta,
    pub prospects: Vec<BeastProspect>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BeastMeta {
    pub version: String,
    pub source: String,
    pub source_url: String,
    pub draft_year: i32,
    pub scraped_at: String,
    pub total_prospects: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BeastProspect {
    pub position: String,
    pub position_rank: i32,
    pub overall_rank: Option<i32>,
    pub first_name: String,
    pub last_name: String,
    pub school: String,
    pub grade_tier: Option<String>,
    pub year_class: Option<String>,
    pub birthday: Option<String>, // YYYY-MM-DD
    pub age: Option<f64>,
    pub jersey_number: Option<String>,
    pub height_inches: Option<i32>,
    pub weight_pounds: Option<i32>,
    pub height_raw: Option<String>,
    pub forty_yard_dash: Option<f64>,
    pub ten_yard_split: Option<f64>,
    pub hand_size: Option<f64>,
    pub arm_length: Option<f64>,
    pub wingspan: Option<f64>,
    pub combine: Option<BeastMeasurables>,
    pub pro_day: Option<BeastMeasurables>,
    pub college_stats: Vec<BeastStatRow>,
    pub background: Option<String>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub summary: Option<String>,
    pub nfl_comparison: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BeastMeasurables {
    pub height_raw: Option<String>,
    pub weight_pounds: Option<i32>,
    pub hand_size: Option<f64>,
    pub arm_length: Option<f64>,
    pub wingspan: Option<f64>,
    pub forty_yard_dash: Option<f64>,
    pub twenty_yard_split: Option<f64>,
    pub ten_yard_split: Option<f64>,
    pub vertical_jump: Option<f64>,
    pub broad_jump: Option<i32>,
    pub twenty_yard_shuttle: Option<f64>,
    pub three_cone_drill: Option<f64>,
    pub bench_press: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BeastStatRow {
    pub year: String,
    #[serde(default)]
    pub notes: Option<String>,
}

pub fn parse_beast_file(path: &str) -> Result<BeastFile> {
    let json = std::fs::read_to_string(path)?;
    let data: BeastFile = serde_json::from_str(&json)?;
    Ok(data)
}

// ============================================================================
// Stats
// ============================================================================

#[derive(Debug, Default)]
pub struct BeastLoadStats {
    pub prospects_seen: usize,
    pub players_matched: usize,
    pub players_discovered: usize,
    pub profiles_upserted: usize,
    pub combine_rows_upserted: usize,
    pub prodays_upserted: usize,
    pub rankings_inserted: usize,
    pub skipped_invalid_position: usize,
    pub errors: Vec<String>,
}

impl BeastLoadStats {
    pub fn print_summary(&self) {
        println!("\nThe Beast 2026 Load Summary:");
        println!("  Prospects seen:           {}", self.prospects_seen);
        println!("  Players matched:          {}", self.players_matched);
        println!("  New players discovered:   {}", self.players_discovered);
        println!("  Profiles upserted:        {}", self.profiles_upserted);
        println!("  Combine rows upserted:    {}", self.combine_rows_upserted);
        println!("  Pro day rows upserted:    {}", self.prodays_upserted);
        println!("  Rankings inserted:        {}", self.rankings_inserted);
        println!(
            "  Skipped (invalid pos):    {}",
            self.skipped_invalid_position
        );
        println!("  Errors:                   {}", self.errors.len());
        if !self.errors.is_empty() {
            for err in self.errors.iter().take(20) {
                println!("    - {}", err);
            }
            if self.errors.len() > 20 {
                println!("    ... and {} more", self.errors.len() - 20);
            }
        }
    }
}

// ============================================================================
// Dry run
// ============================================================================

pub fn load_beast_dry_run(data: &BeastFile) -> Result<BeastLoadStats> {
    let mut stats = BeastLoadStats::default();
    println!(
        "[DRY RUN] Would load {} prospects from '{}' (draft year {})",
        data.prospects.len(),
        data.meta.source,
        data.meta.draft_year
    );
    for p in &data.prospects {
        if map_position(&p.position).is_err() {
            stats.skipped_invalid_position += 1;
            continue;
        }
        stats.prospects_seen += 1;
        stats.profiles_upserted += 1;
        if p.combine.is_some() {
            stats.combine_rows_upserted += 1;
        }
        if p.pro_day.is_some() {
            stats.prodays_upserted += 1;
        }
        if p.overall_rank.is_some() {
            stats.rankings_inserted += 1;
        }
    }
    Ok(stats)
}

// ============================================================================
// Live load
// ============================================================================

pub async fn load_beast(
    data: &BeastFile,
    pool: &PgPool,
    player_repo: &dyn PlayerRepository,
    profile_repo: &dyn ProspectProfileRepository,
    combine_repo: &dyn CombineResultsRepository,
    ranking_source_repo: &dyn RankingSourceRepository,
    _ranking_repo: &dyn ProspectRankingRepository,
) -> Result<BeastLoadStats> {
    let mut stats = BeastLoadStats::default();

    println!(
        "Loading The Beast from {} ({} prospects, draft year {})...",
        data.meta.source,
        data.prospects.len(),
        data.meta.draft_year
    );

    let source = find_or_create_source(&data.meta, ranking_source_repo).await?;

    // Pre-load existing players for the draft year for fast name lookup.
    let existing_players = player_repo
        .find_by_draft_year(data.meta.draft_year)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch players: {}", e))?;
    let mut player_map: HashMap<(String, String), Player> = existing_players
        .into_iter()
        .map(|p| {
            (
                (normalize_name(&p.first_name), normalize_name(&p.last_name)),
                p,
            )
        })
        .collect();
    println!("Found {} existing players for matching", player_map.len());

    let scraped_at = NaiveDate::parse_from_str(&data.meta.scraped_at, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("Invalid scraped_at '{}': {}", data.meta.scraped_at, e))?;

    // Collected ranking rows for the Top 100 batch insert at the end.
    let mut rankings_to_insert: Vec<ProspectRanking> = Vec::new();

    for entry in &data.prospects {
        stats.prospects_seen += 1;

        let position = match map_position(&entry.position) {
            Ok(p) => p,
            Err(e) => {
                stats.skipped_invalid_position += 1;
                stats.errors.push(format!(
                    "Skipping {} {} ({}): {}",
                    entry.first_name, entry.last_name, entry.position, e
                ));
                continue;
            }
        };

        // Match or create player.
        let key = (
            normalize_name(&entry.first_name),
            normalize_name(&entry.last_name),
        );
        let player_id = if let Some(existing) = player_map.get(&key) {
            stats.players_matched += 1;
            existing.id
        } else {
            let mut new_player = match Player::new(
                entry.first_name.clone(),
                entry.last_name.clone(),
                position,
                data.meta.draft_year,
            ) {
                Ok(p) => p,
                Err(e) => {
                    stats.errors.push(format!(
                        "Failed to construct player {} {}: {}",
                        entry.first_name, entry.last_name, e
                    ));
                    continue;
                }
            };
            if !entry.school.trim().is_empty() {
                new_player = new_player
                    .with_college(entry.school.clone())
                    .unwrap_or_else(|_| {
                        // College value rejected by validation; fall back to no college
                        Player::new(
                            entry.first_name.clone(),
                            entry.last_name.clone(),
                            position,
                            data.meta.draft_year,
                        )
                        .unwrap()
                    });
            }
            if let (Some(h), Some(w)) = (entry.height_inches, entry.weight_pounds) {
                if let Ok(p) = new_player.clone().with_physical_stats(h, w) {
                    new_player = p;
                }
            }

            let pid = new_player.id;
            if let Err(e) = player_repo.create(&new_player).await {
                stats.errors.push(format!(
                    "Failed to insert player {} {}: {}",
                    entry.first_name, entry.last_name, e
                ));
                continue;
            }
            stats.players_discovered += 1;
            player_map.insert(key, new_player);
            pid
        };

        // Build and upsert prospect_profile.
        let mut profile = match ProspectProfile::new(
            player_id,
            data.meta.source.clone(),
            entry.position_rank,
            scraped_at,
        ) {
            Ok(p) => p,
            Err(e) => {
                stats.errors.push(format!(
                    "Failed to construct profile for {} {}: {}",
                    entry.first_name, entry.last_name, e
                ));
                continue;
            }
        };
        if let Some(rank) = entry.overall_rank {
            if let Ok(p) = profile.clone().with_overall_rank(rank) {
                profile = p;
            }
        }
        if let Some(ref tier) = entry.grade_tier {
            profile = profile.with_grade_tier(tier.clone());
        }
        if let Some(ref yc) = entry.year_class {
            profile = profile.with_year_class(yc.clone());
        }
        if let Some(ref bday) = entry.birthday {
            if let Ok(d) = NaiveDate::parse_from_str(bday, "%Y-%m-%d") {
                profile = profile.with_birthday(d);
            }
        }
        if let Some(ref jersey) = entry.jersey_number {
            profile = profile.with_jersey_number(jersey.clone());
        }
        if let Some(ref hr) = entry.height_raw {
            profile = profile.with_height_raw(hr.clone());
        }
        if let Some(ref bg) = entry.background {
            profile = profile.with_background(bg.clone());
        }
        if let Some(ref summary) = entry.summary {
            profile = profile.with_summary(summary.clone());
        }
        if !entry.strengths.is_empty() {
            profile = profile.with_strengths(entry.strengths.clone());
        }
        if !entry.weaknesses.is_empty() {
            profile = profile.with_weaknesses(entry.weaknesses.clone());
        }
        if let Some(ref comp) = entry.nfl_comparison {
            profile = profile.with_nfl_comparison(comp.clone());
        }
        if !entry.college_stats.is_empty() {
            if let Ok(json) = serde_json::to_value(&entry.college_stats) {
                profile = profile.with_college_stats(json);
            }
        }

        match profile_repo.upsert(&profile).await {
            Ok(_) => stats.profiles_upserted += 1,
            Err(e) => stats.errors.push(format!(
                "Failed to upsert profile for {} {}: {}",
                entry.first_name, entry.last_name, e
            )),
        }

        // Combine results: write combine row + pro_day row when present.
        if let Some(ref m) = entry.combine {
            if upsert_combine(
                player_id,
                data.meta.draft_year,
                CombineSource::Combine,
                m,
                combine_repo,
            )
            .await
            .is_ok()
            {
                stats.combine_rows_upserted += 1;
            }
        }
        if let Some(ref m) = entry.pro_day {
            if upsert_combine(
                player_id,
                data.meta.draft_year,
                CombineSource::ProDay,
                m,
                combine_repo,
            )
            .await
            .is_ok()
            {
                stats.prodays_upserted += 1;
            }
        }

        // Top-100-style ranking row (only when an overall rank is present).
        if let Some(rank) = entry.overall_rank {
            if let Ok(r) = ProspectRanking::new(source.id, player_id, rank, scraped_at) {
                rankings_to_insert.push(r);
            }
        }
    }

    // Replace any existing rankings for this source in one transaction.
    if !rankings_to_insert.is_empty() {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM prospect_rankings WHERE ranking_source_id = $1")
            .bind(source.id)
            .execute(&mut *tx)
            .await?;

        let ids: Vec<Uuid> = rankings_to_insert.iter().map(|r| r.id).collect();
        let source_ids: Vec<Uuid> = rankings_to_insert
            .iter()
            .map(|r| r.ranking_source_id)
            .collect();
        let player_ids: Vec<Uuid> = rankings_to_insert.iter().map(|r| r.player_id).collect();
        let ranks: Vec<i32> = rankings_to_insert.iter().map(|r| r.rank).collect();
        let scraped_dates: Vec<NaiveDate> =
            rankings_to_insert.iter().map(|r| r.scraped_at).collect();
        let created_dates: Vec<chrono::DateTime<chrono::Utc>> =
            rankings_to_insert.iter().map(|r| r.created_at).collect();

        let result = sqlx::query(
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
        .await?;

        stats.rankings_inserted = result.rows_affected() as usize;
        tx.commit().await?;
    }

    // Suppress unused-import warning when JsonValue isn't referenced after refactors.
    let _ = JsonValue::Null;

    Ok(stats)
}

async fn upsert_combine(
    player_id: Uuid,
    year: i32,
    source: CombineSource,
    m: &BeastMeasurables,
    repo: &dyn CombineResultsRepository,
) -> Result<()> {
    let source_str = match source {
        CombineSource::Combine => "combine",
        CombineSource::ProDay => "pro_day",
    };
    let existing = repo
        .find_by_player_year_source(player_id, year, source_str)
        .await
        .map_err(|e| anyhow::anyhow!("combine lookup failed: {}", e))?;

    let mut record = CombineResults::new(player_id, year)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .with_source(source);
    if let Some(v) = m.forty_yard_dash {
        if let Ok(r) = record.clone().with_forty_yard_dash(v) {
            record = r;
        }
    }
    if let Some(v) = m.ten_yard_split {
        if let Ok(r) = record.clone().with_ten_yard_split(v) {
            record = r;
        }
    }
    if let Some(v) = m.twenty_yard_split {
        if let Ok(r) = record.clone().with_twenty_yard_split(v) {
            record = r;
        }
    }
    if let Some(v) = m.vertical_jump {
        if let Ok(r) = record.clone().with_vertical_jump(v) {
            record = r;
        }
    }
    if let Some(v) = m.broad_jump {
        if let Ok(r) = record.clone().with_broad_jump(v) {
            record = r;
        }
    }
    if let Some(v) = m.three_cone_drill {
        if let Ok(r) = record.clone().with_three_cone_drill(v) {
            record = r;
        }
    }
    if let Some(v) = m.twenty_yard_shuttle {
        if let Ok(r) = record.clone().with_twenty_yard_shuttle(v) {
            record = r;
        }
    }
    if let Some(v) = m.bench_press {
        if let Ok(r) = record.clone().with_bench_press(v) {
            record = r;
        }
    }
    if let Some(v) = m.hand_size {
        if let Ok(r) = record.clone().with_hand_size(v) {
            record = r;
        }
    }
    if let Some(v) = m.arm_length {
        if let Ok(r) = record.clone().with_arm_length(v) {
            record = r;
        }
    }
    if let Some(v) = m.wingspan {
        if let Ok(r) = record.clone().with_wingspan(v) {
            record = r;
        }
    }

    if let Some(prev) = existing {
        // Preserve the existing id so the update updates the right row.
        let mut to_update = record;
        to_update.id = prev.id;
        repo.update(&to_update)
            .await
            .map_err(|e| anyhow::anyhow!("combine update failed: {}", e))?;
    } else {
        repo.create(&record)
            .await
            .map_err(|e| anyhow::anyhow!("combine create failed: {}", e))?;
    }
    Ok(())
}

async fn find_or_create_source(
    meta: &BeastMeta,
    repo: &dyn RankingSourceRepository,
) -> Result<RankingSource> {
    if let Some(s) = repo
        .find_by_name(&meta.source)
        .await
        .map_err(|e| anyhow::anyhow!("ranking source lookup failed: {}", e))?
    {
        println!("Found existing ranking source: {} ({})", s.name, s.id);
        return Ok(s);
    }

    let mut new_source = RankingSource::new(meta.source.clone())?;
    if !meta.source_url.is_empty() && meta.source_url != "N/A" {
        if let Ok(s) = new_source.clone().with_url(meta.source_url.clone()) {
            new_source = s;
        }
    }
    let created = repo
        .create(&new_source)
        .await
        .map_err(|e| anyhow::anyhow!("ranking source create failed: {}", e))?;
    println!("Created ranking source: {} ({})", created.name, created.id);
    Ok(created)
}
