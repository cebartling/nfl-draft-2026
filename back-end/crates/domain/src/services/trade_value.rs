use crate::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

/// Trait for trade value calculation strategies
pub trait TradeValueChart: Send + Sync {
    /// Get the name of this chart
    fn name(&self) -> &str;

    /// Calculate the value of a pick based on its overall pick number
    fn calculate_pick_value(&self, overall_pick: i32) -> DomainResult<i32>;

    /// Validate if a trade is fair within threshold
    /// threshold_percent: 0-100, e.g., 10 means within 10%
    fn is_trade_fair(&self, value1: i32, value2: i32, threshold_percent: i32) -> bool {
        if value1 == 0 || value2 == 0 {
            return false;
        }

        let larger = value1.max(value2) as f64;
        let smaller = value1.min(value2) as f64;
        let difference_percent = ((larger - smaller) / larger * 100.0) as i32;

        difference_percent <= threshold_percent
    }
}

/// Enum for selecting chart type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    JimmyJohnson,
    RichHill,
    ChaseStudartAV,
    FitzgeraldSpielberger,
    PffWar,
    SurplusValue,
}

impl ChartType {
    pub fn create_chart(&self) -> Box<dyn TradeValueChart> {
        match self {
            ChartType::JimmyJohnson => Box::new(JimmyJohnsonChart::new()),
            ChartType::RichHill => Box::new(RichHillChart::new()),
            ChartType::ChaseStudartAV => Box::new(ChaseStudartAVChart::new()),
            ChartType::FitzgeraldSpielberger => Box::new(FitzgeraldSpielbergerChart::new()),
            ChartType::PffWar => Box::new(PffWarChart::new()),
            ChartType::SurplusValue => Box::new(SurplusValueChart::new()),
        }
    }
}

/// Helper function for exponential decay beyond chart range
fn calculate_with_decay(pick_values: &[i32], overall_pick: i32) -> i32 {
    let index = (overall_pick - 1) as usize;
    if index >= pick_values.len() {
        let base_value = *pick_values.last().unwrap();
        let decay_factor = 0.95_f64;
        let extra_picks = (index - pick_values.len()) as f64;
        let value = (base_value as f64 * decay_factor.powf(extra_picks)) as i32;
        value.max(1) // Minimum value of 1
    } else {
        pick_values[index]
    }
}

// ============================================================================
// 1. Jimmy Johnson Chart (Traditional)
// ============================================================================

pub struct JimmyJohnsonChart {
    pick_values: Vec<i32>,
}

impl JimmyJohnsonChart {
    pub fn new() -> Self {
        Self {
            pick_values: vec![
                // Round 1 (picks 1-32)
                3000, 2600, 2200, 1800, 1700, 1600, 1500, 1400,
                1350, 1300, 1250, 1200, 1150, 1100, 1050, 1000,
                950, 900, 875, 850, 800, 780, 760, 740,
                720, 700, 680, 660, 640, 620, 600, 590,
                // Round 2 (picks 33-64)
                580, 560, 550, 540, 530, 520, 510, 500,
                490, 480, 470, 460, 450, 440, 430, 420,
                410, 400, 390, 380, 370, 360, 350, 340,
                330, 320, 310, 300, 292, 284, 276, 268,
                // Round 3 (picks 65-96)
                260, 252, 244, 236, 228, 220, 212, 204,
                197, 190, 183, 176, 170, 164, 158, 152,
                146, 140, 136, 132, 128, 124, 120, 116,
                112, 108, 104, 100, 96, 92, 88, 84,
                // Round 4 (picks 97-128)
                80, 78, 76, 74, 72, 70, 68, 66,
                64, 62, 60, 58, 56, 54, 52, 50,
                49, 48, 47, 46, 45, 44, 43, 42,
                41, 40, 39, 38, 37, 36, 35, 34,
                // Round 5 (picks 129-160)
                33, 32, 31, 30, 29, 28, 27, 26,
                25, 24, 23, 22, 21, 20, 19, 18,
                17, 17, 16, 16, 15, 15, 14, 14,
                13, 13, 13, 13, 13, 12, 12, 12,
                // Round 6 (picks 161-192)
                12, 12, 11, 11, 10, 10, 9, 9,
                8, 8, 8, 8, 8, 7, 7, 7,
                7, 7, 6, 6, 6, 6, 6, 5,
                5, 5, 5, 5, 4, 4, 4, 4,
                // Round 7 (picks 193-224)
                4, 3, 3, 3, 3, 3, 3, 3,
                3, 3, 3, 2, 2, 2, 2, 2,
                2, 2, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
            ]
        }
    }
}

