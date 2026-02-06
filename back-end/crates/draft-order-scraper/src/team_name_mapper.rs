use std::collections::HashMap;

/// Maps Tankathon display names (lowercased) to NFL team abbreviations matching teams_nfl.json
pub fn build_team_name_map() -> HashMap<String, &'static str> {
    let mut map = HashMap::new();

    let entries: &[(&str, &str)] = &[
        ("arizona", "ARI"),
        ("atlanta", "ATL"),
        ("baltimore", "BAL"),
        ("buffalo", "BUF"),
        ("carolina", "CAR"),
        ("chicago", "CHI"),
        ("cincinnati", "CIN"),
        ("cleveland", "CLE"),
        ("dallas", "DAL"),
        ("denver", "DEN"),
        ("detroit", "DET"),
        ("green bay", "GB"),
        ("houston", "HOU"),
        ("indianapolis", "IND"),
        ("jacksonville", "JAX"),
        ("kansas city", "KC"),
        ("las vegas", "LV"),
        ("la chargers", "LAC"),
        ("la rams", "LAR"),
        ("los angeles chargers", "LAC"),
        ("los angeles rams", "LAR"),
        ("miami", "MIA"),
        ("minnesota", "MIN"),
        ("new england", "NE"),
        ("new orleans", "NO"),
        ("ny giants", "NYG"),
        ("ny jets", "NYJ"),
        ("new york giants", "NYG"),
        ("new york jets", "NYJ"),
        ("philadelphia", "PHI"),
        ("pittsburgh", "PIT"),
        ("san francisco", "SF"),
        ("seattle", "SEA"),
        ("tampa bay", "TB"),
        ("tennessee", "TEN"),
        ("washington", "WAS"),
    ];

    for &(name, abbr) in entries {
        map.insert(name.to_string(), abbr);
    }

    map
}

/// Resolve a display name to an NFL abbreviation.
/// Uses case-insensitive matching. Tries exact match first, then strips parentheticals.
pub fn resolve_team_abbreviation(display_name: &str) -> Option<&'static str> {
    let map = build_team_name_map();
    let normalized = display_name.trim().to_lowercase();

    if let Some(&abbr) = map.get(&normalized) {
        return Some(abbr);
    }

    // Try matching just the first word(s) - some Tankathon entries have extra text
    // e.g., "Green Bay (from DAL)" -> strip the parenthetical
    let without_parens = if let Some(idx) = normalized.find('(') {
        normalized[..idx].trim().to_string()
    } else {
        normalized.clone()
    };

    if let Some(&abbr) = map.get(&without_parens) {
        return Some(abbr);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_names() {
        assert_eq!(resolve_team_abbreviation("Dallas"), Some("DAL"));
        assert_eq!(resolve_team_abbreviation("Green Bay"), Some("GB"));
        assert_eq!(resolve_team_abbreviation("NY Jets"), Some("NYJ"));
        assert_eq!(resolve_team_abbreviation("NY Giants"), Some("NYG"));
        assert_eq!(resolve_team_abbreviation("LA Chargers"), Some("LAC"));
        assert_eq!(resolve_team_abbreviation("LA Rams"), Some("LAR"));
        assert_eq!(resolve_team_abbreviation("Las Vegas"), Some("LV"));
        assert_eq!(resolve_team_abbreviation("Kansas City"), Some("KC"));
        assert_eq!(resolve_team_abbreviation("San Francisco"), Some("SF"));
        assert_eq!(resolve_team_abbreviation("Tampa Bay"), Some("TB"));
        assert_eq!(resolve_team_abbreviation("New England"), Some("NE"));
        assert_eq!(resolve_team_abbreviation("New Orleans"), Some("NO"));
    }

    #[test]
    fn test_with_parenthetical() {
        assert_eq!(resolve_team_abbreviation("Dallas (from GB)"), Some("DAL"));
    }

    #[test]
    fn test_trimming() {
        assert_eq!(resolve_team_abbreviation("  Dallas  "), Some("DAL"));
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(resolve_team_abbreviation("dallas"), Some("DAL"));
        assert_eq!(resolve_team_abbreviation("DALLAS"), Some("DAL"));
        assert_eq!(resolve_team_abbreviation("Dallas"), Some("DAL"));
        assert_eq!(resolve_team_abbreviation("GREEN BAY"), Some("GB"));
        assert_eq!(resolve_team_abbreviation("la chargers"), Some("LAC"));
    }

    #[test]
    fn test_unknown() {
        assert_eq!(resolve_team_abbreviation("Unknown Team"), None);
    }

    #[test]
    fn test_all_32_teams_covered() {
        let expected = vec![
            "ARI", "ATL", "BAL", "BUF", "CAR", "CHI", "CIN", "CLE", "DAL", "DEN", "DET", "GB",
            "HOU", "IND", "JAX", "KC", "LAC", "LAR", "LV", "MIA", "MIN", "NE", "NO", "NYG", "NYJ",
            "PHI", "PIT", "SEA", "SF", "TB", "TEN", "WAS",
        ];
        let map = build_team_name_map();
        let values: std::collections::HashSet<&&str> = map.values().collect();
        for abbr in &expected {
            assert!(values.contains(&abbr), "Missing abbreviation: {}", abbr);
        }
    }
}
