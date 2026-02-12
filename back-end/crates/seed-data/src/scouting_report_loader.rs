use std::collections::HashMap;

use anyhow::Result;
use domain::models::ScoutingReport;
use domain::repositories::{PlayerRepository, TeamRepository};
use serde::Deserialize;

use crate::grade_generator::{
    generate_concern_flags, generate_fit_grade, generate_team_grade, rank_to_grade,
};

#[derive(Debug, Deserialize)]
pub struct RankingData {
    pub meta: RankingMeta,
    pub rankings: Vec<RankingEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RankingMeta {
    pub version: String,
    pub source: String,
    pub source_url: String,
    pub draft_year: i32,
    pub scraped_at: String,
    pub total_prospects: usize,
}

#[derive(Debug, Deserialize)]
pub struct RankingEntry {
    pub rank: i32,
    pub first_name: String,
    pub last_name: String,
    pub position: String,
    pub school: String,
}

#[derive(Debug, Default)]
pub struct ScoutingReportLoadStats {
    pub prospects_matched: usize,
    pub prospects_unmatched: usize,
    pub reports_created: usize,
    pub reports_failed: usize,
    pub teams_used: usize,
    pub errors: Vec<String>,
    pub unmatched_names: Vec<String>,
}

impl ScoutingReportLoadStats {
    pub fn print_summary(&self) {
        println!("\nLoad Summary:");
        println!("  Prospects matched:   {}", self.prospects_matched);
        println!("  Prospects unmatched: {}", self.prospects_unmatched);
        println!("  Reports created:     {}", self.reports_created);
        println!("  Reports failed:      {}", self.reports_failed);
        println!("  Teams used:          {}", self.teams_used);
        println!("  Errors:              {}", self.errors.len());

        if !self.unmatched_names.is_empty() {
            println!("\nUnmatched prospects:");
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

pub fn parse_ranking_file(file_path: &str) -> Result<RankingData> {
    let content = std::fs::read_to_string(file_path)?;
    let data: RankingData = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn load_scouting_reports_dry_run(data: &RankingData) -> Result<ScoutingReportLoadStats> {
    let mut stats = ScoutingReportLoadStats::default();
    // In dry run, we assume 32 teams and simulate the fan-out
    let team_count = 32;
    stats.teams_used = team_count;

    for entry in &data.rankings {
        let consensus_grade = rank_to_grade(entry.rank);
        let reports_for_player = team_count;

        println!(
            "[DRY RUN] Would create {} scouting reports for: {} {} (rank {}, consensus grade {:.2})",
            reports_for_player, entry.first_name, entry.last_name, entry.rank, consensus_grade
        );

        stats.prospects_matched += 1;
        stats.reports_created += reports_for_player;
    }

    Ok(stats)
}

/// Maximum number of consecutive failures before aborting.
const MAX_CONSECUTIVE_FAILURES: usize = 10;

pub async fn load_scouting_reports(
    data: &RankingData,
    player_repo: &dyn PlayerRepository,
    team_repo: &dyn TeamRepository,
    pool: &sqlx::PgPool,
) -> Result<ScoutingReportLoadStats> {
    let mut stats = ScoutingReportLoadStats::default();
    let mut consecutive_failures: usize = 0;

    // Load all teams
    let teams = team_repo
        .find_all()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch teams: {}", e))?;

    if teams.is_empty() {
        anyhow::bail!("No teams found in database. Load teams first.");
    }

    stats.teams_used = teams.len();
    println!("Found {} teams", teams.len());

    // Load all players for the draft year and build lookup map
    let players = player_repo
        .find_by_draft_year(data.meta.draft_year)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch players: {}", e))?;

    println!(
        "Found {} players for draft year {}",
        players.len(),
        data.meta.draft_year
    );

    let player_map: HashMap<(String, String), &domain::models::Player> = players
        .iter()
        .map(|p| ((p.first_name.to_lowercase(), p.last_name.to_lowercase()), p))
        .collect();

    // Wrap the DELETE + INSERTs in a transaction so that if the process fails
    // mid-way, old reports are not lost and the database stays consistent.
    let mut tx = pool.begin().await?;

    // Clear existing scouting reports for players in this draft year
    println!(
        "Clearing existing scouting reports for draft year {}...",
        data.meta.draft_year
    );
    let deleted = sqlx::query(
        "DELETE FROM scouting_reports WHERE player_id IN (SELECT id FROM players WHERE draft_year = $1)"
    )
    .bind(data.meta.draft_year)
    .execute(&mut *tx)
    .await?;
    println!(
        "Cleared {} existing scouting reports",
        deleted.rows_affected()
    );

    // Process each ranked prospect
    for entry in &data.rankings {
        let lookup_key = (
            entry.first_name.to_lowercase(),
            entry.last_name.to_lowercase(),
        );

        let player = match player_map.get(&lookup_key) {
            Some(p) => *p,
            None => {
                let name = format!("{} {}", entry.first_name, entry.last_name);
                tracing::warn!("No player match for: {} (rank {})", name, entry.rank);
                stats.prospects_unmatched += 1;
                stats.unmatched_names.push(name);
                continue;
            }
        };

        let consensus_grade = rank_to_grade(entry.rank);
        let mut reports_created_for_player = 0;

        // Create a scouting report for each team
        for team in &teams {
            let team_grade = generate_team_grade(
                consensus_grade,
                &team.abbreviation,
                &entry.first_name,
                &entry.last_name,
            );
            let fit_grade =
                generate_fit_grade(&team.abbreviation, &entry.first_name, &entry.last_name);
            let (injury_concern, character_concern) =
                generate_concern_flags(&team.abbreviation, &entry.first_name, &entry.last_name);

            let report = match ScoutingReport::new(player.id, team.id, team_grade) {
                Ok(r) => r
                    .with_fit_grade(fit_grade)
                    .with_injury_concern(injury_concern)
                    .with_character_concern(character_concern),
                Err(e) => {
                    let msg = format!(
                        "Failed to create scouting report for {} {} / {}: {}",
                        entry.first_name, entry.last_name, team.abbreviation, e
                    );
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                    stats.reports_failed += 1;
                    continue;
                }
            };

            let fit_grade_str = report.fit_grade.map(|g| g.as_str().to_string());
            let insert_result = sqlx::query(
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
            .await;

            match insert_result {
                Ok(_) => {
                    reports_created_for_player += 1;
                }
                Err(e) => {
                    let msg = format!(
                        "Failed to insert scouting report for {} {} / {}: {}",
                        entry.first_name, entry.last_name, team.abbreviation, e
                    );
                    tracing::error!("{}", msg);
                    stats.errors.push(msg);
                    stats.reports_failed += 1;
                }
            }
        }

        if reports_created_for_player > 0 {
            stats.prospects_matched += 1;
            stats.reports_created += reports_created_for_player;
            consecutive_failures = 0;
            tracing::info!(
                "Created {} reports for {} {} (rank {}, grade {:.2})",
                reports_created_for_player,
                entry.first_name,
                entry.last_name,
                entry.rank,
                consensus_grade
            );
        } else {
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
        }
    }

    if !stats.errors.is_empty() {
        // Roll back so old reports are preserved rather than committing partial data.
        tx.rollback().await?;
        anyhow::bail!(
            "Rolling back transaction due to {} error(s). First: {}",
            stats.errors.len(),
            stats.errors[0]
        );
    }

    tx.commit().await?;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> &'static str {
        r#"{
            "meta": {
                "version": "1.0.0",
                "source": "template",
                "source_url": "N/A",
                "draft_year": 2026,
                "scraped_at": "2026-02-09",
                "total_prospects": 3
            },
            "rankings": [
                {
                    "rank": 1,
                    "first_name": "Fernando",
                    "last_name": "Mendoza",
                    "position": "QB",
                    "school": "Indiana"
                },
                {
                    "rank": 2,
                    "first_name": "Caleb",
                    "last_name": "Downs",
                    "position": "S",
                    "school": "Ohio State"
                },
                {
                    "rank": 3,
                    "first_name": "Mykel",
                    "last_name": "Williams",
                    "position": "EDGE",
                    "school": "Georgia"
                }
            ]
        }"#
    }

    #[test]
    fn test_parse_json() {
        let data: RankingData = serde_json::from_str(sample_json()).unwrap();
        assert_eq!(data.meta.draft_year, 2026);
        assert_eq!(data.meta.total_prospects, 3);
        assert_eq!(data.rankings.len(), 3);
        assert_eq!(data.rankings[0].first_name, "Fernando");
        assert_eq!(data.rankings[0].last_name, "Mendoza");
        assert_eq!(data.rankings[0].position, "QB");
        assert_eq!(data.rankings[0].rank, 1);
    }

    #[test]
    fn test_rank_to_grade_top_prospect() {
        let grade = rank_to_grade(1);
        assert!((grade - 9.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rank_to_grade_first_round() {
        let grade = rank_to_grade(32);
        // 9.5 - 31 * 0.04 = 9.5 - 1.24 = 8.26
        assert!((grade - 8.26).abs() < 0.01);
    }

    #[test]
    fn test_rank_to_grade_floor() {
        let grade = rank_to_grade(200);
        assert!((grade - 3.5).abs() < f64::EPSILON);

        let grade = rank_to_grade(500);
        assert!((grade - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rank_to_grade_zero_returns_floor() {
        let grade = rank_to_grade(0);
        assert!((grade - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rank_to_grade_negative_returns_floor() {
        let grade = rank_to_grade(-1);
        assert!((grade - 3.5).abs() < f64::EPSILON);

        let grade = rank_to_grade(-100);
        assert!((grade - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rank_to_grade_mid_range() {
        let grade = rank_to_grade(100);
        // 9.5 - 99 * 0.04 = 9.5 - 3.96 = 5.54
        assert!((grade - 5.54).abs() < 0.01);
    }

    #[test]
    fn test_generate_team_grade_deterministic() {
        let grade1 = generate_team_grade(8.0, "DAL", "John", "Smith");
        let grade2 = generate_team_grade(8.0, "DAL", "John", "Smith");
        assert!((grade1 - grade2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generate_team_grade_varies_by_team() {
        let grade_dal = generate_team_grade(8.0, "DAL", "John", "Smith");
        let grade_buf = generate_team_grade(8.0, "BUF", "John", "Smith");
        assert!(
            (grade_dal - grade_buf).abs() > f64::EPSILON,
            "Grades should differ: DAL={}, BUF={}",
            grade_dal,
            grade_buf
        );
    }

    #[test]
    fn test_generate_team_grade_within_bounds() {
        let grade_high = generate_team_grade(9.5, "DAL", "Test", "Player");
        assert!(grade_high >= 0.0 && grade_high <= 10.0);

        let grade_low = generate_team_grade(0.5, "DAL", "Test", "Player");
        assert!(grade_low >= 0.0 && grade_low <= 10.0);
    }

    #[test]
    fn test_generate_team_grade_range() {
        let teams = [
            "ARI", "ATL", "BAL", "BUF", "CAR", "CHI", "CIN", "CLE", "DAL", "DEN", "DET", "GB",
            "HOU", "IND", "JAX", "KC", "LAC", "LAR", "LV", "MIA", "MIN", "NE", "NO", "NYG", "NYJ",
            "PHI", "PIT", "SEA", "SF", "TB", "TEN", "WAS",
        ];

        let consensus = 8.0;
        let grades: Vec<f64> = teams
            .iter()
            .map(|t| generate_team_grade(consensus, t, "Test", "Player"))
            .collect();

        let min = grades.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = grades.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        assert!(
            min >= 7.2 - f64::EPSILON,
            "Min grade {} below expected",
            min
        );
        assert!(
            max <= 8.8 + f64::EPSILON,
            "Max grade {} above expected",
            max
        );
    }

    #[test]
    fn test_dry_run() {
        let data: RankingData = serde_json::from_str(sample_json()).unwrap();
        let stats = load_scouting_reports_dry_run(&data).unwrap();
        assert_eq!(stats.prospects_matched, 3);
        assert_eq!(stats.reports_created, 3 * 32);
        assert_eq!(stats.teams_used, 32);
        assert!(stats.errors.is_empty());
    }

    #[test]
    fn test_fit_grade_deterministic() {
        let grade1 = generate_fit_grade("DAL", "John", "Smith");
        let grade2 = generate_fit_grade("DAL", "John", "Smith");
        assert_eq!(grade1, grade2);
    }

    #[test]
    fn test_concern_flags_deterministic() {
        let (inj1, char1) = generate_concern_flags("DAL", "John", "Smith");
        let (inj2, char2) = generate_concern_flags("DAL", "John", "Smith");
        assert_eq!(inj1, inj2);
        assert_eq!(char1, char2);
    }
}
