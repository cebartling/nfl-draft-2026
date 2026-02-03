use std::sync::Arc;
use uuid::Uuid;
use crate::errors::{DomainError, DomainResult};
use crate::models::{ChartType, PickTrade, TradeProposal};
use crate::repositories::{DraftPickRepository, TeamRepository, TradeRepository};
use crate::services::trade_value::TradeValueChart;

pub struct TradeEngine {
    trade_repo: Arc<dyn TradeRepository>,
    pick_repo: Arc<dyn DraftPickRepository>,
    team_repo: Arc<dyn TeamRepository>,
    default_chart_type: ChartType,
    fairness_threshold_percent: i32,  // Default: 15%
}

impl TradeEngine {
    pub fn new(
        trade_repo: Arc<dyn TradeRepository>,
        pick_repo: Arc<dyn DraftPickRepository>,
        team_repo: Arc<dyn TeamRepository>,
        default_chart_type: ChartType,
    ) -> Self {
        Self {
            trade_repo,
            pick_repo,
            team_repo,
            default_chart_type,
            fairness_threshold_percent: 15,
        }
    }

    /// Create with default Jimmy Johnson chart
    pub fn with_default_chart(
        trade_repo: Arc<dyn TradeRepository>,
        pick_repo: Arc<dyn DraftPickRepository>,
        team_repo: Arc<dyn TeamRepository>,
    ) -> Self {
        Self::new(trade_repo, pick_repo, team_repo, ChartType::JimmyJohnson)
    }

    /// Propose a trade with value validation using specified chart type
    pub async fn propose_trade(
        &self,
        session_id: Uuid,
        from_team_id: Uuid,
        to_team_id: Uuid,
        from_team_picks: Vec<Uuid>,
        to_team_picks: Vec<Uuid>,
        chart_type: Option<ChartType>,
    ) -> DomainResult<TradeProposal> {
        // Validate teams exist
        self.validate_team_exists(from_team_id).await?;
        self.validate_team_exists(to_team_id).await?;

        // Validate picks ownership and availability (no trade to exclude)
        self.validate_picks_for_trade(
            from_team_id,
            to_team_id,
            &from_team_picks,
            &to_team_picks,
            None,
        ).await?;

        // Create chart instance for this trade
        let chart_type = chart_type.unwrap_or(self.default_chart_type);
        let value_chart = chart_type.create_chart();

        // Calculate trade values
        let from_team_value = self.calculate_total_value_with_chart(&from_team_picks, &*value_chart).await?;
        let to_team_value = self.calculate_total_value_with_chart(&to_team_picks, &*value_chart).await?;

        // Validate trade fairness
        if !value_chart.is_trade_fair(
            from_team_value,
            to_team_value,
            self.fairness_threshold_percent,
        ) {
            return Err(DomainError::ValidationError(format!(
                "Trade is not fair using {} chart: {} points vs {} points (threshold: {}%)",
                value_chart.name(), from_team_value, to_team_value, self.fairness_threshold_percent
            )));
        }

        // Create proposal
        let proposal = TradeProposal::new(
            session_id,
            from_team_id,
            to_team_id,
            from_team_picks,
            to_team_picks,
            from_team_value,
            to_team_value,
        )?;

        // Save to database
        self.trade_repo.create_trade(&proposal).await
    }

    /// Accept trade and auto-execute (transfer picks)
    pub async fn accept_trade(&self, trade_id: Uuid, accepting_team_id: Uuid) -> DomainResult<PickTrade> {
        // Get trade with details
        let mut trade_proposal = self.trade_repo.find_trade_with_details(trade_id).await?
            .ok_or_else(|| DomainError::NotFound(format!("Trade {} not found", trade_id)))?;

        // Verify accepting team is to_team
        if trade_proposal.trade.to_team_id != accepting_team_id {
            return Err(DomainError::ValidationError(
                "Only the receiving team can accept a trade".to_string(),
            ));
        }

        // Accept trade
        trade_proposal.trade.accept()?;

        // Re-validate picks before execution (prevent race conditions, excluding current trade)
        self.validate_picks_for_trade(
            trade_proposal.trade.from_team_id,
            trade_proposal.trade.to_team_id,
            &trade_proposal.from_team_picks,
            &trade_proposal.to_team_picks,
            Some(trade_id),
        ).await?;

        // Execute trade (atomic pick transfer)
        self.trade_repo.transfer_picks(
            trade_proposal.trade.from_team_id,
            trade_proposal.trade.to_team_id,
            &trade_proposal.from_team_picks,
            &trade_proposal.to_team_picks,
        ).await?;

        // Update trade status to Accepted
        self.trade_repo.update(&trade_proposal.trade).await
    }

