use std::collections::HashMap;

/// Maps Tankathon display names to NFL team abbreviations matching teams_nfl.json
pub fn build_team_name_map() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();

    // Full city/region names as displayed on Tankathon
    map.insert("Arizona", "ARI");
    map.insert("Atlanta", "ATL");
    map.insert("Baltimore", "BAL");
    map.insert("Buffalo", "BUF");
    map.insert("Carolina", "CAR");
    map.insert("Chicago", "CHI");
    map.insert("Cincinnati", "CIN");
    map.insert("Cleveland", "CLE");
    map.insert("Dallas", "DAL");
    map.insert("Denver", "DEN");
    map.insert("Detroit", "DET");
    map.insert("Green Bay", "GB");
    map.insert("Houston", "HOU");
    map.insert("Indianapolis", "IND");
    map.insert("Jacksonville", "JAX");
    map.insert("Kansas City", "KC");
    map.insert("Las Vegas", "LV");
    map.insert("LA Chargers", "LAC");
    map.insert("LA Rams", "LAR");
    map.insert("Los Angeles Chargers", "LAC");
    map.insert("Los Angeles Rams", "LAR");
    map.insert("Miami", "MIA");
    map.insert("Minnesota", "MIN");
    map.insert("New England", "NE");
    map.insert("New Orleans", "NO");
    map.insert("NY Giants", "NYG");
    map.insert("NY Jets", "NYJ");
    map.insert("New York Giants", "NYG");
    map.insert("New York Jets", "NYJ");
    map.insert("Philadelphia", "PHI");
    map.insert("Pittsburgh", "PIT");
    map.insert("San Francisco", "SF");
    map.insert("Seattle", "SEA");
    map.insert("Tampa Bay", "TB");
    map.insert("Tennessee", "TEN");
    map.insert("Washington", "WAS");

    map
}

/// Resolve a display name to an NFL abbreviation.
/// Tries exact match first, then tries trimming whitespace.
pub fn resolve_team_abbreviation(display_name: &str) -> Option<&'static str> {
    let map = build_team_name_map();
    let trimmed = display_name.trim();

    if let Some(&abbr) = map.get(trimmed) {
        return Some(abbr);
    }

    // Try matching just the first word(s) - some Tankathon entries have extra text
    // e.g., "Green Bay (from DAL)" -> strip the parenthetical
    let without_parens = if let Some(idx) = trimmed.find('(') {
        trimmed[..idx].trim()
    } else {
        trimmed
    };

    if let Some(&abbr) = map.get(without_parens) {
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
