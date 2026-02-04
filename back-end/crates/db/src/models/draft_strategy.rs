use chrono::{DateTime, Utc};
use sqlx::types::JsonValue;
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

use domain::models::{DraftStrategy, PositionValueMap};

use crate::errors::{DbError, DbResult};
use crate::models::player::{position_to_string, string_to_position};

/// Database model for draft_strategies table
#[derive(Debug, Clone, FromRow)]
pub struct DraftStrategyDb {
    pub id: Uuid,
    pub team_id: Uuid,
    pub draft_id: Uuid,
    pub bpa_weight: i32,
    pub need_weight: i32,
    pub position_values: Option<JsonValue>,
    pub risk_tolerance: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DraftStrategyDb {
    /// Convert from domain DraftStrategy to database DraftStrategyDb
    pub fn from_domain(strategy: &DraftStrategy) -> DbResult<Self> {
        let position_values = strategy
            .position_values
            .as_ref()
            .map(position_values_to_json)
            .transpose()?;

        Ok(Self {
            id: strategy.id,
            team_id: strategy.team_id,
            draft_id: strategy.draft_id,
            bpa_weight: strategy.bpa_weight,
            need_weight: strategy.need_weight,
            position_values,
            risk_tolerance: strategy.risk_tolerance,
            created_at: strategy.created_at,
            updated_at: strategy.updated_at,
        })
    }

    /// Convert from database DraftStrategyDb to domain DraftStrategy
    pub fn to_domain(&self) -> DbResult<DraftStrategy> {
        let position_values = self
            .position_values
            .as_ref()
            .map(json_to_position_values)
            .transpose()?;

        Ok(DraftStrategy {
            id: self.id,
            team_id: self.team_id,
            draft_id: self.draft_id,
            bpa_weight: self.bpa_weight,
            need_weight: self.need_weight,
            position_values,
            risk_tolerance: self.risk_tolerance,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// Convert PositionValueMap to JSON
fn position_values_to_json(map: &PositionValueMap) -> DbResult<JsonValue> {
    let mut json_map = serde_json::Map::new();
    for (position, value) in map {
        json_map.insert(position_to_string(position), serde_json::json!(value));
    }
    Ok(JsonValue::Object(json_map))
}

/// Convert JSON to PositionValueMap
fn json_to_position_values(json: &JsonValue) -> DbResult<PositionValueMap> {
    let obj = json
        .as_object()
        .ok_or_else(|| DbError::MappingError("position_values is not a JSON object".to_string()))?;

    let mut map = HashMap::new();
    for (key, value) in obj {
        let position = string_to_position(key)?;
        let pos_value: f64 = value
            .as_f64()
            .ok_or_else(|| DbError::MappingError(format!("Invalid value for position {}", key)))?;
        map.insert(position, pos_value);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::Position;

    #[test]
    fn test_domain_to_db_conversion() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let strategy = DraftStrategy::default_strategy(team_id, draft_id);

        let strategy_db = DraftStrategyDb::from_domain(&strategy).unwrap();
        assert_eq!(strategy_db.team_id, team_id);
        assert_eq!(strategy_db.draft_id, draft_id);
        assert_eq!(strategy_db.bpa_weight, 60);
        assert_eq!(strategy_db.need_weight, 40);
        assert_eq!(strategy_db.risk_tolerance, 5);
        assert!(strategy_db.position_values.is_some());
    }

    #[test]
    fn test_db_to_domain_conversion() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();

        let mut json_map = serde_json::Map::new();
        json_map.insert("QB".to_string(), serde_json::json!(1.5));
        json_map.insert("RB".to_string(), serde_json::json!(0.85));

        let strategy_db = DraftStrategyDb {
            id: Uuid::new_v4(),
            team_id,
            draft_id,
            bpa_weight: 70,
            need_weight: 30,
            position_values: Some(JsonValue::Object(json_map)),
            risk_tolerance: 7,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = strategy_db.to_domain();
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.bpa_weight, 70);
        assert_eq!(strategy.need_weight, 30);
        assert_eq!(strategy.risk_tolerance, 7);

        let position_values = strategy.position_values.unwrap();
        assert_eq!(position_values.get(&Position::QB), Some(&1.5));
        assert_eq!(position_values.get(&Position::RB), Some(&0.85));
    }

    #[test]
    fn test_roundtrip_conversion() {
        let team_id = Uuid::new_v4();
        let draft_id = Uuid::new_v4();
        let original = DraftStrategy::default_strategy(team_id, draft_id);

        let db_model = DraftStrategyDb::from_domain(&original).unwrap();
        let roundtrip = db_model.to_domain().unwrap();

        assert_eq!(original.team_id, roundtrip.team_id);
        assert_eq!(original.draft_id, roundtrip.draft_id);
        assert_eq!(original.bpa_weight, roundtrip.bpa_weight);
        assert_eq!(original.need_weight, roundtrip.need_weight);
        assert_eq!(original.risk_tolerance, roundtrip.risk_tolerance);

        let original_values = original.position_values.unwrap();
        let roundtrip_values = roundtrip.position_values.unwrap();
        assert_eq!(
            original_values.get(&Position::QB),
            roundtrip_values.get(&Position::QB)
        );
    }
}
