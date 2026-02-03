use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::DraftStrategy;
use domain::repositories::DraftStrategyRepository;

use crate::errors::DbError;
use crate::models::DraftStrategyDb;

/// SQLx implementation of DraftStrategyRepository
pub struct SqlxDraftStrategyRepository {
    pool: PgPool,
}

impl SqlxDraftStrategyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DraftStrategyRepository for SqlxDraftStrategyRepository {
    async fn create(&self, strategy: &DraftStrategy) -> DomainResult<DraftStrategy> {
        let strategy_db = DraftStrategyDb::from_domain(strategy)?;

        let result = sqlx::query_as!(
            DraftStrategyDb,
            r#"
            INSERT INTO draft_strategies
            (id, team_id, draft_id, bpa_weight, need_weight, position_values, risk_tolerance, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, team_id, draft_id, bpa_weight, need_weight, position_values, risk_tolerance, created_at, updated_at
            "#,
            strategy_db.id,
            strategy_db.team_id,
            strategy_db.draft_id,
            strategy_db.bpa_weight,
            strategy_db.need_weight,
            strategy_db.position_values,
            strategy_db.risk_tolerance,
            strategy_db.created_at,
            strategy_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Draft strategy for team {} and draft {} already exists",
                        strategy.team_id, strategy.draft_id
                    ));
                }
                if db_err.is_foreign_key_violation() {
                    return DbError::NotFound("Team or draft not found".to_string());
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftStrategy>> {
        let result = sqlx::query_as!(
            DraftStrategyDb,
            r#"
            SELECT id, team_id, draft_id, bpa_weight, need_weight, position_values, risk_tolerance, created_at, updated_at
            FROM draft_strategies
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(strategy_db) => Ok(Some(strategy_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_team_and_draft(
        &self,
        team_id: Uuid,
        draft_id: Uuid,
    ) -> DomainResult<Option<DraftStrategy>> {
        let result = sqlx::query_as!(
            DraftStrategyDb,
            r#"
            SELECT id, team_id, draft_id, bpa_weight, need_weight, position_values, risk_tolerance, created_at, updated_at
            FROM draft_strategies
            WHERE team_id = $1 AND draft_id = $2
            "#,
            team_id,
            draft_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(strategy_db) => Ok(Some(strategy_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_draft_id(&self, draft_id: Uuid) -> DomainResult<Vec<DraftStrategy>> {
        let results = sqlx::query_as!(
            DraftStrategyDb,
            r#"
            SELECT id, team_id, draft_id, bpa_weight, need_weight, position_values, risk_tolerance, created_at, updated_at
            FROM draft_strategies
            WHERE draft_id = $1
            "#,
            draft_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|r| r.to_domain().map_err(Into::into))
            .collect()
    }

    async fn update(&self, strategy: &DraftStrategy) -> DomainResult<DraftStrategy> {
        let strategy_db = DraftStrategyDb::from_domain(strategy)?;

        let result = sqlx::query_as!(
            DraftStrategyDb,
            r#"
            UPDATE draft_strategies
            SET bpa_weight = $2,
                need_weight = $3,
                position_values = $4,
                risk_tolerance = $5,
                updated_at = $6
            WHERE id = $1
            RETURNING id, team_id, draft_id, bpa_weight, need_weight, position_values, risk_tolerance, created_at, updated_at
            "#,
            strategy_db.id,
            strategy_db.bpa_weight,
            strategy_db.need_weight,
            strategy_db.position_values,
            strategy_db.risk_tolerance,
            strategy_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM draft_strategies WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;
    use domain::models::{Team, Conference, Division, Draft};
    use domain::repositories::{TeamRepository, DraftRepository};
    use crate::repositories::{SqlxTeamRepository, SqlxDraftRepository};

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
            });

        create_pool(&database_url).await.expect("Failed to create pool")
    }

    async fn cleanup_draft_strategies(pool: &PgPool) {
        sqlx::query!("DELETE FROM draft_strategies")
            .execute(pool)
            .await
            .expect("Failed to cleanup draft_strategies");
    }

    async fn cleanup_drafts(pool: &PgPool) {
        sqlx::query!("DELETE FROM draft_picks")
            .execute(pool)
            .await
            .expect("Failed to cleanup draft_picks");
        sqlx::query!("DELETE FROM drafts")
            .execute(pool)
            .await
            .expect("Failed to cleanup drafts");
    }

    async fn cleanup_teams(pool: &PgPool) {
        sqlx::query!("DELETE FROM teams")
            .execute(pool)
            .await
            .expect("Failed to cleanup teams");
    }

    async fn create_test_team(pool: &PgPool, abbr: &str) -> Team {
        let team_repo = SqlxTeamRepository::new(pool.clone());
        let team = Team::new(
            format!("Test Team {}", abbr),
            abbr.to_string(),
            "Test City".to_string(),
            Conference::AFC,
            Division::AFCEast,
        )
        .unwrap();
        team_repo.create(&team).await.unwrap()
    }

    async fn create_test_draft(pool: &PgPool, year: i32) -> Draft {
        let draft_repo = SqlxDraftRepository::new(pool.clone());
        let draft = Draft::new(year, 7, 32).unwrap();
        draft_repo.create(&draft).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_draft_strategy() {
        let pool = setup_test_pool().await;
        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let draft = create_test_draft(&pool, 2026).await;
        let repo = SqlxDraftStrategyRepository::new(pool.clone());

        let strategy = DraftStrategy::default_strategy(team.id, draft.id);
        let created = repo.create(&strategy).await.unwrap();

        assert_eq!(created.team_id, team.id);
        assert_eq!(created.draft_id, draft.id);
        assert_eq!(created.bpa_weight, 60);
        assert_eq!(created.need_weight, 40);
        assert_eq!(created.risk_tolerance, 5);

        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = setup_test_pool().await;
        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let draft = create_test_draft(&pool, 2026).await;
        let repo = SqlxDraftStrategyRepository::new(pool.clone());

        let strategy = DraftStrategy::default_strategy(team.id, draft.id);
        let created = repo.create(&strategy).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.team_id, team.id);
        assert_eq!(found.draft_id, draft.id);

        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_team_and_draft() {
        let pool = setup_test_pool().await;
        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let draft = create_test_draft(&pool, 2026).await;
        let repo = SqlxDraftStrategyRepository::new(pool.clone());

        let strategy = DraftStrategy::default_strategy(team.id, draft.id);
        repo.create(&strategy).await.unwrap();

        let found = repo.find_by_team_and_draft(team.id, draft.id).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.team_id, team.id);
        assert_eq!(found.draft_id, draft.id);

        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_draft_id() {
        let pool = setup_test_pool().await;
        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;

        let team1 = create_test_team(&pool, "TS1").await;
        let team2 = create_test_team(&pool, "TS2").await;
        let draft = create_test_draft(&pool, 2026).await;
        let repo = SqlxDraftStrategyRepository::new(pool.clone());

        let strategy1 = DraftStrategy::default_strategy(team1.id, draft.id);
        let strategy2 = DraftStrategy::default_strategy(team2.id, draft.id);

        repo.create(&strategy1).await.unwrap();
        repo.create(&strategy2).await.unwrap();

        let found = repo.find_by_draft_id(draft.id).await.unwrap();

        assert_eq!(found.len(), 2);

        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_update_draft_strategy() {
        let pool = setup_test_pool().await;
        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let draft = create_test_draft(&pool, 2026).await;
        let repo = SqlxDraftStrategyRepository::new(pool.clone());

        let strategy = DraftStrategy::default_strategy(team.id, draft.id);
        let mut created = repo.create(&strategy).await.unwrap();

        created.update_weights(70, 30).unwrap();
        created.update_risk_tolerance(8).unwrap();

        let updated = repo.update(&created).await.unwrap();

        assert_eq!(updated.bpa_weight, 70);
        assert_eq!(updated.need_weight, 30);
        assert_eq!(updated.risk_tolerance, 8);

        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_draft_strategy() {
        let pool = setup_test_pool().await;
        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let draft = create_test_draft(&pool, 2026).await;
        let repo = SqlxDraftStrategyRepository::new(pool.clone());

        let strategy = DraftStrategy::default_strategy(team.id, draft.id);
        let created = repo.create(&strategy).await.unwrap();

        repo.delete(created.id).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_none());

        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_duplicate_team_draft() {
        let pool = setup_test_pool().await;
        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let draft = create_test_draft(&pool, 2026).await;
        let repo = SqlxDraftStrategyRepository::new(pool.clone());

        let strategy = DraftStrategy::default_strategy(team.id, draft.id);
        repo.create(&strategy).await.unwrap();

        let duplicate = DraftStrategy::default_strategy(team.id, draft.id);
        let result = repo.create(&duplicate).await;

        assert!(result.is_err());

        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_position_values_roundtrip() {
        let pool = setup_test_pool().await;
        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let draft = create_test_draft(&pool, 2026).await;
        let repo = SqlxDraftStrategyRepository::new(pool.clone());

        let strategy = DraftStrategy::default_strategy(team.id, draft.id);
        let created = repo.create(&strategy).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap().unwrap();

        assert!(found.position_values.is_some());
        let position_values = found.position_values.unwrap();
        assert!(position_values.len() > 0);

        cleanup_draft_strategies(&pool).await;
        cleanup_drafts(&pool).await;
        cleanup_teams(&pool).await;
    }
}
