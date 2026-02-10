use crate::models::{RankingData, RankingEntry, RankingMeta};

/// Generate a template ranking file with realistic prospect data.
/// This serves as a fallback when scraping fails and provides a starting
/// point that can be edited manually with real prospect data.
pub fn generate_template(year: i32) -> RankingData {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Template prospects based on commonly projected top 2026 draft prospects
    let prospects = vec![
        ("Shedeur", "Sanders", "QB", "Colorado"),
        ("Cam", "Ward", "QB", "Miami"),
        ("Travis", "Hunter", "CB", "Colorado"),
        ("Tetairoa", "McMillan", "WR", "Oregon"),
        ("Abdul", "Carter", "EDGE", "Penn State"),
        ("Kelvin", "Banks Jr.", "OT", "Texas"),
        ("Mason", "Graham", "DT", "Michigan"),
        ("Luther", "Burden III", "WR", "Missouri"),
        ("Malaki", "Starks", "S", "Georgia"),
        ("Will", "Campbell", "OT", "LSU"),
        ("Colston", "Loveland", "TE", "Michigan"),
        ("Kenneth", "Grant", "DT", "Michigan"),
        ("Jalon", "Walker", "LB", "Georgia"),
        ("Nic", "Scourton", "EDGE", "Texas A&M"),
        ("Will", "Johnson", "CB", "Michigan"),
        ("Mykel", "Williams", "EDGE", "Georgia"),
        ("James", "Pearce Jr.", "EDGE", "Tennessee"),
        ("Tyler", "Warren", "TE", "Penn State"),
        ("Emeka", "Egbuka", "WR", "Ohio State"),
        ("Ashton", "Jeanty", "RB", "Boise State"),
        ("Derrick", "Harmon", "DT", "Oregon"),
        ("Benjamin", "Morrison", "CB", "Notre Dame"),
        ("Josh", "Simmons", "OT", "Ohio State"),
        ("Donovan", "Jackson", "OG", "Ohio State"),
        ("Ty", "Robinson", "DT", "Nebraska"),
        ("Shavon", "Revel Jr.", "CB", "East Carolina"),
        ("Nick", "Singleton", "RB", "Penn State"),
        ("Tyleik", "Williams", "DT", "Ohio State"),
        ("Princely", "Umanmielen", "EDGE", "Ole Miss"),
        ("Matthew", "Golden", "WR", "Texas"),
        ("Trey", "Amos", "CB", "Ole Miss"),
        ("Grey", "Zabel", "OT", "North Dakota State"),
        ("Wyatt", "Milum", "OG", "West Virginia"),
        ("Kyle", "Kennard", "EDGE", "South Carolina"),
        ("Mike", "Matthews", "C", "Ohio State"),
        ("Walter", "Nolen", "DT", "Ole Miss"),
        ("Kevin", "Winston Jr.", "S", "Penn State"),
        ("Tate", "Ratledge", "OG", "Georgia"),
        ("Deone", "Walker", "DT", "Kentucky"),
        ("Andrew", "Mukuba", "S", "Texas"),
        ("Harold", "Fannin Jr.", "TE", "Bowling Green"),
        ("Jack", "Sawyer", "EDGE", "Ohio State"),
        ("Jihaad", "Campbell", "LB", "Alabama"),
        ("Smael", "Mondon Jr.", "LB", "Georgia"),
        ("Tyler", "Booker", "OG", "Alabama"),
        ("Quinshon", "Judkins", "RB", "Ohio State"),
        ("Omarion", "Hampton", "RB", "North Carolina"),
        ("Landon", "Jackson", "EDGE", "Arkansas"),
        ("Jalen", "Milroe", "QB", "Alabama"),
        ("Quinn", "Ewers", "QB", "Texas"),
    ];

    let rankings: Vec<RankingEntry> = prospects
        .iter()
        .enumerate()
        .map(|(i, (first, last, pos, school))| RankingEntry {
            rank: (i + 1) as i32,
            first_name: first.to_string(),
            last_name: last.to_string(),
            position: pos.to_string(),
            school: school.to_string(),
        })
        .collect();

    let total = rankings.len();

    RankingData {
        meta: RankingMeta {
            version: "1.0.0".to_string(),
            source: "template".to_string(),
            source_url: "N/A".to_string(),
            draft_year: year,
            scraped_at: today,
            total_prospects: total,
        },
        rankings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_template_has_prospects() {
        let data = generate_template(2026);
        assert!(!data.rankings.is_empty());
        assert!(data.rankings.len() >= 50);
        assert_eq!(data.meta.draft_year, 2026);
        assert_eq!(data.meta.source, "template");
        assert_eq!(data.meta.total_prospects, data.rankings.len());
    }

    #[test]
    fn test_template_ranks_are_sequential() {
        let data = generate_template(2026);
        for (i, entry) in data.rankings.iter().enumerate() {
            assert_eq!(entry.rank, (i + 1) as i32);
        }
    }

    #[test]
    fn test_template_has_variety_of_positions() {
        let data = generate_template(2026);
        let positions: std::collections::HashSet<_> =
            data.rankings.iter().map(|e| e.position.clone()).collect();

        assert!(positions.contains("QB"));
        assert!(positions.contains("WR"));
        assert!(positions.contains("CB"));
        assert!(positions.contains("OT"));
        assert!(positions.contains("DT"));
        assert!(positions.contains("EDGE"));
    }

    #[test]
    fn test_template_all_fields_non_empty() {
        let data = generate_template(2026);
        for entry in &data.rankings {
            assert!(!entry.first_name.is_empty());
            assert!(!entry.last_name.is_empty());
            assert!(!entry.position.is_empty());
            assert!(!entry.school.is_empty());
        }
    }
}
