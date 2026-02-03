use async_trait::async_trait;
use uuid::Uuid;
use crate::errors::DomainResult;
use crate::models::{PickTrade, TradeProposal};

#[async_trait]
pub trait TradeRepository: Send + Sync {
    /// Create trade with details (atomic transaction)
    async fn create_trade(&self, proposal: &TradeProposal) -> DomainResult<TradeProposal>;

    /// Find trade by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<PickTrade>>;

    /// Find trade with full details
    async fn find_trade_with_details(&self, id: Uuid) -> DomainResult<Option<TradeProposal>>;

    /// Get all trades for a session
    async fn find_by_session(&self, session_id: Uuid) -> DomainResult<Vec<PickTrade>>;

    /// Get pending trades for a team (awaiting their response)
    async fn find_pending_for_team(&self, team_id: Uuid) -> DomainResult<Vec<TradeProposal>>;

    /// Update trade status
    async fn update(&self, trade: &PickTrade) -> DomainResult<PickTrade>;

    /// Check if pick is in any active (Proposed) trade
    async fn is_pick_in_active_trade(&self, pick_id: Uuid) -> DomainResult<bool>;

    /// Transfer pick ownership (atomic)
    async fn transfer_picks(
        &self,
        from_team_id: Uuid,
        to_team_id: Uuid,
        from_team_picks: &[Uuid],
        to_team_picks: &[Uuid],
    ) -> DomainResult<()>;
}
