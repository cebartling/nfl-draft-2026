use domain::models::{FitGrade, ScoutingReport};

/// FNV-1a hash for deterministic, Rust-version-stable hashing.
///
/// Unlike `DefaultHasher`, whose algorithm is explicitly not guaranteed to be
/// stable across Rust releases, FNV-1a is a fixed specification that will
/// produce identical output regardless of toolchain version.
pub fn fnv1a_hash(data: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001B3;

    let mut hash = FNV_OFFSET_BASIS;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Convert a ranking position (1-based) to a scouting grade (0.0–10.0 scale).
///
/// Piecewise-linear curve that is intentionally steep at the top so the
/// gap between elite and good prospects is wider than per-team scouting
/// noise. The previous shallow `9.5 - (rank-1) × 0.03` curve produced
/// only a 0.87-grade gap between rank 1 and rank 30, which was narrower
/// than `generate_team_grade`'s ±0.8 noise — letting mid-round need
/// matches flip past top talents in round 1.
///
/// Bands:
/// - Rank 1-10  : 9.9 → 9.3     (slope −0.0667/rank, "elite")
/// - Rank 10-30 : 9.3 → 8.3     (slope −0.05/rank,   "first round")
/// - Rank 30-100: 8.3 → 5.5     (slope −0.04/rank,   "day 1-2")
/// - Rank 100+  : 5.5 → 3.0 floor (slope −0.025/rank, "day 3")
/// - Rank ≤ 0 returns the 3.0 floor (treated as unranked).
pub fn rank_to_grade(rank: i32) -> f64 {
    if rank <= 0 {
        return 3.0;
    }
    let r = rank as f64;
    let grade = if rank <= 10 {
        9.9 - (r - 1.0) * 0.0667
    } else if rank <= 30 {
        9.3 - (r - 10.0) * 0.05
    } else if rank <= 100 {
        8.3 - (r - 30.0) * 0.04
    } else {
        5.5 - (r - 100.0) * 0.025
    };
    grade.max(3.0)
}

/// Per-team scouting noise half-range selector (consensus grade → max offset).
///
/// Elite prospects have more public consensus, so different teams' grades
/// should cluster tighter. Mid-round and below deserves wider disagreement.
/// Returned value is the ± half-range used by `generate_team_grade`.
fn team_grade_variance(consensus_grade: f64) -> f64 {
    if consensus_grade >= 9.0 {
        0.3
    } else if consensus_grade >= 7.5 {
        0.6
    } else {
        0.8
    }
}

/// Generate a deterministic team-specific grade variation from a consensus grade.
///
/// Uses FNV-1a hash of the team abbreviation + player name to produce a
/// deterministic offset in a symmetric range around 0 whose half-width
/// depends on the consensus grade (see `team_grade_variance`):
/// - consensus ≥ 9.0 : ±0.3 (elite; top ~10-16)
/// - consensus ≥ 7.5 : ±0.6 (first/second round; top ~50)
/// - otherwise       : ±0.8 (mid-to-late)
///
/// The result is clamped to `[0.0, 10.0]`.
pub fn generate_team_grade(consensus_grade: f64, team_abbr: &str, first: &str, last: &str) -> f64 {
    let key = format!("{}-{}-{}", team_abbr, first, last);
    let hash = fnv1a_hash(key.as_bytes());

    let max_offset = team_grade_variance(consensus_grade);
    // Hash bucket in [0, 2000] → fraction in [0.0, 1.0) → scaled to [-max, +max].
    let frac = (hash % 2001) as f64 / 2000.0;
    let offset = (frac * 2.0 - 1.0) * max_offset;

    (consensus_grade + offset).clamp(0.0, 10.0)
}

/// Generate a deterministic fit grade for a team-player combination.
///
/// 70% chance of B (consensus), 15% chance of A (bump up), 15% chance of C (bump down).
pub fn generate_fit_grade(team_abbr: &str, first: &str, last: &str) -> FitGrade {
    let key = format!("fit-{}-{}-{}", team_abbr, first, last);
    let hash = fnv1a_hash(key.as_bytes());

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
pub fn generate_concern_flags(team_abbr: &str, first: &str, last: &str) -> (bool, bool) {
    let injury_key = format!("injury-{}-{}-{}", team_abbr, first, last);
    let injury_hash = fnv1a_hash(injury_key.as_bytes());

    let character_key = format!("character-{}-{}-{}", team_abbr, first, last);
    let character_hash = fnv1a_hash(character_key.as_bytes());

    (injury_hash % 100 < 5, character_hash % 100 < 5)
}

/// Create a scouting report for a player-team combination using deterministic generation.
pub fn create_scouting_report(
    player_id: uuid::Uuid,
    team_id: uuid::Uuid,
    team_abbr: &str,
    first_name: &str,
    last_name: &str,
    rank: i32,
) -> Result<ScoutingReport, domain::errors::DomainError> {
    let consensus_grade = rank_to_grade(rank);
    create_scouting_report_with_grade(
        player_id,
        team_id,
        team_abbr,
        first_name,
        last_name,
        consensus_grade,
    )
}

/// Create a scouting report from a caller-supplied consensus grade.
///
/// Used when the consensus signal comes from a source other than a numeric
/// rank — for example, Brugler's grade tiers on Beast prospects who fall
/// outside his top-100 numeric list.
pub fn create_scouting_report_with_grade(
    player_id: uuid::Uuid,
    team_id: uuid::Uuid,
    team_abbr: &str,
    first_name: &str,
    last_name: &str,
    consensus_grade: f64,
) -> Result<ScoutingReport, domain::errors::DomainError> {
    let team_grade = generate_team_grade(consensus_grade, team_abbr, first_name, last_name);
    let fit_grade = generate_fit_grade(team_abbr, first_name, last_name);
    let (injury_concern, character_concern) =
        generate_concern_flags(team_abbr, first_name, last_name);

    Ok(ScoutingReport::new(player_id, team_id, team_grade)?
        .with_fit_grade(fit_grade)
        .with_injury_concern(injury_concern)
        .with_character_concern(character_concern))
}

/// Map a Brugler "Beast" grade tier string to a 0-10 consensus grade.
///
/// Tiers roughly track where a prospect is expected to be drafted. The
/// resulting grade is on the same scale as `rank_to_grade`, so it can be
/// fed into `generate_team_grade` to produce a team-specific variation.
///
/// Unknown / missing tiers return a neutral 3.0 floor (same as an unranked
/// prospect) so we still produce a usable report rather than silently
/// dropping the player from the draft pool.
pub fn grade_tier_to_consensus_grade(tier: &str) -> f64 {
    let normalized = tier.trim().to_lowercase();
    if normalized.starts_with("1st") {
        8.8
    } else if normalized.starts_with("2nd") {
        7.8
    } else if normalized.starts_with("3rd") {
        6.8
    } else if normalized.starts_with("4th") {
        5.8
    } else if normalized.starts_with("5th") {
        5.0
    } else if normalized.starts_with("6th") {
        4.2
    } else if normalized.starts_with("7th") {
        3.5
    } else {
        3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_to_grade_top_prospect() {
        let grade = rank_to_grade(1);
        assert!((grade - 9.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rank_to_grade_elite_band_end() {
        // Rank 10 is the boundary of the elite band → 9.3
        let grade = rank_to_grade(10);
        assert!(
            (grade - 9.3).abs() < 0.01,
            "rank 10 should be ~9.3, got {}",
            grade
        );
    }

    #[test]
    fn test_rank_to_grade_first_round() {
        // Rank 32 sits in the 30-100 band: 8.3 - (32-30) × 0.04 = 8.22
        let grade = rank_to_grade(32);
        assert!(
            (grade - 8.22).abs() < 0.01,
            "rank 32 should be ~8.22, got {}",
            grade
        );
    }

    #[test]
    fn test_rank_to_grade_gap_top_vs_rank_30_exceeds_old_noise() {
        // The whole point of the new curve: the top-prospect-to-rank-30
        // gap must exceed the old ±0.8 team noise, so a mid-round need
        // match can't flip past a top talent in round 1.
        let gap = rank_to_grade(1) - rank_to_grade(30);
        assert!(
            gap > 1.5,
            "gap rank 1 vs rank 30 must exceed 1.5 (old was 0.87), got {}",
            gap
        );
    }

    #[test]
    fn test_rank_to_grade_monotonically_decreasing() {
        // Higher rank (worse prospect) should never produce a higher grade.
        let mut prev = rank_to_grade(1);
        for r in 2..=250 {
            let cur = rank_to_grade(r);
            assert!(
                cur <= prev + f64::EPSILON,
                "non-monotonic: rank {} = {}, rank {} = {}",
                r - 1,
                prev,
                r,
                cur
            );
            prev = cur;
        }
    }

    #[test]
    fn test_rank_to_grade_floor() {
        let grade = rank_to_grade(250);
        assert!((grade - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rank_to_grade_zero_returns_floor() {
        let grade = rank_to_grade(0);
        assert!((grade - 3.0).abs() < f64::EPSILON);
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
    fn test_team_grade_variance_elite_tighter_than_mid() {
        // Sweep all 32 NFL teams for the same player at elite and mid grades
        // and verify that elite grades cluster tighter than mid grades.
        let teams = [
            "ARI", "ATL", "BAL", "BUF", "CAR", "CHI", "CIN", "CLE", "DAL", "DEN", "DET", "GB",
            "HOU", "IND", "JAX", "KC", "LAC", "LAR", "LV", "MIA", "MIN", "NE", "NO", "NYG", "NYJ",
            "PHI", "PIT", "SEA", "SF", "TB", "TEN", "WAS",
        ];
        let spread = |consensus: f64| -> f64 {
            let grades: Vec<f64> = teams
                .iter()
                .map(|t| generate_team_grade(consensus, t, "Test", "Player"))
                .collect();
            let min = grades.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = grades.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            max - min
        };
        let elite_spread = spread(9.5);
        let mid_spread = spread(6.0);
        assert!(
            elite_spread < mid_spread,
            "elite spread {} should be less than mid spread {}",
            elite_spread,
            mid_spread
        );
        // Elite variance is ±0.3 → max possible spread is 0.6
        assert!(
            elite_spread <= 0.6 + 1e-9,
            "elite spread {} exceeds ±0.3 bound",
            elite_spread
        );
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

    #[test]
    fn test_grade_tier_to_consensus_grade_known_tiers() {
        assert!((grade_tier_to_consensus_grade("1st round") - 8.8).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("2nd round") - 7.8).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("3rd round") - 6.8).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("4th-5th") - 5.8).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("4th round") - 5.8).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("5th-6th") - 5.0).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("6th-7th") - 4.2).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("7th-FA") - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_grade_tier_to_consensus_grade_case_and_whitespace_insensitive() {
        assert!((grade_tier_to_consensus_grade("1ST ROUND") - 8.8).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("  2nd Round  ") - 7.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_grade_tier_to_consensus_grade_unknown_returns_floor() {
        assert!((grade_tier_to_consensus_grade("") - 3.0).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("FA") - 3.0).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("Free Agent") - 3.0).abs() < f64::EPSILON);
        assert!((grade_tier_to_consensus_grade("UDFA") - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_grade_tier_ordering_preserved() {
        // Tiers must be monotonically non-increasing from 1st down to unknown
        let ordered = [
            "1st round",
            "2nd round",
            "3rd round",
            "4th-5th",
            "5th-6th",
            "6th-7th",
            "7th-FA",
            "",
        ];
        let grades: Vec<f64> = ordered
            .iter()
            .map(|t| grade_tier_to_consensus_grade(t))
            .collect();
        for pair in grades.windows(2) {
            assert!(
                pair[0] > pair[1],
                "Tiers not strictly descending: {:?}",
                grades
            );
        }
    }

    #[test]
    fn test_create_scouting_report_with_grade_matches_rank_path() {
        let player = uuid::Uuid::new_v4();
        let team = uuid::Uuid::new_v4();
        // Compute rank_to_grade once and feed both paths the same consensus grade
        // so the test stays correct if the curve is retuned again.
        let consensus = rank_to_grade(32);
        let via_rank = create_scouting_report(player, team, "DAL", "Test", "Player", 32).unwrap();
        let via_grade =
            create_scouting_report_with_grade(player, team, "DAL", "Test", "Player", consensus)
                .unwrap();
        assert!((via_rank.grade - via_grade.grade).abs() < 1e-9);
    }
}
