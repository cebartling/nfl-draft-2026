use anyhow::Result;
use serde::Deserialize;

use domain::models::{CombinePercentile, Measurement};
use domain::repositories::CombinePercentileRepository;

#[derive(Debug, Deserialize)]
pub struct PercentileFileData {
    pub meta: PercentileFileMeta,
    pub percentiles: Vec<PercentileFileEntry>,
}

#[derive(Debug, Deserialize)]
pub struct PercentileFileMeta {
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct PercentileFileEntry {
    pub position: String,
    pub measurement: String,
    pub sample_size: i32,
    pub min_value: f64,
    pub p10: f64,
    pub p20: f64,
    pub p30: f64,
    pub p40: f64,
    pub p50: f64,
    pub p60: f64,
    pub p70: f64,
    pub p80: f64,
    pub p90: f64,
    pub max_value: f64,
    #[serde(default = "default_years_start")]
    pub years_start: i32,
    #[serde(default = "default_years_end")]
    pub years_end: i32,
}

fn default_years_start() -> i32 {
    2000
}
fn default_years_end() -> i32 {
    2025
}

pub struct PercentileLoadStats {
    pub upserted: usize,
    pub errors: Vec<String>,
}

pub fn parse_percentile_json(json: &str) -> Result<PercentileFileData> {
    let data: PercentileFileData = serde_json::from_str(json)?;
    Ok(data)
}

pub async fn load_percentiles(
    data: &PercentileFileData,
    repo: &dyn CombinePercentileRepository,
) -> Result<PercentileLoadStats> {
    let mut upserted = 0;
    let mut errors = Vec::new();

    for entry in &data.percentiles {
        let measurement: Measurement = match entry.measurement.parse() {
            Ok(m) => m,
            Err(e) => {
                errors.push(format!(
                    "Invalid measurement '{}' for {}: {}",
                    entry.measurement, entry.position, e
                ));
                continue;
            }
        };

        let percentile = match CombinePercentile::new(entry.position.clone(), measurement) {
            Ok(p) => p,
            Err(e) => {
                errors.push(format!(
                    "Invalid position '{}': {}",
                    entry.position, e
                ));
                continue;
            }
        };

        let percentile = match percentile
            .with_percentiles(
                entry.sample_size,
                entry.min_value,
                entry.p10,
                entry.p20,
                entry.p30,
                entry.p40,
                entry.p50,
                entry.p60,
                entry.p70,
                entry.p80,
                entry.p90,
                entry.max_value,
            )
            .and_then(|p| p.with_years(entry.years_start, entry.years_end))
        {
            Ok(p) => p,
            Err(e) => {
                errors.push(format!(
                    "Validation error for {} {}: {}",
                    entry.position, entry.measurement, e
                ));
                continue;
            }
        };

        match repo.upsert(&percentile).await {
            Ok(_) => upserted += 1,
            Err(e) => {
                errors.push(format!(
                    "Database error for {} {}: {}",
                    entry.position, entry.measurement, e
                ));
            }
        }
    }

    Ok(PercentileLoadStats { upserted, errors })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_percentile_json() {
        let json = r#"{
            "meta": { "source": "template", "description": "test", "generated_at": "2026-02-16" },
            "percentiles": [
                {
                    "position": "QB",
                    "measurement": "forty_yard_dash",
                    "sample_size": 200,
                    "min_value": 4.4,
                    "p10": 4.55, "p20": 4.6, "p30": 4.65, "p40": 4.7,
                    "p50": 4.75, "p60": 4.8, "p70": 4.85, "p80": 4.9,
                    "p90": 5.0,
                    "max_value": 5.3
                }
            ]
        }"#;

        let data = parse_percentile_json(json).unwrap();
        assert_eq!(data.percentiles.len(), 1);
        assert_eq!(data.percentiles[0].position, "QB");
        assert_eq!(data.percentiles[0].measurement, "forty_yard_dash");
    }
}
