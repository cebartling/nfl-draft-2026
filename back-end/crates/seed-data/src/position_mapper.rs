use anyhow::{anyhow, Result};
use domain::models::Position;

/// Maps source position abbreviations to the canonical Position enum variants.
///
/// Handles common variations from scouting sources (e.g., EDGE -> DE, HB -> RB).
/// Returns an error for ambiguous positions that require manual assignment.
/// Logs at debug level when an alternate abbreviation is mapped to a canonical position.
pub fn map_position(source: &str) -> Result<Position> {
    let normalized = source.trim().to_uppercase();

    let (position, canonical) = match normalized.as_str() {
        "QB" => (Position::QB, "QB"),
        "RB" => (Position::RB, "RB"),
        "HB" => (Position::RB, "RB"),
        "WR" => (Position::WR, "WR"),
        "TE" => (Position::TE, "TE"),
        "OT" => (Position::OT, "OT"),
        "T" => (Position::OT, "OT"),
        "OG" => (Position::OG, "OG"),
        "G" => (Position::OG, "OG"),
        // IOL (Interior Offensive Line) → OG: most IOL prospects play guard
        "IOL" => (Position::OG, "OG"),
        "C" => (Position::C, "C"),
        "DE" => (Position::DE, "DE"),
        "EDGE" => (Position::DE, "DE"),
        // EDGE/LB hybrid → DE: prioritize pass-rush role over coverage
        "EDGE/LB" => (Position::DE, "DE"),
        "DT" => (Position::DT, "DT"),
        // DL (generic defensive line) → DT: most generic DL prospects are interior
        "DL" => (Position::DT, "DT"),
        "NT" => (Position::DT, "DT"),
        "LB" => (Position::LB, "LB"),
        "OLB" => (Position::LB, "LB"),
        "ILB" => (Position::LB, "LB"),
        "MLB" => (Position::LB, "LB"),
        "CB" => (Position::CB, "CB"),
        "S" => (Position::S, "S"),
        "SS" => (Position::S, "S"),
        "FS" => (Position::S, "S"),
        "K" => (Position::K, "K"),
        "P" => (Position::P, "P"),
        _ => {
            return Err(anyhow!(
                "Invalid position: '{}'. Must manually assign a valid position.",
                source
            ))
        }
    };

    if normalized != canonical {
        tracing::debug!(
            source = source,
            canonical = canonical,
            "Alternate abbreviation '{}' mapped to canonical '{}'",
            normalized,
            canonical
        );
    }

    Ok(position)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_match_positions() {
        assert_eq!(map_position("QB").unwrap(), Position::QB);
        assert_eq!(map_position("RB").unwrap(), Position::RB);
        assert_eq!(map_position("WR").unwrap(), Position::WR);
        assert_eq!(map_position("TE").unwrap(), Position::TE);
        assert_eq!(map_position("OT").unwrap(), Position::OT);
        assert_eq!(map_position("OG").unwrap(), Position::OG);
        assert_eq!(map_position("C").unwrap(), Position::C);
        assert_eq!(map_position("DE").unwrap(), Position::DE);
        assert_eq!(map_position("DT").unwrap(), Position::DT);
        assert_eq!(map_position("LB").unwrap(), Position::LB);
        assert_eq!(map_position("CB").unwrap(), Position::CB);
        assert_eq!(map_position("S").unwrap(), Position::S);
        assert_eq!(map_position("K").unwrap(), Position::K);
        assert_eq!(map_position("P").unwrap(), Position::P);
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(map_position("qb").unwrap(), Position::QB);
        assert_eq!(map_position("Qb").unwrap(), Position::QB);
        assert_eq!(map_position("edge").unwrap(), Position::DE);
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(map_position(" QB ").unwrap(), Position::QB);
        assert_eq!(map_position("  EDGE  ").unwrap(), Position::DE);
    }

    #[test]
    fn test_alternate_abbreviations() {
        assert_eq!(map_position("HB").unwrap(), Position::RB);
        assert_eq!(map_position("T").unwrap(), Position::OT);
        assert_eq!(map_position("G").unwrap(), Position::OG);
        assert_eq!(map_position("IOL").unwrap(), Position::OG);
        assert_eq!(map_position("EDGE").unwrap(), Position::DE);
        assert_eq!(map_position("EDGE/LB").unwrap(), Position::DE);
        assert_eq!(map_position("DL").unwrap(), Position::DT);
        assert_eq!(map_position("NT").unwrap(), Position::DT);
        assert_eq!(map_position("OLB").unwrap(), Position::LB);
        assert_eq!(map_position("ILB").unwrap(), Position::LB);
        assert_eq!(map_position("MLB").unwrap(), Position::LB);
        assert_eq!(map_position("SS").unwrap(), Position::S);
        assert_eq!(map_position("FS").unwrap(), Position::S);
    }

    #[test]
    fn test_invalid_positions() {
        assert!(map_position("ATH").is_err());
        assert!(map_position("DB").is_err());
        assert!(map_position("OL").is_err());
        assert!(map_position("").is_err());
        assert!(map_position("INVALID").is_err());
    }

    #[test]
    fn test_error_message_includes_input() {
        let err = map_position("ATH").unwrap_err();
        assert!(err.to_string().contains("ATH"));
    }
}
