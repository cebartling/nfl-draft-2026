use crate::errors::{DomainError, DomainResult};
use crate::models::{ChartType, PickTrade, TradeProposal};
use crate::repositories::{DraftPickRepository, TeamRepository, TradeRepository};
use crate::services::trade_value::TradeValueChart;
use std::sync::Arc;
use uuid::Uuid;

pub struct TradeEngine {
    trade_repo: Arc<dyn TradeRepository>,
    pick_repo: Arc<dyn DraftPickRepository>,
    team_repo: Arc<dyn TeamRepository>,
    default_chart_type: ChartType,
    fairness_threshold_percent: i32, // Default: 15%
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
        )
        .await?;

        // Create chart instance for this trade
        let chart_type = chart_type.unwrap_or(self.default_chart_type);
        let value_chart = chart_type.create_chart();

        // Calculate trade values
        let from_team_value = self
            .calculate_total_value_with_chart(&from_team_picks, &*value_chart)
            .await?;
        let to_team_value = self
            .calculate_total_value_with_chart(&to_team_picks, &*value_chart)
            .await?;

        // Validate trade fairness
        if !value_chart.is_trade_fair(
            from_team_value,
            to_team_value,
            self.fairness_threshold_percent,
        ) {
            return Err(DomainError::ValidationError(format!(
                "Trade is not fair using {} chart: {} points vs {} points (threshold: {}%)",
                value_chart.name(),
                from_team_value,
                to_team_value,
                self.fairness_threshold_percent
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

        // Save to database, passing the chart type used for value calculation
        self.trade_repo.create_trade(&proposal, chart_type).await
    }

    /// Accept trade and auto-execute (transfer picks)
    pub async fn accept_trade(
        &self,
        trade_id: Uuid,
        accepting_team_id: Uuid,
    ) -> DomainResult<PickTrade> {
        // Get trade with details
        let mut trade_proposal = self
            .trade_repo
            .find_trade_with_details(trade_id)
            .await?
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
        )
        .await?;

        // Execute trade (atomic pick transfer)
        self.trade_repo
            .transfer_picks(
                trade_proposal.trade.from_team_id,
                trade_proposal.trade.to_team_id,
                &trade_proposal.from_team_picks,
                &trade_proposal.to_team_picks,
            )
            .await?;

        // Update trade status to Accepted
        self.trade_repo.update(&trade_proposal.trade).await
    }

    /// Reject a trade
    pub async fn reject_trade(
        &self,
        trade_id: Uuid,
        rejecting_team_id: Uuid,
    ) -> DomainResult<PickTrade> {
        let mut trade = self
            .trade_repo
            .find_by_id(trade_id)
            .await?
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
        self.team_repo
            .find_by_id(team_id)
            .await?
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
            self.validate_pick_not_traded(*pick_id, exclude_trade_id)
                .await?;
        }

        for pick_id in to_team_picks {
            self.validate_pick_ownership(*pick_id, to_team_id).await?;
            self.validate_pick_not_traded(*pick_id, exclude_trade_id)
                .await?;
        }

        Ok(())
    }

    async fn validate_pick_ownership(
        &self,
        pick_id: Uuid,
        expected_team_id: Uuid,
    ) -> DomainResult<()> {
        let pick = self
            .pick_repo
            .find_by_id(pick_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Pick {} not found", pick_id)))?;

        if pick.team_id != expected_team_id {
            return Err(DomainError::ValidationError(format!(
                "Pick {} is not owned by team {}",
                pick_id, expected_team_id
            )));
        }

        if pick.is_picked() {
            return Err(DomainError::ValidationError(format!(
                "Pick {} has already been used",
                pick_id
            )));
        }

        Ok(())
    }

    async fn validate_pick_not_traded(
        &self,
        pick_id: Uuid,
        exclude_trade_id: Option<Uuid>,
    ) -> DomainResult<()> {
        if self
            .trade_repo
            .is_pick_in_active_trade(pick_id, exclude_trade_id)
            .await?
        {
            return Err(DomainError::ValidationError(format!(
                "Pick {} is already in an active trade",
                pick_id
            )));
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
            let pick = self
                .pick_repo
                .find_by_id(*pick_id)
                .await?
                .ok_or_else(|| DomainError::NotFound(format!("Pick {} not found", pick_id)))?;

            let value = value_chart.calculate_pick_value(pick.overall_pick)?;
            total_value += value;
        }

        Ok(total_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Conference, Division, DraftPick, PickTrade, Team, TradeProposal};
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        TradeRepo {}
        #[async_trait::async_trait]
        impl TradeRepository for TradeRepo {
            async fn create_trade(&self, proposal: &TradeProposal, chart_type: ChartType) -> DomainResult<TradeProposal>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<PickTrade>>;
            async fn find_trade_with_details(&self, id: Uuid) -> DomainResult<Option<TradeProposal>>;
            async fn find_by_session(&self, session_id: Uuid) -> DomainResult<Vec<PickTrade>>;
            async fn find_pending_for_team(&self, team_id: Uuid) -> DomainResult<Vec<TradeProposal>>;
            async fn update(&self, trade: &PickTrade) -> DomainResult<PickTrade>;
            async fn is_pick_in_active_trade(&self, pick_id: Uuid, exclude_trade_id: Option<Uuid>) -> DomainResult<bool>;
            async fn transfer_picks(&self, from_team_id: Uuid, to_team_id: Uuid, from_team_picks: &[Uuid], to_team_picks: &[Uuid]) -> DomainResult<()>;
        }
    }

    mock! {
        DraftPickRepo {}
        #[async_trait::async_trait]
        impl DraftPickRepository for DraftPickRepo {
            async fn create(&self, pick: &DraftPick) -> DomainResult<DraftPick>;
            async fn create_many(&self, picks: &[DraftPick]) -> DomainResult<Vec<DraftPick>>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftPick>>;
            async fn find_by_draft_id(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>>;
            async fn find_by_draft_and_round(&self, draft_id: Uuid, round: i32) -> DomainResult<Vec<DraftPick>>;
            async fn find_by_draft_and_team(&self, draft_id: Uuid, team_id: Uuid) -> DomainResult<Vec<DraftPick>>;
            async fn find_next_pick(&self, draft_id: Uuid) -> DomainResult<Option<DraftPick>>;
            async fn find_available_picks(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>>;
            async fn update(&self, pick: &DraftPick) -> DomainResult<DraftPick>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
            async fn delete_by_draft_id(&self, draft_id: Uuid) -> DomainResult<()>;
        }
    }

    mock! {
        TeamRepo {}
        #[async_trait::async_trait]
        impl TeamRepository for TeamRepo {
            async fn create(&self, team: &Team) -> DomainResult<Team>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Team>>;
            async fn find_by_abbreviation(&self, abbreviation: &str) -> DomainResult<Option<Team>>;
            async fn find_all(&self) -> DomainResult<Vec<Team>>;
            async fn update(&self, team: &Team) -> DomainResult<Team>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
        }
    }

    fn make_team(name: &str, abbr: &str) -> Team {
        Team::new(
            name.to_string(),
            abbr.to_string(),
            "City".to_string(),
            Conference::AFC,
            Division::AFCEast,
        )
        .unwrap()
    }

    fn make_pick(team_id: Uuid, overall_pick: i32) -> DraftPick {
        DraftPick::new(Uuid::new_v4(), 1, overall_pick, overall_pick, team_id).unwrap()
    }

    fn setup_engine(
        trade_repo: MockTradeRepo,
        pick_repo: MockDraftPickRepo,
        team_repo: MockTeamRepo,
    ) -> TradeEngine {
        TradeEngine::new(
            Arc::new(trade_repo),
            Arc::new(pick_repo),
            Arc::new(team_repo),
            ChartType::JimmyJohnson,
        )
    }

    // --- propose_trade tests ---

    #[tokio::test]
    async fn test_propose_trade_success() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let pick_a = make_pick(team_a.id, 1); // Pick 1 = 3000 pts
        let pick_b = make_pick(team_b.id, 2); // Pick 2 = 2600 pts

        let team_a_id = team_a.id;
        let team_b_id = team_b.id;
        let pick_a_id = pick_a.id;
        let pick_b_id = pick_b.id;

        let mut team_repo = MockTeamRepo::new();
        let team_a_clone = team_a.clone();
        let team_b_clone = team_b.clone();
        team_repo
            .expect_find_by_id()
            .with(eq(team_a_id))
            .returning(move |_| Ok(Some(team_a_clone.clone())));
        team_repo
            .expect_find_by_id()
            .with(eq(team_b_id))
            .returning(move |_| Ok(Some(team_b_clone.clone())));

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_a_clone = pick_a.clone();
        let pick_b_clone = pick_b.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_a_id))
            .returning(move |_| Ok(Some(pick_a_clone.clone())));
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_b_id))
            .returning(move |_| Ok(Some(pick_b_clone.clone())));

        let mut trade_repo = MockTradeRepo::new();
        trade_repo
            .expect_is_pick_in_active_trade()
            .returning(|_, _| Ok(false));
        trade_repo
            .expect_create_trade()
            .returning(|proposal, _| Ok(proposal.clone()));

        let engine = setup_engine(trade_repo, pick_repo, team_repo);
        let session_id = Uuid::new_v4();

        let result = engine
            .propose_trade(
                session_id,
                team_a_id,
                team_b_id,
                vec![pick_a_id],
                vec![pick_b_id],
                None,
            )
            .await;

        assert!(result.is_ok());
        let proposal = result.unwrap();
        assert_eq!(proposal.trade.from_team_id, team_a_id);
        assert_eq!(proposal.trade.to_team_id, team_b_id);
    }

    #[tokio::test]
    async fn test_propose_trade_from_team_not_found() {
        let mut team_repo = MockTeamRepo::new();
        let from_team_id = Uuid::new_v4();
        team_repo
            .expect_find_by_id()
            .with(eq(from_team_id))
            .returning(|_| Ok(None));

        let engine = setup_engine(MockTradeRepo::new(), MockDraftPickRepo::new(), team_repo);

        let result = engine
            .propose_trade(
                Uuid::new_v4(),
                from_team_id,
                Uuid::new_v4(),
                vec![Uuid::new_v4()],
                vec![Uuid::new_v4()],
                None,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_propose_trade_to_team_not_found() {
        let team_a = make_team("Team A", "TMA");
        let team_a_id = team_a.id;
        let to_team_id = Uuid::new_v4();

        let mut team_repo = MockTeamRepo::new();
        team_repo
            .expect_find_by_id()
            .with(eq(team_a_id))
            .returning(move |_| Ok(Some(team_a.clone())));
        team_repo
            .expect_find_by_id()
            .with(eq(to_team_id))
            .returning(|_| Ok(None));

        let engine = setup_engine(MockTradeRepo::new(), MockDraftPickRepo::new(), team_repo);

        let result = engine
            .propose_trade(
                Uuid::new_v4(),
                team_a_id,
                to_team_id,
                vec![Uuid::new_v4()],
                vec![Uuid::new_v4()],
                None,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_propose_trade_pick_not_owned() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let other_team = make_team("Other", "OTH");
        // Pick is owned by other_team, not team_a
        let pick_a = make_pick(other_team.id, 1);

        let team_a_id = team_a.id;
        let team_b_id = team_b.id;
        let pick_a_id = pick_a.id;

        let mut team_repo = MockTeamRepo::new();
        let team_a_c = team_a.clone();
        let team_b_c = team_b.clone();
        team_repo
            .expect_find_by_id()
            .with(eq(team_a_id))
            .returning(move |_| Ok(Some(team_a_c.clone())));
        team_repo
            .expect_find_by_id()
            .with(eq(team_b_id))
            .returning(move |_| Ok(Some(team_b_c.clone())));

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_a_c = pick_a.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_a_id))
            .returning(move |_| Ok(Some(pick_a_c.clone())));

        let engine = setup_engine(MockTradeRepo::new(), pick_repo, team_repo);

        let result = engine
            .propose_trade(
                Uuid::new_v4(),
                team_a_id,
                team_b_id,
                vec![pick_a_id],
                vec![Uuid::new_v4()],
                None,
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => assert!(msg.contains("not owned by")),
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_propose_trade_pick_already_used() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let mut pick_a = make_pick(team_a.id, 1);
        pick_a.make_pick(Uuid::new_v4()).unwrap(); // Mark as used

        let team_a_id = team_a.id;
        let team_b_id = team_b.id;
        let pick_a_id = pick_a.id;

        let mut team_repo = MockTeamRepo::new();
        let team_a_c = team_a.clone();
        let team_b_c = team_b.clone();
        team_repo
            .expect_find_by_id()
            .with(eq(team_a_id))
            .returning(move |_| Ok(Some(team_a_c.clone())));
        team_repo
            .expect_find_by_id()
            .with(eq(team_b_id))
            .returning(move |_| Ok(Some(team_b_c.clone())));

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_a_c = pick_a.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_a_id))
            .returning(move |_| Ok(Some(pick_a_c.clone())));

        let engine = setup_engine(MockTradeRepo::new(), pick_repo, team_repo);

        let result = engine
            .propose_trade(
                Uuid::new_v4(),
                team_a_id,
                team_b_id,
                vec![pick_a_id],
                vec![Uuid::new_v4()],
                None,
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => assert!(msg.contains("already been used")),
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_propose_trade_pick_in_active_trade() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let pick_a = make_pick(team_a.id, 1);

        let team_a_id = team_a.id;
        let team_b_id = team_b.id;
        let pick_a_id = pick_a.id;

        let mut team_repo = MockTeamRepo::new();
        let team_a_c = team_a.clone();
        let team_b_c = team_b.clone();
        team_repo
            .expect_find_by_id()
            .with(eq(team_a_id))
            .returning(move |_| Ok(Some(team_a_c.clone())));
        team_repo
            .expect_find_by_id()
            .with(eq(team_b_id))
            .returning(move |_| Ok(Some(team_b_c.clone())));

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_a_c = pick_a.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_a_id))
            .returning(move |_| Ok(Some(pick_a_c.clone())));

        let mut trade_repo = MockTradeRepo::new();
        trade_repo
            .expect_is_pick_in_active_trade()
            .with(eq(pick_a_id), eq(None))
            .returning(|_, _| Ok(true));

        let engine = setup_engine(trade_repo, pick_repo, team_repo);

        let result = engine
            .propose_trade(
                Uuid::new_v4(),
                team_a_id,
                team_b_id,
                vec![pick_a_id],
                vec![Uuid::new_v4()],
                None,
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => assert!(msg.contains("already in an active trade")),
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_propose_trade_unfair_rejected() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let pick_a = make_pick(team_a.id, 1);   // Pick 1 = 3000 pts (Jimmy Johnson)
        let pick_b = make_pick(team_b.id, 32);  // Pick 32 = 590 pts — huge gap

        let team_a_id = team_a.id;
        let team_b_id = team_b.id;
        let pick_a_id = pick_a.id;
        let pick_b_id = pick_b.id;

        let mut team_repo = MockTeamRepo::new();
        let team_a_c = team_a.clone();
        let team_b_c = team_b.clone();
        team_repo
            .expect_find_by_id()
            .with(eq(team_a_id))
            .returning(move |_| Ok(Some(team_a_c.clone())));
        team_repo
            .expect_find_by_id()
            .with(eq(team_b_id))
            .returning(move |_| Ok(Some(team_b_c.clone())));

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_a_c = pick_a.clone();
        let pick_b_c = pick_b.clone();
        // find_by_id called for ownership validation and value calculation
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_a_id))
            .returning(move |_| Ok(Some(pick_a_c.clone())));
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_b_id))
            .returning(move |_| Ok(Some(pick_b_c.clone())));

        let mut trade_repo = MockTradeRepo::new();
        trade_repo
            .expect_is_pick_in_active_trade()
            .returning(|_, _| Ok(false));

        let engine = setup_engine(trade_repo, pick_repo, team_repo);

        let result = engine
            .propose_trade(
                Uuid::new_v4(),
                team_a_id,
                team_b_id,
                vec![pick_a_id],
                vec![pick_b_id],
                None,
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => assert!(msg.contains("not fair")),
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_propose_trade_uses_default_chart() {
        // Use RichHill as default, verify it is used
        // Picks 10 and 11 are close in value on all charts
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let pick_a = make_pick(team_a.id, 10);
        let pick_b = make_pick(team_b.id, 11);

        let team_a_id = team_a.id;
        let team_b_id = team_b.id;
        let pick_a_id = pick_a.id;
        let pick_b_id = pick_b.id;

        let mut team_repo = MockTeamRepo::new();
        let team_a_c = team_a.clone();
        let team_b_c = team_b.clone();
        team_repo
            .expect_find_by_id()
            .with(eq(team_a_id))
            .returning(move |_| Ok(Some(team_a_c.clone())));
        team_repo
            .expect_find_by_id()
            .with(eq(team_b_id))
            .returning(move |_| Ok(Some(team_b_c.clone())));

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_a_c = pick_a.clone();
        let pick_b_c = pick_b.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_a_id))
            .returning(move |_| Ok(Some(pick_a_c.clone())));
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_b_id))
            .returning(move |_| Ok(Some(pick_b_c.clone())));

        let mut trade_repo = MockTradeRepo::new();
        trade_repo
            .expect_is_pick_in_active_trade()
            .returning(|_, _| Ok(false));
        // Verify that RichHill chart type is passed to create_trade
        trade_repo
            .expect_create_trade()
            .withf(|_, chart_type| *chart_type == ChartType::RichHill)
            .returning(|proposal, _| Ok(proposal.clone()));

        // Engine with RichHill default
        let engine = TradeEngine::new(
            Arc::new(trade_repo),
            Arc::new(pick_repo),
            Arc::new(team_repo),
            ChartType::RichHill,
        );

        let result = engine
            .propose_trade(
                Uuid::new_v4(),
                team_a_id,
                team_b_id,
                vec![pick_a_id],
                vec![pick_b_id],
                None, // No chart specified → uses default RichHill
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_propose_trade_uses_specified_chart() {
        // Picks 10 and 11 are close in value on all charts
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let pick_a = make_pick(team_a.id, 10);
        let pick_b = make_pick(team_b.id, 11);

        let team_a_id = team_a.id;
        let team_b_id = team_b.id;
        let pick_a_id = pick_a.id;
        let pick_b_id = pick_b.id;

        let mut team_repo = MockTeamRepo::new();
        let team_a_c = team_a.clone();
        let team_b_c = team_b.clone();
        team_repo
            .expect_find_by_id()
            .with(eq(team_a_id))
            .returning(move |_| Ok(Some(team_a_c.clone())));
        team_repo
            .expect_find_by_id()
            .with(eq(team_b_id))
            .returning(move |_| Ok(Some(team_b_c.clone())));

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_a_c = pick_a.clone();
        let pick_b_c = pick_b.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_a_id))
            .returning(move |_| Ok(Some(pick_a_c.clone())));
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_b_id))
            .returning(move |_| Ok(Some(pick_b_c.clone())));

        let mut trade_repo = MockTradeRepo::new();
        trade_repo
            .expect_is_pick_in_active_trade()
            .returning(|_, _| Ok(false));
        // Verify RichHill chart type is passed even though default is JimmyJohnson
        trade_repo
            .expect_create_trade()
            .withf(|_, chart_type| *chart_type == ChartType::RichHill)
            .returning(|proposal, _| Ok(proposal.clone()));

        let engine = setup_engine(trade_repo, pick_repo, team_repo);

        let result = engine
            .propose_trade(
                Uuid::new_v4(),
                team_a_id,
                team_b_id,
                vec![pick_a_id],
                vec![pick_b_id],
                Some(ChartType::RichHill), // Explicitly specified
            )
            .await;

        assert!(result.is_ok());
    }

    // --- accept_trade tests ---

    #[tokio::test]
    async fn test_accept_trade_success() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let pick_a = make_pick(team_a.id, 1);
        let pick_b = make_pick(team_b.id, 2);

        let team_b_id = team_b.id;
        let pick_a_id = pick_a.id;
        let pick_b_id = pick_b.id;

        let proposal = TradeProposal::new(
            Uuid::new_v4(),
            team_a.id,
            team_b.id,
            vec![pick_a_id],
            vec![pick_b_id],
            3000,
            2600,
        )
        .unwrap();
        let trade_id = proposal.trade.id;

        let mut trade_repo = MockTradeRepo::new();
        let proposal_c = proposal.clone();
        trade_repo
            .expect_find_trade_with_details()
            .with(eq(trade_id))
            .returning(move |_| Ok(Some(proposal_c.clone())));
        trade_repo
            .expect_is_pick_in_active_trade()
            .returning(|_, _| Ok(false));
        trade_repo
            .expect_transfer_picks()
            .returning(|_, _, _, _| Ok(()));
        trade_repo
            .expect_update()
            .returning(|trade| Ok(trade.clone()));

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_a_c = pick_a.clone();
        let pick_b_c = pick_b.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_a_id))
            .returning(move |_| Ok(Some(pick_a_c.clone())));
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_b_id))
            .returning(move |_| Ok(Some(pick_b_c.clone())));

        let engine = setup_engine(trade_repo, pick_repo, MockTeamRepo::new());

        let result = engine.accept_trade(trade_id, team_b_id).await;

        assert!(result.is_ok());
        let trade = result.unwrap();
        assert_eq!(trade.status, crate::models::TradeStatus::Accepted);
    }

    #[tokio::test]
    async fn test_accept_trade_not_found() {
        let trade_id = Uuid::new_v4();

        let mut trade_repo = MockTradeRepo::new();
        trade_repo
            .expect_find_trade_with_details()
            .with(eq(trade_id))
            .returning(|_| Ok(None));

        let engine = setup_engine(trade_repo, MockDraftPickRepo::new(), MockTeamRepo::new());

        let result = engine.accept_trade(trade_id, Uuid::new_v4()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_accept_trade_wrong_team() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let wrong_team_id = Uuid::new_v4();

        let proposal = TradeProposal::new(
            Uuid::new_v4(),
            team_a.id,
            team_b.id,
            vec![Uuid::new_v4()],
            vec![Uuid::new_v4()],
            3000,
            2600,
        )
        .unwrap();
        let trade_id = proposal.trade.id;

        let mut trade_repo = MockTradeRepo::new();
        trade_repo
            .expect_find_trade_with_details()
            .with(eq(trade_id))
            .returning(move |_| Ok(Some(proposal.clone())));

        let engine = setup_engine(trade_repo, MockDraftPickRepo::new(), MockTeamRepo::new());

        let result = engine.accept_trade(trade_id, wrong_team_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => {
                assert!(msg.contains("Only the receiving team can accept"))
            }
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_accept_trade_revalidation_fails() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let pick_a = make_pick(team_a.id, 1);
        let pick_b = make_pick(team_b.id, 2);

        let team_b_id = team_b.id;
        let pick_a_id = pick_a.id;

        let proposal = TradeProposal::new(
            Uuid::new_v4(),
            team_a.id,
            team_b.id,
            vec![pick_a_id],
            vec![pick_b.id],
            3000,
            2600,
        )
        .unwrap();
        let trade_id = proposal.trade.id;

        let mut trade_repo = MockTradeRepo::new();
        let proposal_c = proposal.clone();
        trade_repo
            .expect_find_trade_with_details()
            .with(eq(trade_id))
            .returning(move |_| Ok(Some(proposal_c.clone())));
        // Pick is now in another active trade (revalidation fails)
        trade_repo
            .expect_is_pick_in_active_trade()
            .with(eq(pick_a_id), eq(Some(trade_id)))
            .returning(|_, _| Ok(true));

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_a_c = pick_a.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_a_id))
            .returning(move |_| Ok(Some(pick_a_c.clone())));

        let engine = setup_engine(trade_repo, pick_repo, MockTeamRepo::new());

        let result = engine.accept_trade(trade_id, team_b_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => {
                assert!(msg.contains("already in an active trade"))
            }
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }

    // --- reject_trade tests ---

    #[tokio::test]
    async fn test_reject_trade_success() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");

        let trade =
            PickTrade::new(Uuid::new_v4(), team_a.id, team_b.id, 3000, 2600).unwrap();
        let trade_id = trade.id;
        let team_b_id = team_b.id;

        let mut trade_repo = MockTradeRepo::new();
        trade_repo
            .expect_find_by_id()
            .with(eq(trade_id))
            .returning(move |_| Ok(Some(trade.clone())));
        trade_repo
            .expect_update()
            .returning(|trade| Ok(trade.clone()));

        let engine = setup_engine(trade_repo, MockDraftPickRepo::new(), MockTeamRepo::new());

        let result = engine.reject_trade(trade_id, team_b_id).await;

        assert!(result.is_ok());
        let rejected = result.unwrap();
        assert_eq!(rejected.status, crate::models::TradeStatus::Rejected);
    }

    #[tokio::test]
    async fn test_reject_trade_wrong_team() {
        let team_a = make_team("Team A", "TMA");
        let team_b = make_team("Team B", "TMB");
        let wrong_team_id = Uuid::new_v4();

        let trade =
            PickTrade::new(Uuid::new_v4(), team_a.id, team_b.id, 3000, 2600).unwrap();
        let trade_id = trade.id;

        let mut trade_repo = MockTradeRepo::new();
        trade_repo
            .expect_find_by_id()
            .with(eq(trade_id))
            .returning(move |_| Ok(Some(trade.clone())));

        let engine = setup_engine(trade_repo, MockDraftPickRepo::new(), MockTeamRepo::new());

        let result = engine.reject_trade(trade_id, wrong_team_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => {
                assert!(msg.contains("Only the receiving team can reject"))
            }
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }
}
