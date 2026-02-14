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

/// Convert a ranking position (1-based) to a scouting grade (0.0-10.0 scale).
///
/// Formula: grade = max(3.0, 9.5 - (rank - 1) * 0.03)
/// - Rank 1 = 9.50 (elite prospect)
/// - Rank 32 = 8.57 (first-round caliber)
/// - Rank 100 = 6.53
/// - Rank 200 = 3.53
/// - Rank 218+ = 3.0 (floor)
/// - Rank <= 0 is treated as "unranked" and assigned the floor grade (3.0)
pub fn rank_to_grade(rank: i32) -> f64 {
    if rank <= 0 {
        return 3.0;
    }
    let grade = 9.5 - (rank - 1) as f64 * 0.03;
    grade.max(3.0)
}

/// Generate a deterministic team-specific grade variation from a consensus grade.
///
/// Uses FNV-1a hash of the team abbreviation + player name to produce a
/// deterministic offset in the range [-0.8, +0.8], clamped to [0.0, 10.0].
pub fn generate_team_grade(consensus_grade: f64, team_abbr: &str, first: &str, last: &str) -> f64 {
    let key = format!("{}-{}-{}", team_abbr, first, last);
    let hash = fnv1a_hash(key.as_bytes());

    // Map hash to range [-0.8, 0.8]
    let offset = ((hash % 1601) as f64 / 1000.0) - 0.8;

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
    let team_grade = generate_team_grade(consensus_grade, team_abbr, first_name, last_name);
    let fit_grade = generate_fit_grade(team_abbr, first_name, last_name);
    let (injury_concern, character_concern) =
        generate_concern_flags(team_abbr, first_name, last_name);

    Ok(ScoutingReport::new(player_id, team_id, team_grade)?
        .with_fit_grade(fit_grade)
        .with_injury_concern(injury_concern)
        .with_character_concern(character_concern))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_to_grade_top_prospect() {
        let grade = rank_to_grade(1);
        assert!((grade - 9.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rank_to_grade_first_round() {
        let grade = rank_to_grade(32);
        // 9.5 - 31 * 0.03 = 9.5 - 0.93 = 8.57
        assert!((grade - 8.57).abs() < 0.01);
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
