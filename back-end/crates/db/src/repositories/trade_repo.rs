use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use domain::errors::DomainResult;
use domain::models::{PickTrade, PickTradeDetail, TradeDirection, TradeProposal};
use domain::repositories::TradeRepository;
use domain::services::ChartType;
use crate::errors::DbError;
use crate::models::{PickTradeDb, PickTradeDetailDb};

pub struct SqlxTradeRepository {
    pool: PgPool,
}

impl SqlxTradeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TradeRepository for SqlxTradeRepository {
    async fn create_trade(&self, proposal: &TradeProposal) -> DomainResult<TradeProposal> {
        let trade_db = PickTradeDb::from_domain(&proposal.trade);

        // Transaction for atomic creation
        let mut tx = self.pool.begin().await.map_err(DbError::DatabaseError)?;

        // Insert trade
        let trade_result = sqlx::query_as!(
            PickTradeDb,
            r#"
            INSERT INTO pick_trades (
                id, session_id, from_team_id, to_team_id, status,
                from_team_value, to_team_value, value_difference,
                proposed_at, responded_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, session_id, from_team_id, to_team_id, status,
                      from_team_value, to_team_value, value_difference,
                      proposed_at, responded_at, created_at, updated_at
            "#,
            trade_db.id,
            trade_db.session_id,
            trade_db.from_team_id,
            trade_db.to_team_id,
            trade_db.status,
            trade_db.from_team_value,
            trade_db.to_team_value,
            trade_db.value_difference,
            trade_db.proposed_at,
            trade_db.responded_at,
            trade_db.created_at,
            trade_db.updated_at
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(DbError::DatabaseError)?;

        // Use Jimmy Johnson chart for calculating pick values
        let value_chart = ChartType::JimmyJohnson.create_chart();

        // Insert trade details for each pick
        for (pick_id, direction) in proposal.from_team_picks.iter()
            .map(|id| (*id, TradeDirection::FromTeam))
            .chain(proposal.to_team_picks.iter().map(|id| (*id, TradeDirection::ToTeam)))
        {
            // Get pick to determine value
            let pick = sqlx::query!("SELECT overall_pick FROM draft_picks WHERE id = $1", pick_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(DbError::DatabaseError)?;

            // Calculate value using trade value chart
            let value = value_chart.calculate_pick_value(pick.overall_pick)
                .map_err(|e| DbError::MappingError(format!("Failed to calculate pick value: {:?}", e)))?;

            let detail = PickTradeDetail::new(proposal.trade.id, pick_id, direction, value);
            let detail_db = PickTradeDetailDb::from_domain(&detail);

            sqlx::query!(
                r#"
                INSERT INTO pick_trade_details (id, trade_id, pick_id, direction, pick_value, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                detail_db.id,
                detail_db.trade_id,
                detail_db.pick_id,
                detail_db.direction,
                detail_db.pick_value,
                detail_db.created_at
            )
            .execute(&mut *tx)
            .await
            .map_err(DbError::DatabaseError)?;
        }

        tx.commit().await.map_err(DbError::DatabaseError)?;

        Ok(TradeProposal {
            trade: trade_result.to_domain()?,
            from_team_picks: proposal.from_team_picks.clone(),
            to_team_picks: proposal.to_team_picks.clone(),
        })
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<PickTrade>> {
        let result = sqlx::query_as!(
            PickTradeDb,
            r#"
            SELECT id, session_id, from_team_id, to_team_id, status,
                   from_team_value, to_team_value, value_difference,
                   proposed_at, responded_at, created_at, updated_at
            FROM pick_trades
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(db) => Ok(Some(db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_trade_with_details(&self, id: Uuid) -> DomainResult<Option<TradeProposal>> {
        let trade = match self.find_by_id(id).await? {
            Some(t) => t,
            None => return Ok(None),
        };

        let details = sqlx::query_as!(
            PickTradeDetailDb,
            r#"
            SELECT id, trade_id, pick_id, direction, pick_value, created_at
            FROM pick_trade_details
            WHERE trade_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        let mut from_team_picks = Vec::new();
        let mut to_team_picks = Vec::new();

        for detail_db in details {
            let detail = detail_db.to_domain()?;
            match detail.direction {
                TradeDirection::FromTeam => from_team_picks.push(detail.pick_id),
                TradeDirection::ToTeam => to_team_picks.push(detail.pick_id),
            }
        }

        Ok(Some(TradeProposal {
            trade,
            from_team_picks,
            to_team_picks,
        }))
    }

    async fn find_by_session(&self, session_id: Uuid) -> DomainResult<Vec<PickTrade>> {
        let results = sqlx::query_as!(
            PickTradeDb,
            r#"
            SELECT id, session_id, from_team_id, to_team_id, status,
                   from_team_value, to_team_value, value_difference,
                   proposed_at, responded_at, created_at, updated_at
            FROM pick_trades
            WHERE session_id = $1
            ORDER BY created_at DESC
            "#,
            session_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results.into_iter().map(|db| db.to_domain()).collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    async fn find_pending_for_team(&self, team_id: Uuid) -> DomainResult<Vec<TradeProposal>> {
        let trades = sqlx::query_as!(
            PickTradeDb,
            r#"
            SELECT id, session_id, from_team_id, to_team_id, status,
                   from_team_value, to_team_value, value_difference,
                   proposed_at, responded_at, created_at, updated_at
            FROM pick_trades
            WHERE to_team_id = $1 AND status = 'Proposed'
            ORDER BY created_at DESC
            "#,
            team_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        let mut proposals = Vec::new();
        for trade_db in trades {
            let trade = trade_db.to_domain()?;
            if let Some(proposal) = self.find_trade_with_details(trade.id).await? {
                proposals.push(proposal);
            }
        }

        Ok(proposals)
    }

    async fn update(&self, trade: &PickTrade) -> DomainResult<PickTrade> {
        let trade_db = PickTradeDb::from_domain(trade);

        let result = sqlx::query_as!(
            PickTradeDb,
            r#"
            UPDATE pick_trades
            SET status = $2, responded_at = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, session_id, from_team_id, to_team_id, status,
                      from_team_value, to_team_value, value_difference,
                      proposed_at, responded_at, created_at, updated_at
            "#,
            trade_db.id,
            trade_db.status,
            trade_db.responded_at,
            trade_db.updated_at
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?
        .ok_or_else(|| DbError::NotFound(format!("Trade {} not found", trade_db.id)))?;

        result.to_domain().map_err(Into::into)
    }

    async fn is_pick_in_active_trade(&self, pick_id: Uuid, exclude_trade_id: Option<Uuid>) -> DomainResult<bool> {
        let exists = match exclude_trade_id {
            Some(exclude_id) => {
                let result = sqlx::query!(
                    r#"
                    SELECT EXISTS(
                        SELECT 1
                        FROM pick_trade_details ptd
                        JOIN pick_trades pt ON pt.id = ptd.trade_id
                        WHERE ptd.pick_id = $1 AND pt.status = 'Proposed' AND pt.id != $2
                    ) as "exists!"
                    "#,
                    pick_id,
                    exclude_id
                )
                .fetch_one(&self.pool)
                .await
                .map_err(DbError::DatabaseError)?;
                result.exists
            }
            None => {
                let result = sqlx::query!(
                    r#"
                    SELECT EXISTS(
                        SELECT 1
                        FROM pick_trade_details ptd
                        JOIN pick_trades pt ON pt.id = ptd.trade_id
                        WHERE ptd.pick_id = $1 AND pt.status = 'Proposed'
                    ) as "exists!"
                    "#,
                    pick_id
                )
                .fetch_one(&self.pool)
                .await
                .map_err(DbError::DatabaseError)?;
                result.exists
            }
        };

        Ok(exists)
    }

    async fn transfer_picks(
        &self,
        from_team_id: Uuid,
        to_team_id: Uuid,
        from_team_picks: &[Uuid],
        to_team_picks: &[Uuid],
    ) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.map_err(DbError::DatabaseError)?;

        // Transfer from_team picks to to_team
        for pick_id in from_team_picks {
            sqlx::query!(
                "UPDATE draft_picks SET team_id = $1, updated_at = NOW() WHERE id = $2",
                to_team_id,
                pick_id
            )
            .execute(&mut *tx)
            .await
            .map_err(DbError::DatabaseError)?;
        }

        // Transfer to_team picks to from_team
        for pick_id in to_team_picks {
            sqlx::query!(
                "UPDATE draft_picks SET team_id = $1, updated_at = NOW() WHERE id = $2",
                from_team_id,
                pick_id
            )
            .execute(&mut *tx)
            .await
            .map_err(DbError::DatabaseError)?;
        }

        tx.commit().await.map_err(DbError::DatabaseError)?;
        Ok(())
    }
}
