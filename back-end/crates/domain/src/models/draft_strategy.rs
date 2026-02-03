use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};
use crate::models::Position;

pub type PositionValueMap = HashMap<Position, f64>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct DraftStrategy {
    pub id: Uuid,
    pub team_id: Uuid,
    pub draft_id: Uuid,
    pub bpa_weight: i32,
    pub need_weight: i32,
    pub position_values: Option<PositionValueMap>,
    pub risk_tolerance: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DraftStrategy {
    pub fn new(
        team_id: Uuid,
        draft_id: Uuid,
        bpa_weight: i32,
        need_weight: i32,
        position_values: Option<PositionValueMap>,
        risk_tolerance: i32,
    ) -> DomainResult<Self> {
        Self::validate_weights(bpa_weight, need_weight)?;
        Self::validate_risk_tolerance(risk_tolerance)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            team_id,
            draft_id,
            bpa_weight,
            need_weight,
            position_values,
            risk_tolerance,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn default_strategy(team_id: Uuid, draft_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            team_id,
            draft_id,
            bpa_weight: 60,
            need_weight: 40,
            position_values: Some(Self::default_position_values()),
            risk_tolerance: 5,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_weights(&mut self, bpa_weight: i32, need_weight: i32) -> DomainResult<()> {
        Self::validate_weights(bpa_weight, need_weight)?;
        self.bpa_weight = bpa_weight;
        self.need_weight = need_weight;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_risk_tolerance(&mut self, risk_tolerance: i32) -> DomainResult<()> {
        Self::validate_risk_tolerance(risk_tolerance)?;
        self.risk_tolerance = risk_tolerance;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_position_values(&mut self, position_values: PositionValueMap) {
        self.position_values = Some(position_values);
        self.updated_at = Utc::now();
    }

    pub fn get_position_value(&self, position: Position) -> f64 {
        self.position_values
            .as_ref()
            .and_then(|map| map.get(&position).copied())
            .unwrap_or_else(|| Self::default_position_values().get(&position).copied().unwrap_or(1.0))
    }

    fn validate_weights(bpa_weight: i32, need_weight: i32) -> DomainResult<()> {
        if bpa_weight < 0 || bpa_weight > 100 {
            return Err(DomainError::ValidationError(
                "BPA weight must be between 0 and 100".to_string(),
            ));
        }
        if need_weight < 0 || need_weight > 100 {
            return Err(DomainError::ValidationError(
                "Need weight must be between 0 and 100".to_string(),
            ));
        }
        if bpa_weight + need_weight != 100 {
            return Err(DomainError::ValidationError(
                "BPA weight and need weight must sum to 100".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_risk_tolerance(risk_tolerance: i32) -> DomainResult<()> {
        if risk_tolerance < 0 || risk_tolerance > 10 {
            return Err(DomainError::ValidationError(
                "Risk tolerance must be between 0 and 10".to_string(),
            ));
        }
        Ok(())
    }

    fn default_position_values() -> PositionValueMap {
        let mut map = HashMap::new();
        // Offensive positions
        map.insert(Position::QB, 1.5);
        map.insert(Position::RB, 0.85);
        map.insert(Position::WR, 1.0);
        map.insert(Position::TE, 0.9);
        map.insert(Position::OT, 1.2);
        map.insert(Position::OG, 1.0);
        map.insert(Position::C, 1.0);
        // Defensive positions
        map.insert(Position::DE, 1.3);
        map.insert(Position::DT, 1.1);
        map.insert(Position::LB, 1.1);
        map.insert(Position::CB, 1.2);
        map.insert(Position::S, 1.0);
        // Special teams
        map.insert(Position::K, 0.5);
        map.insert(Position::P, 0.5);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_draft_strategy() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let strategy = DraftStrategy::new(team_id, draft_id, 60, 40, None, 5).unwrap();

        assert_eq!(strategy.team_id, team_id);
        assert_eq!(strategy.draft_id, draft_id);
        assert_eq!(strategy.bpa_weight, 60);
        assert_eq!(strategy.need_weight, 40);
        assert_eq!(strategy.risk_tolerance, 5);
    }

    #[test]
    fn test_default_strategy() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let strategy = DraftStrategy::default_strategy(team_id, draft_id);

        assert_eq!(strategy.bpa_weight, 60);
        assert_eq!(strategy.need_weight, 40);
        assert_eq!(strategy.risk_tolerance, 5);
        assert!(strategy.position_values.is_some());
    }

    #[test]
    fn test_invalid_weights_range() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        assert!(DraftStrategy::new(team_id, draft_id, -10, 110, None, 5).is_err());
        assert!(DraftStrategy::new(team_id, draft_id, 110, -10, None, 5).is_err());
    }

    #[test]
    fn test_invalid_weights_sum() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        assert!(DraftStrategy::new(team_id, draft_id, 50, 30, None, 5).is_err());
        assert!(DraftStrategy::new(team_id, draft_id, 70, 40, None, 5).is_err());
    }

    #[test]
    fn test_invalid_risk_tolerance() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        assert!(DraftStrategy::new(team_id, draft_id, 60, 40, None, -1).is_err());
        assert!(DraftStrategy::new(team_id, draft_id, 60, 40, None, 11).is_err());
    }

    #[test]
    fn test_update_weights() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let mut strategy = DraftStrategy::new(team_id, draft_id, 60, 40, None, 5).unwrap();

        strategy.update_weights(70, 30).unwrap();
        assert_eq!(strategy.bpa_weight, 70);
        assert_eq!(strategy.need_weight, 30);

        assert!(strategy.update_weights(50, 30).is_err());
    }

    #[test]
    fn test_update_risk_tolerance() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let mut strategy = DraftStrategy::new(team_id, draft_id, 60, 40, None, 5).unwrap();

        strategy.update_risk_tolerance(8).unwrap();
        assert_eq!(strategy.risk_tolerance, 8);

        assert!(strategy.update_risk_tolerance(11).is_err());
    }

    #[test]
    fn test_get_position_value() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let strategy = DraftStrategy::default_strategy(team_id, draft_id);

        assert_eq!(strategy.get_position_value(Position::QB), 1.5);
        assert_eq!(strategy.get_position_value(Position::DE), 1.3);
        assert_eq!(strategy.get_position_value(Position::RB), 0.85);
        assert_eq!(strategy.get_position_value(Position::K), 0.5);
    }

    #[test]
    fn test_default_position_values() {
        let values = DraftStrategy::default_position_values();

        assert_eq!(values.get(&Position::QB), Some(&1.5));
        assert_eq!(values.get(&Position::DE), Some(&1.3));
        assert_eq!(values.get(&Position::OT), Some(&1.2));
        assert_eq!(values.get(&Position::CB), Some(&1.2));
        assert_eq!(values.get(&Position::WR), Some(&1.0));
        assert_eq!(values.get(&Position::RB), Some(&0.85));
        assert_eq!(values.get(&Position::K), Some(&0.5));
    }
}
