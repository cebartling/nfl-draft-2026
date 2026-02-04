use anyhow::{anyhow, Result};
use domain::models::Position;

/// Maps source position abbreviations to the canonical Position enum variants.
///
/// Handles common variations from scouting sources (e.g., EDGE -> DE, HB -> RB).
/// Returns an error for ambiguous positions that require manual assignment.
pub fn map_position(source: &str) -> Result<Position> {
    let normalized = source.trim().to_uppercase();

    match normalized.as_str() {
        "QB" => Ok(Position::QB),
        "RB" | "HB" => Ok(Position::RB),
        "WR" => Ok(Position::WR),
        "TE" => Ok(Position::TE),
        "OT" | "T" => Ok(Position::OT),
        "OG" | "G" => Ok(Position::OG),
        "C" => Ok(Position::C),
        "DE" | "EDGE" => Ok(Position::DE),
        "DT" | "NT" => Ok(Position::DT),
        "LB" | "OLB" | "ILB" | "MLB" => Ok(Position::LB),
        "CB" => Ok(Position::CB),
        "S" | "SS" | "FS" => Ok(Position::S),
        "K" => Ok(Position::K),
        "P" => Ok(Position::P),
        _ => Err(anyhow!(
            "Invalid position: '{}'. Must manually assign a valid position.",
            source
        )),
    }
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
        assert_eq!(map_position("EDGE").unwrap(), Position::DE);
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
        assert!(map_position("DL").is_err());
        assert!(map_position("").is_err());
        assert!(map_position("INVALID").is_err());
    }

    #[test]
    fn test_error_message_includes_input() {
        let err = map_position("ATH").unwrap_err();
        assert!(err.to_string().contains("ATH"));
    }
}
