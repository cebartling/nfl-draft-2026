use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::ProspectRanking;
use domain::repositories::ProspectRankingRepository;

use crate::errors::DbError;
use crate::models::ProspectRankingDb;

/// SQLx implementation of ProspectRankingRepository
pub struct SqlxProspectRankingRepository {
    pool: PgPool,
}

impl SqlxProspectRankingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProspectRankingRepository for SqlxProspectRankingRepository {
    async fn create_batch(&self, rankings: &[ProspectRanking]) -> DomainResult<usize> {
        if rankings.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for ranking in rankings {
            let db = ProspectRankingDb::from_domain(ranking);
            sqlx::query!(
                r#"
                INSERT INTO prospect_rankings (id, ranking_source_id, player_id, rank, scraped_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                db.id,
                db.ranking_source_id,
                db.player_id,
                db.rank,
                db.scraped_at,
                db.created_at
            )
            .execute(&self.pool)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.is_unique_violation() {
                        return DbError::DuplicateEntry(format!(
                            "Ranking for source {} and player {} already exists",
                            ranking.ranking_source_id, ranking.player_id
                        ));
                    }
                }
                DbError::DatabaseError(e)
            })?;
            count += 1;
        }

        Ok(count)
    }

    async fn find_by_player(&self, player_id: Uuid) -> DomainResult<Vec<ProspectRanking>> {
        let results = sqlx::query_as!(
            ProspectRankingDb,
            r#"
            SELECT id, ranking_source_id, player_id, rank, scraped_at, created_at
            FROM prospect_rankings
            WHERE player_id = $1
            ORDER BY rank
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

    async fn find_by_source(&self, source_id: Uuid) -> DomainResult<Vec<ProspectRanking>> {
        let results = sqlx::query_as!(
            ProspectRankingDb,
            r#"
            SELECT id, ranking_source_id, player_id, rank, scraped_at, created_at
            FROM prospect_rankings
            WHERE ranking_source_id = $1
            ORDER BY rank
            "#,
            source_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|r| r.to_domain().map_err(Into::into))
            .collect()
    }

    async fn delete_by_source(&self, source_id: Uuid) -> DomainResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM prospect_rankings WHERE ranking_source_id = $1
            "#,
            source_id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        Ok(result.rows_affected())
    }
}
