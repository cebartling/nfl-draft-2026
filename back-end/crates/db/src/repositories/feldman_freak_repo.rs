use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::FeldmanFreak;
use domain::repositories::FeldmanFreakRepository;

use crate::errors::DbError;
use crate::models::FeldmanFreakDb;

/// SQLx implementation of FeldmanFreakRepository
pub struct SqlxFeldmanFreakRepository {
    pool: PgPool,
}

impl SqlxFeldmanFreakRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FeldmanFreakRepository for SqlxFeldmanFreakRepository {
    async fn create(&self, freak: &FeldmanFreak) -> DomainResult<FeldmanFreak> {
        let freak_db = FeldmanFreakDb::from_domain(freak);

        let result = sqlx::query_as!(
            FeldmanFreakDb,
            r#"
            INSERT INTO feldman_freaks
            (id, player_id, year, rank, description, article_url, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, player_id, year, rank, description, article_url, created_at
            "#,
            freak_db.id,
            freak_db.player_id,
            freak_db.year,
            freak_db.rank,
            freak_db.description,
            freak_db.article_url,
            freak_db.created_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Feldman Freak entry for player {} in year {} already exists",
                        freak.player_id, freak.year
                    ));
                }
                if db_err.is_foreign_key_violation() {
                    return DbError::NotFound("Player not found".to_string());
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_player(&self, player_id: Uuid) -> DomainResult<Option<FeldmanFreak>> {
        let result = sqlx::query_as!(
            FeldmanFreakDb,
            r#"
            SELECT id, player_id, year, rank, description, article_url, created_at
            FROM feldman_freaks
            WHERE player_id = $1
            "#,
            player_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(freak_db) => Ok(Some(freak_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_year(&self, year: i32) -> DomainResult<Vec<FeldmanFreak>> {
        let results = sqlx::query_as!(
            FeldmanFreakDb,
            r#"
            SELECT id, player_id, year, rank, description, article_url, created_at
            FROM feldman_freaks
            WHERE year = $1
            ORDER BY rank ASC
            "#,
            year
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|r| r.to_domain().map_err(Into::into))
            .collect()
    }

    async fn find_all(&self) -> DomainResult<Vec<FeldmanFreak>> {
        let results = sqlx::query_as!(
            FeldmanFreakDb,
            r#"
            SELECT id, player_id, year, rank, description, article_url, created_at
            FROM feldman_freaks
            ORDER BY year DESC, rank ASC
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

    async fn delete_by_year(&self, year: i32) -> DomainResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM feldman_freaks WHERE year = $1
            "#,
            year
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        Ok(result.rows_affected())
    }
}
