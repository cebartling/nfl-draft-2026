use std::collections::HashMap;
use std::sync::LazyLock;

/// Overrides for SVG abbreviations that don't match our canonical abbreviations.
static SVG_ABBR_OVERRIDES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let entries: &[(&str, &str)] = &[("wsh", "WAS"), ("jac", "JAX")];
    entries.iter().copied().collect()
});

/// Normalize a Tankathon SVG URL slug to a canonical NFL team abbreviation.
///
/// Tankathon logos use URL paths like `/nfl/lv.svg`. This function takes the slug
/// portion (e.g. `"lv"`) and returns the canonical abbreviation (e.g. `"LV"`).
/// Special cases like `"wsh"` → `"WAS"` are handled via an override map.
pub fn normalize_svg_abbreviation(slug: &str) -> String {
    let lower = slug.to_lowercase();
    if let Some(&canonical) = SVG_ABBR_OVERRIDES.get(lower.as_str()) {
        canonical.to_string()
    } else {
        slug.to_uppercase()
    }
}

/// Lazily-initialized map from lowercased Tankathon display names to NFL team abbreviations.
#[cfg_attr(not(test), allow(dead_code))]
static TEAM_NAME_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
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

    entries.iter().copied().collect()
});

/// Resolve a display name to an NFL abbreviation.
/// Uses case-insensitive matching. Tries exact match first, then strips parentheticals.
#[cfg_attr(not(test), allow(dead_code))]
pub fn resolve_team_abbreviation(display_name: &str) -> Option<&'static str> {
    let normalized = display_name.trim().to_lowercase();

    if let Some(&abbr) = TEAM_NAME_MAP.get(normalized.as_str()) {
        return Some(abbr);
    }

    // Try matching just the first word(s) - some Tankathon entries have extra text
    // e.g., "Green Bay (from DAL)" -> strip the parenthetical
    if let Some(idx) = normalized.find('(') {
        let without_parens = normalized[..idx].trim();
        if let Some(&abbr) = TEAM_NAME_MAP.get(without_parens) {
            return Some(abbr);
        }
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
        let values: std::collections::HashSet<&&str> = TEAM_NAME_MAP.values().collect();
        for abbr in &expected {
            assert!(values.contains(&abbr), "Missing abbreviation: {}", abbr);
        }
    }

    #[test]
    fn test_normalize_svg_simple() {
        assert_eq!(normalize_svg_abbreviation("lv"), "LV");
        assert_eq!(normalize_svg_abbreviation("nyj"), "NYJ");
        assert_eq!(normalize_svg_abbreviation("sf"), "SF");
        assert_eq!(normalize_svg_abbreviation("kc"), "KC");
        assert_eq!(normalize_svg_abbreviation("lar"), "LAR");
    }

    #[test]
    fn test_normalize_svg_overrides() {
        assert_eq!(normalize_svg_abbreviation("wsh"), "WAS");
        assert_eq!(normalize_svg_abbreviation("WSH"), "WAS");
        assert_eq!(normalize_svg_abbreviation("jac"), "JAX");
        assert_eq!(normalize_svg_abbreviation("JAC"), "JAX");
    }

    #[test]
    fn test_normalize_svg_already_uppercase() {
        assert_eq!(normalize_svg_abbreviation("DAL"), "DAL");
        assert_eq!(normalize_svg_abbreviation("GB"), "GB");
    }
}