impl TradeValueChart for JimmyJohnsonChart {
    fn name(&self) -> &str {
        "Jimmy Johnson"
    }

    fn calculate_pick_value(&self, overall_pick: i32) -> DomainResult<i32> {
        if overall_pick < 1 {
            return Err(DomainError::ValidationError(
                format!("Invalid pick number: {}", overall_pick)
            ));
        }
        Ok(calculate_with_decay(&self.pick_values, overall_pick))
    }
}

// ============================================================================
// 2. Rich Hill Chart (More accurate modern values)
// ============================================================================

pub struct RichHillChart {
    pick_values: Vec<i32>,
}

impl RichHillChart {
    pub fn new() -> Self {
        Self {
            // Rich Hill chart emphasizes top picks more heavily
            pick_values: vec![
                // Round 1 (1-32) - Higher premium on top picks
                3400, 3000, 2600, 2200, 2000, 1800, 1700, 1600,
                1500, 1450, 1400, 1350, 1300, 1250, 1200, 1150,
                1100, 1050, 1000, 960, 920, 880, 840, 800,
                770, 740, 710, 680, 655, 630, 605, 580,
            ]
        }
    }
}

impl TradeValueChart for RichHillChart {
    fn name(&self) -> &str {
        "Rich Hill"
    }

    fn calculate_pick_value(&self, overall_pick: i32) -> DomainResult<i32> {
        if overall_pick < 1 {
            return Err(DomainError::ValidationError(
                format!("Invalid pick number: {}", overall_pick)
            ));
        }
        Ok(calculate_with_decay(&self.pick_values, overall_pick))
    }
}

// ============================================================================
// 3. Chase Stuart AV Chart (Approximate Value based)
// ============================================================================

pub struct ChaseStudartAVChart {
    pick_values: Vec<i32>,
}

impl ChaseStudartAVChart {
    pub fn new() -> Self {
        Self {
            // Placeholder values - would be based on actual AV data
            // This chart is flatter (less premium on top picks)
            pick_values: vec![
                2800, 2500, 2200, 1900, 1700, 1550, 1450, 1350,
                1250, 1180, 1120, 1070, 1020, 980, 940, 900,
            ]
        }
    }
}

impl TradeValueChart for ChaseStudartAVChart {
    fn name(&self) -> &str {
        "Chase Stuart AV"
    }

    fn calculate_pick_value(&self, overall_pick: i32) -> DomainResult<i32> {
        if overall_pick < 1 {
            return Err(DomainError::ValidationError(
                format!("Invalid pick number: {}", overall_pick)
            ));
        }
        Ok(calculate_with_decay(&self.pick_values, overall_pick))
    }
}

// ============================================================================
// 4. Fitzgerald-Spielberger Chart
// ============================================================================

pub struct FitzgeraldSpielbergerChart {
    pick_values: Vec<i32>,
}

impl FitzgeraldSpielbergerChart {
    pub fn new() -> Self {
        Self {
            // Placeholder - would need actual data
            pick_values: vec![
                3200, 2800, 2400, 2000, 1800, 1650, 1500, 1380,
                1280, 1200, 1130, 1070, 1010, 960, 920, 880,
            ]
        }
    }
}

impl TradeValueChart for FitzgeraldSpielbergerChart {
    fn name(&self) -> &str {
        "Fitzgerald-Spielberger"
    }

    fn calculate_pick_value(&self, overall_pick: i32) -> DomainResult<i32> {
        if overall_pick < 1 {
            return Err(DomainError::ValidationError(
                format!("Invalid pick number: {}", overall_pick)
            ));
        }
        Ok(calculate_with_decay(&self.pick_values, overall_pick))
    }
}

// ============================================================================
// 5. PFF WAR Chart (Pro Football Focus Wins Above Replacement)
// ============================================================================

pub struct PffWarChart {
    pick_values: Vec<i32>,
}

impl PffWarChart {
    pub fn new() -> Self {
        Self {
            // Placeholder - would be based on PFF WAR data
            pick_values: vec![
                3100, 2700, 2300, 1950, 1750, 1600, 1480, 1370,
                1270, 1190, 1120, 1060, 1010, 960, 920, 880,
            ]
        }
    }
}

impl TradeValueChart for PffWarChart {
    fn name(&self) -> &str {
        "PFF WAR"
    }

    fn calculate_pick_value(&self, overall_pick: i32) -> DomainResult<i32> {
        if overall_pick < 1 {
            return Err(DomainError::ValidationError(
                format!("Invalid pick number: {}", overall_pick)
            ));
        }
        Ok(calculate_with_decay(&self.pick_values, overall_pick))
    }
}

