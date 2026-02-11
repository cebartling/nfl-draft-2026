use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::RankingSource;
use domain::repositories::RankingSourceRepository;

use crate::errors::DbError;
use crate::models::RankingSourceDb;

/// SQLx implementation of RankingSourceRepository
pub struct SqlxRankingSourceRepository {
    pool: PgPool,
}

impl SqlxRankingSourceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RankingSourceRepository for SqlxRankingSourceRepository {
    async fn create(&self, source: &RankingSource) -> DomainResult<RankingSource> {
        let source_db = RankingSourceDb::from_domain(source);

        let result = sqlx::query_as!(
            RankingSourceDb,
            r#"
            INSERT INTO ranking_sources (id, name, url, description, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, url, description, created_at, updated_at
            "#,
            source_db.id,
            source_db.name,
            source_db.url,
            source_db.description,
            source_db.created_at,
            source_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Ranking source '{}' already exists",
                        source.name
                    ));
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<RankingSource>> {
        let result = sqlx::query_as!(
            RankingSourceDb,
            r#"
            SELECT id, name, url, description, created_at, updated_at
            FROM ranking_sources
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

    async fn find_by_name(&self, name: &str) -> DomainResult<Option<RankingSource>> {
        let result = sqlx::query_as!(
            RankingSourceDb,
            r#"
            SELECT id, name, url, description, created_at, updated_at
            FROM ranking_sources
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(db) => Ok(Some(db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> DomainResult<Vec<RankingSource>> {
        let results = sqlx::query_as!(
            RankingSourceDb,
            r#"
            SELECT id, name, url, description, created_at, updated_at
            FROM ranking_sources
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|r| r.to_domain().map_err(Into::into))
            .collect()
    }
}
