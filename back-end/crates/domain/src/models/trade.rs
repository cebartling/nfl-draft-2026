use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeStatus {
    Proposed,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeDirection {
    FromTeam,  // Pick given by from_team
    ToTeam,    // Pick given by to_team
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PickTrade {
    pub id: Uuid,
    pub session_id: Uuid,
    pub from_team_id: Uuid,
    pub to_team_id: Uuid,
    pub status: TradeStatus,
    pub from_team_value: i32,
    pub to_team_value: i32,
    pub value_difference: i32,
    pub proposed_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PickTrade {
    pub fn new(
        session_id: Uuid,
        from_team_id: Uuid,
        to_team_id: Uuid,
        from_team_value: i32,
        to_team_value: i32,
    ) -> DomainResult<Self> {
        Self::validate_different_teams(from_team_id, to_team_id)?;

        let value_difference = (from_team_value - to_team_value).abs();
        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            session_id,
            from_team_id,
            to_team_id,
            status: TradeStatus::Proposed,
            from_team_value,
            to_team_value,
            value_difference,
            proposed_at: now,
            responded_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn accept(&mut self) -> DomainResult<()> {
        match self.status {
            TradeStatus::Proposed => {
                self.status = TradeStatus::Accepted;
                self.responded_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(DomainError::InvalidState(
                format!("Cannot accept trade in status: {:?}", self.status)
            )),
        }
    }

    pub fn reject(&mut self) -> DomainResult<()> {
        match self.status {
            TradeStatus::Proposed => {
                self.status = TradeStatus::Rejected;
                self.responded_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(DomainError::InvalidState(
                format!("Cannot reject trade in status: {:?}", self.status)
            )),
        }
    }

    fn validate_different_teams(from_team_id: Uuid, to_team_id: Uuid) -> DomainResult<()> {
        if from_team_id == to_team_id {
            return Err(DomainError::ValidationError(
                "Cannot trade with the same team".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PickTradeDetail {
    pub id: Uuid,
    pub trade_id: Uuid,
    pub pick_id: Uuid,
    pub direction: TradeDirection,
    pub pick_value: i32,
    pub created_at: DateTime<Utc>,
}

impl PickTradeDetail {
    pub fn new(trade_id: Uuid, pick_id: Uuid, direction: TradeDirection, pick_value: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            trade_id,
            pick_id,
            direction,
            pick_value,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeProposal {
    pub trade: PickTrade,
    pub from_team_picks: Vec<Uuid>,
    pub to_team_picks: Vec<Uuid>,
}

impl TradeProposal {
    pub fn new(
        session_id: Uuid,
        from_team_id: Uuid,
        to_team_id: Uuid,
        from_team_picks: Vec<Uuid>,
        to_team_picks: Vec<Uuid>,
        from_team_value: i32,
        to_team_value: i32,
    ) -> DomainResult<Self> {
        Self::validate_picks(&from_team_picks, &to_team_picks)?;

        let trade = PickTrade::new(
            session_id,
            from_team_id,
            to_team_id,
            from_team_value,
            to_team_value,
        )?;

        Ok(Self {
            trade,
            from_team_picks,
            to_team_picks,
        })
    }

    fn validate_picks(from_picks: &[Uuid], to_picks: &[Uuid]) -> DomainResult<()> {
        if from_picks.is_empty() && to_picks.is_empty() {
            return Err(DomainError::ValidationError(
                "Trade must include at least one pick".to_string(),
            ));
        }

        let mut seen = std::collections::HashSet::new();
        for pick_id in from_picks.iter().chain(to_picks.iter()) {
            if !seen.insert(pick_id) {
                return Err(DomainError::ValidationError(
                    format!("Duplicate pick in trade: {}", pick_id)
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pick_trade() {
        let session_id = Uuid::new_v4();
        let from_team = Uuid::new_v4();
        let to_team = Uuid::new_v4();

        let trade = PickTrade::new(session_id, from_team, to_team, 3000, 2900).unwrap();

        assert_eq!(trade.session_id, session_id);
        assert_eq!(trade.from_team_id, from_team);
        assert_eq!(trade.to_team_id, to_team);
        assert_eq!(trade.status, TradeStatus::Proposed);
        assert_eq!(trade.from_team_value, 3000);
        assert_eq!(trade.to_team_value, 2900);
        assert_eq!(trade.value_difference, 100);
        assert!(trade.responded_at.is_none());
    }

    #[test]
    fn test_cannot_trade_with_same_team() {
        let session_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let result = PickTrade::new(session_id, team_id, team_id, 3000, 3000);

        assert!(result.is_err());
        match result {
            Err(DomainError::ValidationError(msg)) => {
                assert_eq!(msg, "Cannot trade with the same team");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_accept_trade() {
        let session_id = Uuid::new_v4();
        let from_team = Uuid::new_v4();
        let to_team = Uuid::new_v4();

        let mut trade = PickTrade::new(session_id, from_team, to_team, 3000, 2900).unwrap();
        assert_eq!(trade.status, TradeStatus::Proposed);

        trade.accept().unwrap();
        assert_eq!(trade.status, TradeStatus::Accepted);
        assert!(trade.responded_at.is_some());
    }

    #[test]
    fn test_cannot_accept_already_accepted_trade() {
        let session_id = Uuid::new_v4();
        let from_team = Uuid::new_v4();
        let to_team = Uuid::new_v4();

        let mut trade = PickTrade::new(session_id, from_team, to_team, 3000, 2900).unwrap();
        trade.accept().unwrap();

        let result = trade.accept();
        assert!(result.is_err());
        match result {
            Err(DomainError::InvalidState(msg)) => {
                assert!(msg.contains("Cannot accept trade in status"));
            }
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[test]
    fn test_reject_trade() {
        let session_id = Uuid::new_v4();
        let from_team = Uuid::new_v4();
        let to_team = Uuid::new_v4();

        let mut trade = PickTrade::new(session_id, from_team, to_team, 3000, 2900).unwrap();
        trade.reject().unwrap();

        assert_eq!(trade.status, TradeStatus::Rejected);
        assert!(trade.responded_at.is_some());
    }

    #[test]
    fn test_cannot_reject_already_rejected_trade() {
        let session_id = Uuid::new_v4();
        let from_team = Uuid::new_v4();
        let to_team = Uuid::new_v4();

        let mut trade = PickTrade::new(session_id, from_team, to_team, 3000, 2900).unwrap();
        trade.reject().unwrap();

        let result = trade.reject();
        assert!(result.is_err());
    }

    #[test]
    fn test_trade_proposal_validates_picks() {
        let session_id = Uuid::new_v4();
        let from_team = Uuid::new_v4();
        let to_team = Uuid::new_v4();

        // Empty picks
        let result = TradeProposal::new(session_id, from_team, to_team, vec![], vec![], 0, 0);
        assert!(result.is_err());

        // Duplicate picks
        let pick_id = Uuid::new_v4();
        let result = TradeProposal::new(
            session_id,
            from_team,
            to_team,
            vec![pick_id],
            vec![pick_id],
            3000,
            3000,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_trade_proposal() {
        let session_id = Uuid::new_v4();
        let from_team = Uuid::new_v4();
        let to_team = Uuid::new_v4();
        let pick1 = Uuid::new_v4();
        let pick2 = Uuid::new_v4();

        let proposal = TradeProposal::new(
            session_id,
            from_team,
            to_team,
            vec![pick1],
            vec![pick2],
            3000,
            2900,
        )
        .unwrap();

        assert_eq!(proposal.from_team_picks.len(), 1);
        assert_eq!(proposal.to_team_picks.len(), 1);
        assert_eq!(proposal.trade.status, TradeStatus::Proposed);
    }
}
