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
