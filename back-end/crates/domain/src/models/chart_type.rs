use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Trade value chart selection
///
/// Represents the different methodologies for calculating NFL draft pick values.
/// Each chart uses a different analytical approach to determine trade fairness.
///
/// # Available Charts
///
/// - **JimmyJohnson**: Traditional chart from 1990s Dallas Cowboys
/// - **RichHill**: Modern analytics-based on historical trades
/// - **ChaseStudartAV**: Empirical performance based on Approximate Value
/// - **FitzgeraldSpielberger**: Contract value based on rookie APY analysis
/// - **PffWar**: Expected performance using PFF's WAR metric
/// - **SurplusValue**: Economic efficiency (value minus cost)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    JimmyJohnson,
    RichHill,
    ChaseStudartAV,
    FitzgeraldSpielberger,
    PffWar,
    SurplusValue,
}

impl fmt::Display for ChartType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ChartType::JimmyJohnson => "JimmyJohnson",
            ChartType::RichHill => "RichHill",
            ChartType::ChaseStudartAV => "ChaseStudartAV",
            ChartType::FitzgeraldSpielberger => "FitzgeraldSpielberger",
            ChartType::PffWar => "PffWar",
            ChartType::SurplusValue => "SurplusValue",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for ChartType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "JimmyJohnson" => Ok(ChartType::JimmyJohnson),
            "RichHill" => Ok(ChartType::RichHill),
            "ChaseStudartAV" => Ok(ChartType::ChaseStudartAV),
            "FitzgeraldSpielberger" => Ok(ChartType::FitzgeraldSpielberger),
            "PffWar" => Ok(ChartType::PffWar),
            "SurplusValue" => Ok(ChartType::SurplusValue),
            _ => Err(format!("Invalid chart type: {}", s)),
        }
    }
}