// ============================================================================
// 6. Surplus Value Chart (Contract value based)
// ============================================================================

pub struct SurplusValueChart {
    pick_values: Vec<i32>,
}

impl SurplusValueChart {
    pub fn new() -> Self {
        Self {
            // Placeholder - based on rookie contract surplus value
            pick_values: vec![
                3500, 3100, 2700, 2300, 2050, 1850, 1700, 1550,
                1420, 1310, 1210, 1130, 1060, 1000, 950, 900,
            ]
        }
    }
}

impl TradeValueChart for SurplusValueChart {
    fn name(&self) -> &str {
        "Surplus Value"
    }

    fn calculate_pick_value(&self, overall_pick: i32) -> DomainResult<i32> {
        if overall_pick < 1 {
            return Err(DomainError::ValidationError(
                format!("Invalid pick number: {}", overall_pick)
            ));
        }
        Ok(calculate_with_decay(&self.pick_values, overall_pick))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jimmy_johnson_values() {
        let chart = JimmyJohnsonChart::new();
        assert_eq!(chart.calculate_pick_value(1).unwrap(), 3000);
        assert_eq!(chart.calculate_pick_value(16).unwrap(), 1000);
        assert_eq!(chart.calculate_pick_value(32).unwrap(), 590);
    }

    #[test]
    fn test_jimmy_johnson_decay() {
        let chart = JimmyJohnsonChart::new();
        // Pick 225 should use decay from pick 224 (which has value 1)
        let value_224 = chart.calculate_pick_value(224).unwrap();
        let value_225 = chart.calculate_pick_value(225).unwrap();
        assert_eq!(value_224, 1);
        assert_eq!(value_225, 1); // Decay bottoms out at 1

        // Test decay with a higher pick value
        let value_200 = chart.calculate_pick_value(200).unwrap();
        assert!(value_200 >= 1);
    }

    #[test]
    fn test_invalid_pick_number() {
        let chart = JimmyJohnsonChart::new();
        let result = chart.calculate_pick_value(0);
        assert!(result.is_err());
        match result {
            Err(DomainError::ValidationError(msg)) => {
                assert!(msg.contains("Invalid pick number"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_chart_selection() {
        let jj_chart = ChartType::JimmyJohnson.create_chart();
        assert_eq!(jj_chart.name(), "Jimmy Johnson");

        let rh_chart = ChartType::RichHill.create_chart();
        assert_eq!(rh_chart.name(), "Rich Hill");

        let cs_chart = ChartType::ChaseStudartAV.create_chart();
        assert_eq!(cs_chart.name(), "Chase Stuart AV");

        let fs_chart = ChartType::FitzgeraldSpielberger.create_chart();
        assert_eq!(fs_chart.name(), "Fitzgerald-Spielberger");

        let pff_chart = ChartType::PffWar.create_chart();
        assert_eq!(pff_chart.name(), "PFF WAR");

        let sv_chart = ChartType::SurplusValue.create_chart();
        assert_eq!(sv_chart.name(), "Surplus Value");
    }

    #[test]
    fn test_trade_fairness() {
        let chart = JimmyJohnsonChart::new();

        // Exact match
        assert!(chart.is_trade_fair(1000, 1000, 10));

        // Within 10% threshold
        assert!(chart.is_trade_fair(1000, 950, 10));

        // Outside 10% threshold
        assert!(!chart.is_trade_fair(1000, 800, 10));

        // Zero values
        assert!(!chart.is_trade_fair(0, 1000, 10));
        assert!(!chart.is_trade_fair(1000, 0, 10));
    }

    #[test]
    fn test_trade_fairness_percentage_calculation() {
        let chart = JimmyJohnsonChart::new();

        // 5% difference (1000 vs 950) - should pass with 10% threshold
        assert!(chart.is_trade_fair(1000, 950, 10));
        assert!(chart.is_trade_fair(950, 1000, 10));

        // 15% difference (1000 vs 850) - should fail with 10% threshold
        assert!(!chart.is_trade_fair(1000, 850, 10));

        // 15% difference - should pass with 20% threshold
        assert!(chart.is_trade_fair(1000, 850, 20));
    }

    #[test]
    fn test_rich_hill_premium_on_top_picks() {
        let jj_chart = JimmyJohnsonChart::new();
        let rh_chart = RichHillChart::new();

        // Rich Hill should value pick 1 higher than Jimmy Johnson
        assert!(rh_chart.calculate_pick_value(1).unwrap() > jj_chart.calculate_pick_value(1).unwrap());
    }
}
