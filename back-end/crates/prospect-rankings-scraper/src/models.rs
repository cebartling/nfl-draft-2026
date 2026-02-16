use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RankingMeta {
    pub version: String,
    pub source: String,
    pub source_url: String,
    pub draft_year: i32,
    pub scraped_at: String,
    pub total_prospects: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingEntry {
    pub rank: i32,
    pub first_name: String,
    pub last_name: String,
    pub position: String,
    pub school: String,
    #[serde(default)]
    pub height_inches: Option<i32>,
    #[serde(default)]
    pub weight_pounds: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RankingData {
    pub meta: RankingMeta,
    pub rankings: Vec<RankingEntry>,
}
