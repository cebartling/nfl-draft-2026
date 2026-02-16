use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::{DomainError, DomainResult};
use domain::models::CombinePercentile;
use domain::repositories::CombinePercentileRepository;

use crate::models::CombinePercentileDb;

pub struct SqlxCombinePercentileRepository {
    pool: PgPool,
}

impl SqlxCombinePercentileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CombinePercentileRepository for SqlxCombinePercentileRepository {
    async fn find_all(&self) -> DomainResult<Vec<CombinePercentile>> {
        let rows = sqlx::query_as!(
            CombinePercentileDb,
            r#"
            SELECT id, position, measurement, sample_size, min_value,
                   p10, p20, p30, p40, p50, p60, p70, p80, p90,
                   max_value, years_start, years_end, created_at, updated_at
            FROM combine_percentiles
            ORDER BY position, measurement
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                r.to_domain()
                    .map_err(|e| DomainError::InternalError(e.to_string()))
            })
            .collect()
    }

    async fn find_by_position(&self, position: &str) -> DomainResult<Vec<CombinePercentile>> {
        let rows = sqlx::query_as!(
            CombinePercentileDb,
            r#"
            SELECT id, position, measurement, sample_size, min_value,
                   p10, p20, p30, p40, p50, p60, p70, p80, p90,
                   max_value, years_start, years_end, created_at, updated_at
            FROM combine_percentiles
            WHERE position = $1
            ORDER BY measurement
            "#,
            position
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                r.to_domain()
                    .map_err(|e| DomainError::InternalError(e.to_string()))
            })
            .collect()
    }

    async fn find_by_position_and_measurement(
        &self,
        position: &str,
        measurement: &str,
    ) -> DomainResult<Option<CombinePercentile>> {
        let row = sqlx::query_as!(
            CombinePercentileDb,
            r#"
            SELECT id, position, measurement, sample_size, min_value,
                   p10, p20, p30, p40, p50, p60, p70, p80, p90,
                   max_value, years_start, years_end, created_at, updated_at
            FROM combine_percentiles
            WHERE position = $1 AND measurement = $2
            "#,
            position,
            measurement
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(
                r.to_domain()
                    .map_err(|e| DomainError::InternalError(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    async fn upsert(&self, percentile: &CombinePercentile) -> DomainResult<CombinePercentile> {
        let db = CombinePercentileDb::from_domain(percentile);

        let row = sqlx::query_as!(
            CombinePercentileDb,
            r#"
            INSERT INTO combine_percentiles (
                id, position, measurement, sample_size, min_value,
                p10, p20, p30, p40, p50, p60, p70, p80, p90,
                max_value, years_start, years_end
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (position, measurement)
            DO UPDATE SET
                sample_size = EXCLUDED.sample_size,
                min_value = EXCLUDED.min_value,
                p10 = EXCLUDED.p10,
                p20 = EXCLUDED.p20,
                p30 = EXCLUDED.p30,
                p40 = EXCLUDED.p40,
                p50 = EXCLUDED.p50,
                p60 = EXCLUDED.p60,
                p70 = EXCLUDED.p70,
                p80 = EXCLUDED.p80,
                p90 = EXCLUDED.p90,
                max_value = EXCLUDED.max_value,
                years_start = EXCLUDED.years_start,
                years_end = EXCLUDED.years_end,
                updated_at = NOW()
            RETURNING id, position, measurement, sample_size, min_value,
                      p10, p20, p30, p40, p50, p60, p70, p80, p90,
                      max_value, years_start, years_end, created_at, updated_at
            "#,
            db.id,
            db.position,
            db.measurement,
            db.sample_size,
            db.min_value,
            db.p10,
            db.p20,
            db.p30,
            db.p40,
            db.p50,
            db.p60,
            db.p70,
            db.p80,
            db.p90,
            db.max_value,
            db.years_start,
            db.years_end,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        row.to_domain()
            .map_err(|e| DomainError::InternalError(e.to_string()))
    }

    async fn delete_all(&self) -> DomainResult<u64> {
        let result = sqlx::query!("DELETE FROM combine_percentiles")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let result = sqlx::query!("DELETE FROM combine_percentiles WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!(
                "Combine percentile with id {} not found",
                id
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::Measurement;

    async fn setup_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });
        crate::create_pool(&database_url)
            .await
            .expect("Failed to create pool")
    }

    async fn cleanup(pool: &PgPool) {
        sqlx::query!("DELETE FROM combine_percentiles")
            .execute(pool)
            .await
            .expect("Failed to cleanup");
    }

    #[tokio::test]
    async fn test_upsert_and_find_all() {
        let pool = setup_pool().await;
        cleanup(&pool).await;

        let repo = SqlxCombinePercentileRepository::new(pool.clone());

        let p = CombinePercentile::new("QB".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(100, 4.4, 4.55, 4.6, 4.65, 4.7, 4.75, 4.8, 4.85, 4.9, 5.0, 5.3)
            .unwrap();

        let created = repo.upsert(&p).await.unwrap();
        assert_eq!(created.position, "QB");
        assert_eq!(created.measurement, Measurement::FortyYardDash);
        assert_eq!(created.sample_size, 100);

        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 1);

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_upsert_updates_on_conflict() {
        let pool = setup_pool().await;
        cleanup(&pool).await;

        let repo = SqlxCombinePercentileRepository::new(pool.clone());

        let p1 = CombinePercentile::new("QB".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(100, 4.4, 4.55, 4.6, 4.65, 4.7, 4.75, 4.8, 4.85, 4.9, 5.0, 5.3)
            .unwrap();

        repo.upsert(&p1).await.unwrap();

        // Upsert again with different data
        let p2 = CombinePercentile::new("QB".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(200, 4.3, 4.45, 4.5, 4.55, 4.6, 4.65, 4.7, 4.75, 4.8, 4.9, 5.2)
            .unwrap();

        let updated = repo.upsert(&p2).await.unwrap();
        assert_eq!(updated.sample_size, 200);
        assert_eq!(updated.p50, 4.65);

        // Should still be 1 row
        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 1);

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_position() {
        let pool = setup_pool().await;
        cleanup(&pool).await;

        let repo = SqlxCombinePercentileRepository::new(pool.clone());

        let qb_40 = CombinePercentile::new("QB".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(100, 4.4, 4.5, 4.6, 4.65, 4.7, 4.75, 4.8, 4.85, 4.9, 5.0, 5.3)
            .unwrap();
        let qb_bench = CombinePercentile::new("QB".to_string(), Measurement::BenchPress)
            .unwrap()
            .with_percentiles(100, 10.0, 14.0, 16.0, 18.0, 19.0, 20.0, 22.0, 24.0, 26.0, 28.0, 35.0)
            .unwrap();
        let wr_40 = CombinePercentile::new("WR".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(150, 4.2, 4.3, 4.35, 4.4, 4.42, 4.45, 4.5, 4.55, 4.6, 4.7, 5.0)
            .unwrap();

        repo.upsert(&qb_40).await.unwrap();
        repo.upsert(&qb_bench).await.unwrap();
        repo.upsert(&wr_40).await.unwrap();

        let qb_results = repo.find_by_position("QB").await.unwrap();
        assert_eq!(qb_results.len(), 2);

        let wr_results = repo.find_by_position("WR").await.unwrap();
        assert_eq!(wr_results.len(), 1);

        let te_results = repo.find_by_position("TE").await.unwrap();
        assert_eq!(te_results.len(), 0);

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_position_and_measurement() {
        let pool = setup_pool().await;
        cleanup(&pool).await;

        let repo = SqlxCombinePercentileRepository::new(pool.clone());

        let p = CombinePercentile::new("QB".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(100, 4.4, 4.55, 4.6, 4.65, 4.7, 4.75, 4.8, 4.85, 4.9, 5.0, 5.3)
            .unwrap();

        repo.upsert(&p).await.unwrap();

        let found = repo
            .find_by_position_and_measurement("QB", "forty_yard_dash")
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().p50, 4.75);

        let not_found = repo
            .find_by_position_and_measurement("QB", "bench_press")
            .await
            .unwrap();
        assert!(not_found.is_none());

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_all() {
        let pool = setup_pool().await;
        cleanup(&pool).await;

        let repo = SqlxCombinePercentileRepository::new(pool.clone());

        let p = CombinePercentile::new("QB".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(100, 4.4, 4.55, 4.6, 4.65, 4.7, 4.75, 4.8, 4.85, 4.9, 5.0, 5.3)
            .unwrap();
        repo.upsert(&p).await.unwrap();

        let deleted = repo.delete_all().await.unwrap();
        assert_eq!(deleted, 1);

        let all = repo.find_all().await.unwrap();
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_delete_by_id() {
        let pool = setup_pool().await;
        cleanup(&pool).await;

        let repo = SqlxCombinePercentileRepository::new(pool.clone());

        let p = CombinePercentile::new("QB".to_string(), Measurement::FortyYardDash)
            .unwrap()
            .with_percentiles(100, 4.4, 4.55, 4.6, 4.65, 4.7, 4.75, 4.8, 4.85, 4.9, 5.0, 5.3)
            .unwrap();
        let created = repo.upsert(&p).await.unwrap();

        repo.delete(created.id).await.unwrap();

        let all = repo.find_all().await.unwrap();
        assert!(all.is_empty());

        cleanup(&pool).await;
    }
}
