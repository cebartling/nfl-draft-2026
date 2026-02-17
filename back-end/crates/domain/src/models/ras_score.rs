use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Individual measurement score from the RAS engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementScore {
    pub measurement: String,
    pub raw_value: f64,
    pub percentile: f64,
    pub score: f64,
}

/// Complete RAS (Relative Athletic Score) for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasScore {
    pub player_id: Uuid,
    pub overall_score: Option<f64>,
    pub size_score: Option<f64>,
    pub speed_score: Option<f64>,
    pub strength_score: Option<f64>,
    pub explosion_score: Option<f64>,
    pub agility_score: Option<f64>,
    pub measurements_used: usize,
    pub measurements_total: usize,
    pub individual_scores: Vec<MeasurementScore>,
    pub explanation: Option<String>,
}

impl RasScore {
    /// Minimum number of measurements required for an overall score
    pub const MIN_MEASUREMENTS: usize = 6;
    pub const TOTAL_MEASUREMENTS: usize = 10;
}
