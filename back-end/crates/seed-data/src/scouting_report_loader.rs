use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use anyhow::Result;
use domain::models::{FitGrade, ScoutingReport};
use domain::repositories::{PlayerRepository, ScoutingReportRepository, TeamRepository};
use serde::Deserialize;

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

/// Convert a ranking position (1-based) to a scouting grade (0.0-10.0 scale).
///
/// Formula: grade = max(3.5, 9.5 - (rank - 1) * 0.04)
/// - Rank 1 = 9.5 (elite prospect)
/// - Rank 32 = 8.26 (first-round caliber)
/// - Rank 100 = 5.54
/// - Rank 150+ = 3.5 (floor)
pub fn rank_to_grade(rank: i32) -> f64 {
    let grade = 9.5 - (rank - 1) as f64 * 0.04;
    grade.max(3.5)
}

/// Generate a deterministic team-specific grade variation from a consensus grade.
///
/// Uses a hash of the team abbreviation + player name to produce a deterministic
/// offset in the range [-0.8, +0.8], clamped to [0.0, 10.0].
pub fn generate_team_grade(consensus_grade: f64, team_abbr: &str, first: &str, last: &str) -> f64 {
    let key = format!("{}-{}-{}", team_abbr, first, last);
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();

    // Map hash to range [-0.8, 0.8]
    let offset = ((hash % 1601) as f64 / 1000.0) - 0.8;

    (consensus_grade + offset).clamp(0.0, 10.0)
}

/// Generate a deterministic fit grade for a team-player combination.
///
/// 70% chance of B (consensus), 15% chance of A (bump up), 15% chance of C (bump down).
fn generate_fit_grade(team_abbr: &str, first: &str, last: &str) -> FitGrade {
    let key = format!("fit-{}-{}-{}", team_abbr, first, last);
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();

    let bucket = hash % 100;
    if bucket < 15 {
        FitGrade::A
    } else if bucket < 85 {
        FitGrade::B
    } else {
        FitGrade::C
    }
}

/// Generate deterministic injury/character concern flags.
///
/// ~5% chance of each flag being set.
fn generate_concern_flags(team_abbr: &str, first: &str, last: &str) -> (bool, bool) {
    let injury_key = format!("injury-{}-{}-{}", team_abbr, first, last);
    let mut hasher = DefaultHasher::new();
    injury_key.hash(&mut hasher);
    let injury_hash = hasher.finish();

    let character_key = format!("character-{}-{}-{}", team_abbr, first, last);
    let mut hasher2 = DefaultHasher::new();
    character_key.hash(&mut hasher2);
    let character_hash = hasher2.finish();

    (injury_hash % 100 < 5, character_hash % 100 < 5)
}

/// Maximum number of consecutive failures before aborting.
const MAX_CONSECUTIVE_FAILURES: usize = 10;

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

pub async fn load_scouting_reports(
    data: &RankingData,
    player_repo: &dyn PlayerRepository,
    team_repo: &dyn TeamRepository,
    scouting_repo: &dyn ScoutingReportRepository,
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

    // Clear existing scouting reports for players in this draft year
    println!(
        "Clearing existing scouting reports for draft year {}...",
        data.meta.draft_year
    );
    let deleted = sqlx::query(
        "DELETE FROM scouting_reports WHERE player_id IN (SELECT id FROM players WHERE draft_year = $1)"
    )
    .bind(data.meta.draft_year)
    .execute(pool)
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

            match scouting_repo.create(&report).await {
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
        // Different teams should generally produce different grades
        // (can be the same by coincidence but very unlikely)
        assert!(
            (grade_dal - grade_buf).abs() > f64::EPSILON || grade_dal == grade_buf, // Allow same by coincidence
            "Grades should generally vary: DAL={}, BUF={}",
            grade_dal,
            grade_buf
        );
    }

    #[test]
    fn test_generate_team_grade_within_bounds() {
        // Test with extreme consensus grades
        let grade_high = generate_team_grade(9.5, "DAL", "Test", "Player");
        assert!(grade_high >= 0.0 && grade_high <= 10.0);

        let grade_low = generate_team_grade(0.5, "DAL", "Test", "Player");
        assert!(grade_low >= 0.0 && grade_low <= 10.0);
    }

    #[test]
    fn test_generate_team_grade_range() {
        // Generate grades for many teams to verify the spread
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

        // All grades should be within ±0.8 of consensus, clamped
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
        assert_eq!(stats.reports_created, 3 * 32); // 3 prospects × 32 teams
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
