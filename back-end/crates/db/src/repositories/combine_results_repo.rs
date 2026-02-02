use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::CombineResults;
use domain::repositories::CombineResultsRepository;

use crate::errors::DbError;
use crate::models::CombineResultsDb;

/// SQLx implementation of CombineResultsRepository
pub struct SqlxCombineResultsRepository {
    pool: PgPool,
}

impl SqlxCombineResultsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CombineResultsRepository for SqlxCombineResultsRepository {
    async fn create(&self, results: &CombineResults) -> DomainResult<CombineResults> {
        let results_db = CombineResultsDb::from_domain(results);

        let result = sqlx::query_as!(
            CombineResultsDb,
            r#"
            INSERT INTO combine_results
            (id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
             broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                      broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            "#,
            results_db.id,
            results_db.player_id,
            results_db.year,
            results_db.forty_yard_dash,
            results_db.bench_press,
            results_db.vertical_jump,
            results_db.broad_jump,
            results_db.three_cone_drill,
            results_db.twenty_yard_shuttle,
            results_db.created_at,
            results_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Combine results for player {} in year {} already exist",
                        results.player_id, results.year
                    ));
                }
                if db_err.is_foreign_key_violation() {
                    return DbError::NotFound(format!("Player with id {} not found", results.player_id));
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<CombineResults>> {
        let result = sqlx::query_as!(
            CombineResultsDb,
            r#"
            SELECT id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                   broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            FROM combine_results
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(results_db) => Ok(Some(results_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_player_id(&self, player_id: Uuid) -> DomainResult<Vec<CombineResults>> {
        let results = sqlx::query_as!(
            CombineResultsDb,
            r#"
            SELECT id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                   broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            FROM combine_results
            WHERE player_id = $1
            ORDER BY year DESC
            "#,
            player_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|r| r.to_domain().map_err(Into::into))
            .collect()
    }

    async fn find_by_player_and_year(
        &self,
        player_id: Uuid,
        year: i32,
    ) -> DomainResult<Option<CombineResults>> {
        let result = sqlx::query_as!(
            CombineResultsDb,
            r#"
            SELECT id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                   broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            FROM combine_results
            WHERE player_id = $1 AND year = $2
            "#,
            player_id,
            year
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(results_db) => Ok(Some(results_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn update(&self, results: &CombineResults) -> DomainResult<CombineResults> {
        let results_db = CombineResultsDb::from_domain(results);

        let result = sqlx::query_as!(
            CombineResultsDb,
            r#"
            UPDATE combine_results
            SET forty_yard_dash = $2,
                bench_press = $3,
                vertical_jump = $4,
                broad_jump = $5,
                three_cone_drill = $6,
                twenty_yard_shuttle = $7,
                updated_at = $8
            WHERE id = $1
            RETURNING id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                   broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            "#,
            results_db.id,
            results_db.forty_yard_dash,
            results_db.bench_press,
            results_db.vertical_jump,
            results_db.broad_jump,
            results_db.three_cone_drill,
            results_db.twenty_yard_shuttle,
            results_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM combine_results WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        Ok(())
    }
}
