use serde::{Deserialize, Serialize};

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
