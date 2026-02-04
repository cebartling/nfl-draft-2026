use crate::errors::{DbError, DbResult};
use chrono::{DateTime, Utc};
use domain::models::{PickTrade, PickTradeDetail, TradeDirection, TradeStatus};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PickTradeDb {
    pub id: Uuid,
    pub session_id: Uuid,
    pub from_team_id: Uuid,
    pub to_team_id: Uuid,
    pub status: String,
    pub from_team_value: i32,
    pub to_team_value: i32,
    pub value_difference: i32,
    pub proposed_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PickTradeDb {
    pub fn from_domain(trade: &PickTrade) -> Self {
        Self {
            id: trade.id,
            session_id: trade.session_id,
            from_team_id: trade.from_team_id,
            to_team_id: trade.to_team_id,
            status: format!("{:?}", trade.status),
            from_team_value: trade.from_team_value,
            to_team_value: trade.to_team_value,
            value_difference: trade.value_difference,
            proposed_at: trade.proposed_at,
            responded_at: trade.responded_at,
            created_at: trade.created_at,
            updated_at: trade.updated_at,
        }
    }

    pub fn to_domain(&self) -> DbResult<PickTrade> {
        Ok(PickTrade {
            id: self.id,
            session_id: self.session_id,
            from_team_id: self.from_team_id,
            to_team_id: self.to_team_id,
            status: match self.status.as_str() {
                "Proposed" => TradeStatus::Proposed,
                "Accepted" => TradeStatus::Accepted,
                "Rejected" => TradeStatus::Rejected,
                _ => {
                    return Err(DbError::MappingError(format!(
                        "Invalid status: {}",
                        self.status
                    )))
                }
            },
            from_team_value: self.from_team_value,
            to_team_value: self.to_team_value,
            value_difference: self.value_difference,
            proposed_at: self.proposed_at,
            responded_at: self.responded_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PickTradeDetailDb {
    pub id: Uuid,
    pub trade_id: Uuid,
    pub pick_id: Uuid,
    pub direction: String,
    pub pick_value: i32,
    pub created_at: DateTime<Utc>,
}

impl PickTradeDetailDb {
    pub fn from_domain(detail: &PickTradeDetail) -> Self {
        Self {
            id: detail.id,
            trade_id: detail.trade_id,
            pick_id: detail.pick_id,
            direction: format!("{:?}", detail.direction),
            pick_value: detail.pick_value,
            created_at: detail.created_at,
        }
    }

    pub fn to_domain(&self) -> DbResult<PickTradeDetail> {
        Ok(PickTradeDetail {
            id: self.id,
            trade_id: self.trade_id,
            pick_id: self.pick_id,
            direction: match self.direction.as_str() {
                "FromTeam" => TradeDirection::FromTeam,
                "ToTeam" => TradeDirection::ToTeam,
                _ => {
                    return Err(DbError::MappingError(format!(
                        "Invalid direction: {}",
                        self.direction
                    )))
                }
            },
            pick_value: self.pick_value,
            created_at: self.created_at,
        })
    }
}
