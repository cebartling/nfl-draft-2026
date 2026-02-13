use crate::models::{RankingData, RankingEntry, RankingMeta};

/// Generate a template ranking file with 2026 draft class prospect data.
/// This serves as a fallback when scraping fails and provides a starting
/// point that can be edited manually with real prospect rankings.
///
/// Contains 184 prospects sourced from Tankathon and Walter Football
/// via the merge tool, deduplicated by normalized name.
pub fn generate_template(year: i32) -> RankingData {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Template prospects from the 2026 draft class (sourced from Tankathon
    // and Walter Football via merge). Ranked in approximate consensus order.
    let prospects = vec![
        ("Arvell", "Reese", "LB", "Ohio State"),
        ("Rueben", "Bain Jr.", "EDGE", "Miami"),
        ("Caleb", "Downs", "S", "Ohio State"),
        ("Fernando", "Mendoza", "QB", "Indiana"),
        ("David", "Bailey", "EDGE", "Texas Tech"),
        ("Francis", "Mauigoa", "OT", "Miami"),
        ("Carnell", "Tate", "WR", "Ohio State"),
        ("Spencer", "Fano", "OT", "Utah"),
        ("Jeremiyah", "Love", "RB", "Notre Dame"),
        ("Jordyn", "Tyson", "WR", "Arizona State"),
        ("Mansoor", "Delane", "CB", "LSU"),
        ("Makai", "Lemon", "WR", "USC"),
        ("Sonny", "Styles", "LB", "Ohio State"),
        ("Jermod", "McCoy", "CB", "Tennessee"),
        ("Keldric", "Faulk", "EDGE", "Auburn"),
        ("Peter", "Woods", "DT", "Clemson"),
        ("Kenyon", "Sadiq", "TE", "Oregon"),
        ("Vega", "Ioane", "OG", "Penn State"),
        ("Denzel", "Boston", "WR", "Washington"),
        ("Cashius", "Howell", "EDGE", "Texas A&M"),
        ("Avieon", "Terrell", "CB", "Clemson"),
        ("Kadyn", "Proctor", "OT", "Alabama"),
        ("Caleb", "Lomu", "OT", "Utah"),
        ("CJ", "Allen", "LB", "Georgia"),
        ("Kayden", "McDonald", "DT", "Ohio State"),
        ("KC", "Concepcion", "WR", "Texas A&M"),
        ("Ty", "Simpson", "QB", "Alabama"),
        ("TJ", "Parker", "EDGE", "Clemson"),
        ("Caleb", "Banks", "DT", "Florida"),
        ("Brandon", "Cisse", "CB", "South Carolina"),
        ("Akheem", "Mesidor", "EDGE", "Miami"),
        ("Monroe", "Freeling", "OT", "Georgia"),
        ("Colton", "Hood", "CB", "Tennessee"),
        ("Emmanuel", "McNeil-Warren", "S", "Toledo"),
        ("Anthony", "Hill Jr.", "LB", "Texas"),
        ("Emmanuel", "Pregnon", "OG", "Oregon"),
        ("Lee", "Hunter", "DT", "Texas Tech"),
        ("Dillon", "Thieneman", "S", "Oregon"),
        ("Blake", "Miller", "OT", "Clemson"),
        ("R. Mason", "Thomas", "EDGE", "Oklahoma"),
        ("Chris", "Bell", "WR", "Louisville"),
        ("Christen", "Miller", "DT", "Georgia"),
        ("Zion", "Young", "EDGE", "Missouri"),
        ("Gennings", "Dunker", "OT", "Iowa"),
        ("Chris", "Johnson", "CB", "San Diego State"),
        ("Zachariah", "Branch", "WR", "Georgia"),
        ("Keith", "Abney II", "CB", "Arizona State"),
        ("D'Angelo", "Ponds", "CB", "Indiana"),
        ("Germie", "Bernard", "WR", "Alabama"),
        ("Max", "Iheanachor", "OT", "Arizona State"),
        ("AJ", "Haulcy", "S", "LSU"),
        ("Jake", "Golday", "LB", "Cincinnati"),
        ("Chris", "Brazzell II", "WR", "Tennessee"),
        ("Caleb", "Tiernan", "OT", "Northwestern"),
        ("LT", "Overton", "EDGE", "Alabama"),
        ("Keionte", "Scott", "CB", "Miami"),
        ("Jadarian", "Price", "RB", "Notre Dame"),
        ("Kamari", "Ramsey", "S", "USC"),
        ("Elijah", "Sarratt", "WR", "Indiana"),
        ("Omar", "Cooper Jr.", "WR", "Indiana"),
        ("Connor", "Lew", "OG", "Auburn"),
        ("Chase", "Bisontis", "OG", "Texas A&M"),
        ("Gabe", "Jacas", "EDGE", "Illinois"),
        ("Max", "Klare", "TE", "Ohio State"),
        ("Deontae", "Lawson", "LB", "Alabama"),
        ("Joshua", "Josephs", "EDGE", "Tennessee"),
        ("Malik", "Muhammad", "CB", "Texas"),
        ("Isaiah", "World", "OT", "Oregon"),
        ("Josiah", "Trotter", "LB", "Missouri"),
        ("Domonique", "Orange", "DT", "Iowa State"),
        ("Ja'Kobi", "Lane", "WR", "USC"),
        ("Malachi", "Fields", "WR", "Notre Dame"),
        ("Jacob", "Rodriguez", "LB", "Texas Tech"),
        ("Antonio", "Williams", "WR", "Clemson"),
        ("Eli", "Stowers", "TE", "Vanderbilt"),
        ("Derrick", "Moore", "EDGE", "Michigan"),
        ("Romello", "Height", "EDGE", "Texas Tech"),
        ("Trinidad", "Chambliss", "QB", "Ole Miss"),
        ("Julian", "Neal", "CB", "Arkansas"),
        ("Dani", "Dennis-Sutton", "EDGE", "Penn State"),
        ("Darrell", "Jackson Jr.", "DT", "Florida State"),
        ("Jonah", "Coleman", "RB", "Washington"),
        ("Michael", "Trigg", "TE", "Baylor"),
        ("Jake", "Slaughter", "OG", "Florida"),
        ("Emmett", "Johnson", "RB", "Nebraska"),
        ("Davison", "Igbinosun", "CB", "Ohio State"),
        ("Anthony", "Lucas", "EDGE", "USC"),
        ("Genesis", "Smith", "S", "Arizona"),
        ("Chandler", "Rivers", "CB", "Duke"),
        ("Deion", "Burks", "WR", "Oklahoma"),
        ("Austin", "Barber", "OT", "Florida"),
        ("Carson", "Beck", "QB", "Miami"),
        ("Will", "Lee III", "CB", "Texas A&M"),
        ("Treydan", "Stukes", "CB", "Arizona"),
        ("Devin", "Moore", "CB", "Florida"),
        ("Taurean", "York", "LB", "Texas A&M"),
        ("Nicholas", "Singleton", "RB", "Penn State"),
        ("Jack", "Endries", "TE", "Texas"),
        ("Skyler", "Bell", "WR", "UConn"),
        ("Dontay", "Corleone", "DT", "Cincinnati"),
        ("Harold", "Perkins Jr.", "LB", "LSU"),
        ("Drew", "Shelton", "OT", "Penn State"),
        ("Kaytron", "Allen", "RB", "Penn State"),
        ("Zakee", "Wheatley", "S", "Penn State"),
        ("Malachi", "Lawrence", "EDGE", "UCF"),
        ("Chris", "McClellan", "DT", "Missouri"),
        ("Brian", "Parker II", "OT", "Duke"),
        ("Daylen", "Everette", "CB", "Georgia"),
        ("Ted", "Hurst", "WR", "Georgia State"),
        ("Lander", "Barton", "LB", "Utah"),
        ("Garrett", "Nussmeier", "QB", "LSU"),
        ("Jaishawn", "Barham", "LB", "Michigan"),
        ("Jude", "Bowry", "OT", "Boston College"),
        ("Gracen", "Halton", "DT", "Oklahoma"),
        ("Caden", "Curry", "EDGE", "Ohio State"),
        ("Jalon", "Kilgore", "S", "South Carolina"),
        ("Dametrious", "Crownover", "OT", "Texas A&M"),
        ("Mikail", "Kamara", "EDGE", "Indiana"),
        ("Bud", "Clark", "S", "TCU"),
        ("CJ", "Daniels", "WR", "Miami"),
        ("Sam", "Hecht", "OG", "Kansas State"),
        ("Justin", "Joly", "TE", "NC State"),
        ("Keylan", "Rutledge", "OG", "Georgia Tech"),
        ("Michael", "Taaffe", "S", "Texas"),
        ("Ar'maj", "Reed-Adams", "OG", "Texas A&M"),
        ("Kyle", "Louis", "LB", "Pittsburgh"),
        ("Zane", "Durant", "DT", "Penn State"),
        ("Tyreak", "Sapp", "EDGE", "Florida"),
        ("Eli", "Raridon", "TE", "Notre Dame"),
        ("Demond", "Claiborne", "RB", "Wake Forest"),
        ("Tim", "Keenan III", "DT", "Alabama"),
        ("Drew", "Allar", "QB", "Penn State"),
        ("Sam", "Roush", "TE", "Stanford"),
        ("Louis", "Moore", "S", "Indiana"),
        ("Fernando", "Carmona Jr.", "OT", "Arkansas"),
        ("Aamil", "Wagner", "OT", "Notre Dame"),
        ("Oscar", "Delp", "TE", "Georgia"),
        ("Xavier", "Scott", "CB", "Illinois"),
        ("Mike", "Washington Jr.", "RB", "Arkansas"),
        ("Tacario", "Davis", "CB", "Washington"),
        ("Joe", "Royer", "TE", "Cincinnati"),
        ("Skyler", "Gill-Howard", "DT", "Texas Tech"),
        ("Bryce", "Lance", "WR", "North Dakota State"),
        ("Parker", "Brailsford", "OG", "Alabama"),
        ("Logan", "Jones", "OG", "Iowa"),
        ("Cade", "Klubnik", "QB", "Clemson"),
        ("Beau", "Stephens", "OG", "Iowa"),
        ("Brenen", "Thompson", "WR", "Mississippi State"),
        ("Albert", "Regis", "DT", "Texas A&M"),
        ("Hezekiah", "Masses", "CB", "California"),
        ("JC", "Davis", "OT", "Illinois"),
        ("Kevin", "Coleman Jr.", "WR", "Missouri"),
        ("Domani", "Jackson", "CB", "Alabama"),
        ("DJ", "Campbell", "OG", "Texas"),
        ("Rayshaun", "Benny", "DT", "Michigan"),
        ("Dallen", "Bentley", "TE", "Utah"),
        ("Eric", "Rivers", "WR", "Georgia Tech"),
        ("Josh", "Cameron", "WR", "Baylor"),
        ("Zxavian", "Harris", "DT", "Ole Miss"),
        ("Jaeden", "Roberts", "OG", "Alabama"),
        ("Kage", "Casey", "OT", "Boise State"),
        ("Aaron", "Anderson", "WR", "LSU"),
        ("Aiden", "Fisher", "LB", "Indiana"),
        ("Eric", "McAlister", "WR", "TCU"),
        ("John Michael", "Gyllenborg", "TE", "Wyoming"),
        ("TJ", "Hall", "CB", "Iowa"),
        ("J'Mari", "Taylor", "RB", "Virginia"),
        ("DeMonte", "Capehart", "DT", "Clemson"),
        ("Sawyer", "Robertson", "QB", "Baylor"),
        ("Max", "Llewellyn", "EDGE", "Iowa"),
        ("Thaddeus", "Dixon", "CB", "North Carolina"),
        ("Marlin", "Klein", "TE", "Michigan"),
        ("Dae'Quan", "Wright", "TE", "Ole Miss"),
        ("Bishop", "Fitzgerald", "S", "USC"),
        ("Trey", "Moore", "EDGE", "Texas"),
        ("Trey", "Zuhn III", "OT", "Texas A&M"),
        ("Keyron", "Crawford", "EDGE", "Auburn"),
        ("Ola", "Ioane", "OG", "Penn State"),
        ("Jaylon", "Guilbeau", "CB", "Texas"),
        ("Diego", "Pounds", "OT", "Ole Miss"),
        ("Matthew", "Hibner", "TE", "SMU"),
        ("Nate", "Boerkircher", "TE", "Texas A&M"),
        ("Tanner", "Koziol", "TE", "Houston"),
        ("Earnest", "Greene III", "OT", "Georgia"),
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
        assert!(data.rankings.len() >= 180);
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