    /// Reject a trade
    pub async fn reject_trade(&self, trade_id: Uuid, rejecting_team_id: Uuid) -> DomainResult<PickTrade> {
        let mut trade = self.trade_repo.find_by_id(trade_id).await?
            .ok_or_else(|| DomainError::NotFound(format!("Trade {} not found", trade_id)))?;

        if trade.to_team_id != rejecting_team_id {
            return Err(DomainError::ValidationError(
                "Only the receiving team can reject a trade".to_string(),
            ));
        }

        trade.reject()?;
        self.trade_repo.update(&trade).await
    }

    /// Get pending trades for a team
    pub async fn get_pending_trades(&self, team_id: Uuid) -> DomainResult<Vec<TradeProposal>> {
        self.trade_repo.find_pending_for_team(team_id).await
    }

    /// Get trade by ID with details
    pub async fn get_trade(&self, trade_id: Uuid) -> DomainResult<Option<TradeProposal>> {
        self.trade_repo.find_trade_with_details(trade_id).await
    }

    // --- Private helper methods ---

    async fn validate_team_exists(&self, team_id: Uuid) -> DomainResult<()> {
        self.team_repo.find_by_id(team_id).await?
            .ok_or_else(|| DomainError::NotFound(format!("Team {} not found", team_id)))?;
        Ok(())
    }

    async fn validate_picks_for_trade(
        &self,
        from_team_id: Uuid,
        to_team_id: Uuid,
        from_team_picks: &[Uuid],
        to_team_picks: &[Uuid],
        exclude_trade_id: Option<Uuid>,
    ) -> DomainResult<()> {
        for pick_id in from_team_picks {
            self.validate_pick_ownership(*pick_id, from_team_id).await?;
            self.validate_pick_not_traded(*pick_id, exclude_trade_id).await?;
        }

        for pick_id in to_team_picks {
            self.validate_pick_ownership(*pick_id, to_team_id).await?;
            self.validate_pick_not_traded(*pick_id, exclude_trade_id).await?;
        }

        Ok(())
    }

    async fn validate_pick_ownership(&self, pick_id: Uuid, expected_team_id: Uuid) -> DomainResult<()> {
        let pick = self.pick_repo.find_by_id(pick_id).await?
            .ok_or_else(|| DomainError::NotFound(format!("Pick {} not found", pick_id)))?;

        if pick.team_id != expected_team_id {
            return Err(DomainError::ValidationError(
                format!("Pick {} is not owned by team {}", pick_id, expected_team_id)
            ));
        }

        if pick.is_picked() {
            return Err(DomainError::ValidationError(
                format!("Pick {} has already been used", pick_id)
            ));
        }

        Ok(())
    }

    async fn validate_pick_not_traded(&self, pick_id: Uuid, exclude_trade_id: Option<Uuid>) -> DomainResult<()> {
        if self.trade_repo.is_pick_in_active_trade(pick_id, exclude_trade_id).await? {
            return Err(DomainError::ValidationError(
                format!("Pick {} is already in an active trade", pick_id)
            ));
        }
        Ok(())
    }

    async fn calculate_total_value_with_chart(
        &self,
        pick_ids: &[Uuid],
        value_chart: &dyn TradeValueChart,
    ) -> DomainResult<i32> {
        let mut total_value = 0;

        for pick_id in pick_ids {
            let pick = self.pick_repo.find_by_id(*pick_id).await?
                .ok_or_else(|| DomainError::NotFound(format!("Pick {} not found", pick_id)))?;

            let value = value_chart.calculate_pick_value(pick.overall_pick)?;
            total_value += value;
        }

        Ok(total_value)
    }
}

// Tests will be covered by acceptance tests in the API crate
