use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::ProspectProfile;
use domain::repositories::ProspectProfileRepository;

use crate::errors::DbError;
use crate::models::ProspectProfileDb;

/// SQLx implementation of `ProspectProfileRepository`.
pub struct SqlxProspectProfileRepository {
    pool: PgPool,
}

impl SqlxProspectProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProspectProfileRepository for SqlxProspectProfileRepository {
    async fn upsert(&self, profile: &ProspectProfile) -> DomainResult<ProspectProfile> {
        let db = ProspectProfileDb::from_domain(profile);

        let result = sqlx::query_as!(
            ProspectProfileDb,
            r#"
            INSERT INTO prospect_profiles (
                id, player_id, source, grade_tier, overall_rank, position_rank,
                year_class, birthday, jersey_number, height_raw, nfl_comparison,
                background, summary, strengths, weaknesses, college_stats,
                scraped_at, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16,
                $17, $18, $19
            )
            ON CONFLICT (player_id, source) DO UPDATE SET
                grade_tier = EXCLUDED.grade_tier,
                overall_rank = EXCLUDED.overall_rank,
                position_rank = EXCLUDED.position_rank,
                year_class = EXCLUDED.year_class,
                birthday = EXCLUDED.birthday,
                jersey_number = EXCLUDED.jersey_number,
                height_raw = EXCLUDED.height_raw,
                nfl_comparison = EXCLUDED.nfl_comparison,
                background = EXCLUDED.background,
                summary = EXCLUDED.summary,
                strengths = EXCLUDED.strengths,
                weaknesses = EXCLUDED.weaknesses,
                college_stats = EXCLUDED.college_stats,
                scraped_at = EXCLUDED.scraped_at,
                updated_at = NOW()
            RETURNING
                id, player_id, source, grade_tier, overall_rank, position_rank,
                year_class, birthday, jersey_number, height_raw, nfl_comparison,
                background, summary,
                strengths as "strengths: serde_json::Value",
                weaknesses as "weaknesses: serde_json::Value",
                college_stats as "college_stats: serde_json::Value",
                scraped_at, created_at, updated_at
            "#,
            db.id,
            db.player_id,
            db.source,
            db.grade_tier,
            db.overall_rank,
            db.position_rank,
            db.year_class,
            db.birthday,
            db.jersey_number,
            db.height_raw,
            db.nfl_comparison,
            db.background,
            db.summary,
            db.strengths,
            db.weaknesses,
            db.college_stats,
            db.scraped_at,
            db.created_at,
            db.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_foreign_key_violation() {
                    return DbError::NotFound(format!("Player {} not found", profile.player_id));
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_latest_by_player(
        &self,
        player_id: Uuid,
    ) -> DomainResult<Option<ProspectProfile>> {
        let result = sqlx::query_as!(
            ProspectProfileDb,
            r#"
            SELECT
                id, player_id, source, grade_tier, overall_rank, position_rank,
                year_class, birthday, jersey_number, height_raw, nfl_comparison,
                background, summary,
                strengths as "strengths: serde_json::Value",
                weaknesses as "weaknesses: serde_json::Value",
                college_stats as "college_stats: serde_json::Value",
                scraped_at, created_at, updated_at
            FROM prospect_profiles
            WHERE player_id = $1
            ORDER BY scraped_at DESC, updated_at DESC
            LIMIT 1
            "#,
            player_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(row) => Ok(Some(row.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_player_and_source(
        &self,
        player_id: Uuid,
        source: &str,
    ) -> DomainResult<Option<ProspectProfile>> {
        let result = sqlx::query_as!(
            ProspectProfileDb,
            r#"
            SELECT
                id, player_id, source, grade_tier, overall_rank, position_rank,
                year_class, birthday, jersey_number, height_raw, nfl_comparison,
                background, summary,
                strengths as "strengths: serde_json::Value",
                weaknesses as "weaknesses: serde_json::Value",
                college_stats as "college_stats: serde_json::Value",
                scraped_at, created_at, updated_at
            FROM prospect_profiles
            WHERE player_id = $1 AND source = $2
            "#,
            player_id,
            source
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(row) => Ok(Some(row.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_source(&self, source: &str) -> DomainResult<Vec<ProspectProfile>> {
        let results = sqlx::query_as!(
            ProspectProfileDb,
            r#"
            SELECT
                id, player_id, source, grade_tier, overall_rank, position_rank,
                year_class, birthday, jersey_number, height_raw, nfl_comparison,
                background, summary,
                strengths as "strengths: serde_json::Value",
                weaknesses as "weaknesses: serde_json::Value",
                college_stats as "college_stats: serde_json::Value",
                scraped_at, created_at, updated_at
            FROM prospect_profiles
            WHERE source = $1
            ORDER BY overall_rank ASC NULLS LAST, position_rank ASC
            "#,
            source
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|r| r.to_domain().map_err(Into::into))
            .collect()
    }

    async fn delete_by_source(&self, source: &str) -> DomainResult<u64> {
        let result = sqlx::query!(r#"DELETE FROM prospect_profiles WHERE source = $1"#, source)
            .execute(&self.pool)
            .await
            .map_err(DbError::DatabaseError)?;

        Ok(result.rows_affected())
    }
}
