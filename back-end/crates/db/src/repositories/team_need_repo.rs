use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::TeamNeed;
use domain::repositories::TeamNeedRepository;

use crate::errors::DbError;
use crate::models::TeamNeedDb;

/// SQLx implementation of TeamNeedRepository
pub struct SqlxTeamNeedRepository {
    pool: PgPool,
}

impl SqlxTeamNeedRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamNeedRepository for SqlxTeamNeedRepository {
    async fn create(&self, need: &TeamNeed) -> DomainResult<TeamNeed> {
        let need_db = TeamNeedDb::from_domain(need);

        let result = sqlx::query_as!(
            TeamNeedDb,
            r#"
            INSERT INTO team_needs (id, team_id, position, priority, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, team_id, position, priority, created_at, updated_at
            "#,
            need_db.id,
            need_db.team_id,
            need_db.position,
            need_db.priority,
            need_db.created_at,
            need_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Team need for team {} and position {} already exists",
                        need.team_id, need_db.position
                    ));
                }
                if db_err.is_foreign_key_violation() {
                    return DbError::NotFound(format!("Team with id {} not found", need.team_id));
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<TeamNeed>> {
        let result = sqlx::query_as!(
            TeamNeedDb,
            r#"
            SELECT id, team_id, position, priority, created_at, updated_at
            FROM team_needs
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(need_db) => Ok(Some(need_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_team_id(&self, team_id: Uuid) -> DomainResult<Vec<TeamNeed>> {
        let results = sqlx::query_as!(
            TeamNeedDb,
            r#"
            SELECT id, team_id, position, priority, created_at, updated_at
            FROM team_needs
            WHERE team_id = $1
            ORDER BY priority ASC
            "#,
            team_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|r| r.to_domain().map_err(Into::into))
            .collect()
    }

    async fn update(&self, need: &TeamNeed) -> DomainResult<TeamNeed> {
        let need_db = TeamNeedDb::from_domain(need);

        let result = sqlx::query_as!(
            TeamNeedDb,
            r#"
            UPDATE team_needs
            SET priority = $2,
                updated_at = $3
            WHERE id = $1
            RETURNING id, team_id, position, priority, created_at, updated_at
            "#,
            need_db.id,
            need_db.priority,
            need_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM team_needs WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        Ok(())
    }

    async fn delete_by_team_id(&self, team_id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM team_needs WHERE team_id = $1
            "#,
            team_id
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
    use crate::repositories::SqlxTeamRepository;
    use domain::models::{Conference, Division, Position, Team};
    use domain::repositories::TeamRepository;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        create_pool(&database_url)
            .await
            .expect("Failed to create pool")
    }

    async fn cleanup_team_needs(pool: &PgPool) {
        sqlx::query!("DELETE FROM team_needs")
            .execute(pool)
            .await
            .expect("Failed to cleanup team_needs");
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

    #[tokio::test]
    async fn test_create_team_need() {
        let pool = setup_test_pool().await;
        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxTeamNeedRepository::new(pool.clone());

        let need = TeamNeed::new(team.id, Position::QB, 10).unwrap();
        let created = repo.create(&need).await.unwrap();

        assert_eq!(created.team_id, team.id);
        assert_eq!(created.position, Position::QB);
        assert_eq!(created.priority, 10);

        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = setup_test_pool().await;
        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxTeamNeedRepository::new(pool.clone());

        let need = TeamNeed::new(team.id, Position::QB, 10).unwrap();
        let created = repo.create(&need).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.position, Position::QB);

        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_team_id() {
        let pool = setup_test_pool().await;
        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxTeamNeedRepository::new(pool.clone());

        let need1 = TeamNeed::new(team.id, Position::QB, 10).unwrap();
        let need2 = TeamNeed::new(team.id, Position::WR, 8).unwrap();
        let need3 = TeamNeed::new(team.id, Position::DE, 5).unwrap();

        repo.create(&need1).await.unwrap();
        repo.create(&need2).await.unwrap();
        repo.create(&need3).await.unwrap();

        let found = repo.find_by_team_id(team.id).await.unwrap();

        assert_eq!(found.len(), 3);
        assert_eq!(found[0].priority, 5); // Highest priority (lowest number) first
        assert_eq!(found[1].priority, 8);
        assert_eq!(found[2].priority, 10);

        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_update_team_need() {
        let pool = setup_test_pool().await;
        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxTeamNeedRepository::new(pool.clone());

        let need = TeamNeed::new(team.id, Position::QB, 10).unwrap();
        let created = repo.create(&need).await.unwrap();

        let updated = TeamNeed {
            priority: 5,
            ..created
        };

        let result = repo.update(&updated).await.unwrap();

        assert_eq!(result.priority, 5);

        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_team_need() {
        let pool = setup_test_pool().await;
        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxTeamNeedRepository::new(pool.clone());

        let need = TeamNeed::new(team.id, Position::QB, 10).unwrap();
        let created = repo.create(&need).await.unwrap();

        repo.delete(created.id).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_none());

        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_by_team_id() {
        let pool = setup_test_pool().await;
        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxTeamNeedRepository::new(pool.clone());

        let need1 = TeamNeed::new(team.id, Position::QB, 10).unwrap();
        let need2 = TeamNeed::new(team.id, Position::WR, 8).unwrap();

        repo.create(&need1).await.unwrap();
        repo.create(&need2).await.unwrap();

        repo.delete_by_team_id(team.id).await.unwrap();

        let found = repo.find_by_team_id(team.id).await.unwrap();
        assert_eq!(found.len(), 0);

        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_duplicate_team_position() {
        let pool = setup_test_pool().await;
        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxTeamNeedRepository::new(pool.clone());

        let need = TeamNeed::new(team.id, Position::QB, 10).unwrap();
        repo.create(&need).await.unwrap();

        let duplicate = TeamNeed::new(team.id, Position::QB, 5).unwrap();
        let result = repo.create(&duplicate).await;

        assert!(result.is_err());

        cleanup_team_needs(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_invalid_priority() {
        let pool = setup_test_pool().await;
        cleanup_teams(&pool).await;

        let team = create_test_team(&pool, "TST").await;

        let result = TeamNeed::new(team.id, Position::QB, 0);
        assert!(result.is_err());

        let result = TeamNeed::new(team.id, Position::QB, 11);
        assert!(result.is_err());

        cleanup_teams(&pool).await;
    }
}
