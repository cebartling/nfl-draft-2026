use crate::models::{RankingData, RankingEntry, RankingMeta};

/// Generate a template ranking file with 2026 draft class prospect data.
/// This serves as a fallback when scraping fails and provides a starting
/// point that can be edited manually with real prospect rankings.
pub fn generate_template(year: i32) -> RankingData {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Template prospects from the 2026 draft class (sourced from database)
    // NOTE: Order is NOT an actual big-board ranking — edit manually to
    // reflect consensus rankings from a credible source.
    let prospects = vec![
        ("Carson", "Beck", "QB", "University of Miami"),
        ("Fernando", "Mendoza", "QB", "Indiana University"),
        ("Ty", "Simpson", "QB", "University of Alabama"),
        ("Diego", "Pavia", "QB", "Vanderbilt University"),
        ("Cole", "Payton", "QB", "North Dakota State University"),
        ("Terion", "Stewart", "RB", "Virginia Tech"),
        ("Desmond", "Reid", "RB", "University of Pittsburgh"),
        ("Le'Veon", "Moss", "RB", "Texas A&M University"),
        ("Nicholas", "Singleton", "RB", "Penn State University"),
        ("Chase", "Roberts", "WR", "Brigham Young University"),
        ("Keelan", "Marion", "WR", "University of Miami"),
        ("Carnell", "Tate", "WR", "Ohio State University"),
        ("Bryce", "Lance", "WR", "North Dakota State University"),
        ("Jeff", "Caldwell", "WR", "University of Cincinnati"),
        ("Cyrus", "Allen", "WR", "University of Cincinnati"),
        ("Brenen", "Thompson", "WR", "Mississippi State University"),
        ("Josh", "Cuevas", "TE", "University of Alabama"),
        ("Oscar", "Delp", "TE", "University of Georgia"),
        ("Tanner", "Koziol", "TE", "University of Houston"),
        ("Kevin", "Cline", "OT", "Boston College"),
        ("Caleb", "Tiernan", "OT", "Northwestern University"),
        ("Chris", "Adams", "OT", "University of Memphis"),
        ("Carver", "Willis", "OT", "University of Washington"),
        ("Rasheed", "Miller", "OT", "University of Louisville"),
        ("Kam", "Dewberry", "OG", "University of Alabama"),
        ("Micah", "Morris", "OG", "University of Georgia"),
        ("Matt", "Gulbin", "OG", "Michigan State University"),
        ("Francis", "Mauigoa", "OG", "University of Miami"),
        ("Bryce", "Foster", "C", "University of Kansas"),
        ("Sam", "Hecht", "C", "Kansas State University"),
        ("Jack", "Pyburn", "EDGE", "Louisiana State University"),
        ("Bryan", "Thomas Jr.", "EDGE", "University of South Carolina"),
        ("Ethan", "Burke", "EDGE", "University of Texas"),
        ("Malachi", "Lawrence", "EDGE", "University of Central Florida"),
        ("Keyron", "Crawford", "EDGE", "Auburn University"),
        ("Tim", "Keenan III", "DT", "University of Alabama"),
        ("Deven", "Eastern", "DT", "University of Minnesota"),
        ("Kody", "Huisman", "DT", "Virginia Tech"),
        ("DeMonte", "Capehart", "DT", "Clemson University"),
        ("Red", "Murdock", "LB", "University at Buffalo"),
        ("Shad", "Banks Jr.", "LB", "University of Texas at San Antonio"),
        ("Arvell", "Reese", "LB", "Ohio State University"),
        ("Mohamed", "Toure", "LB", "University of Miami"),
        ("Ephesians", "Prysock", "CB", "University of Washington"),
        ("Fred", "Davis II", "CB", "Northwestern University"),
        ("D'Angelo", "Ponds", "CB", "Indiana University"),
        ("Treydan", "Stukes", "CB", "University of Arizona"),
        ("Caleb", "Downs", "S", "Ohio State University"),
        ("DeShon", "Singleton", "S", "University of Nebraska"),
        ("Malik", "Spencer", "S", "Michigan State University"),
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
