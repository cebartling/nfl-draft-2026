use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::ScoutingReport;
use domain::repositories::ScoutingReportRepository;

use crate::errors::DbError;
use crate::models::ScoutingReportDb;

/// SQLx implementation of ScoutingReportRepository
pub struct SqlxScoutingReportRepository {
    pool: PgPool,
}

impl SqlxScoutingReportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ScoutingReportRepository for SqlxScoutingReportRepository {
    async fn create(&self, report: &ScoutingReport) -> DomainResult<ScoutingReport> {
        let report_db = ScoutingReportDb::from_domain(report);

        let result = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            INSERT INTO scouting_reports
            (id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            "#,
            report_db.id,
            report_db.player_id,
            report_db.team_id,
            report_db.grade,
            report_db.notes,
            report_db.fit_grade,
            report_db.injury_concern,
            report_db.character_concern,
            report_db.created_at,
            report_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Scouting report for team {} and player {} already exists",
                        report.team_id, report.player_id
                    ));
                }
                if db_err.is_foreign_key_violation() {
                    return DbError::NotFound("Team or player not found".to_string());
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<ScoutingReport>> {
        let result = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            SELECT id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            FROM scouting_reports
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(report_db) => Ok(Some(report_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_team_id(&self, team_id: Uuid) -> DomainResult<Vec<ScoutingReport>> {
        let results = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            SELECT id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            FROM scouting_reports
            WHERE team_id = $1
            ORDER BY grade DESC
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

    async fn find_by_player_id(&self, player_id: Uuid) -> DomainResult<Vec<ScoutingReport>> {
        let results = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            SELECT id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            FROM scouting_reports
            WHERE player_id = $1
            ORDER BY grade DESC
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

    async fn find_by_team_and_player(
        &self,
        team_id: Uuid,
        player_id: Uuid,
    ) -> DomainResult<Option<ScoutingReport>> {
        let result = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            SELECT id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            FROM scouting_reports
            WHERE team_id = $1 AND player_id = $2
            "#,
            team_id,
            player_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(report_db) => Ok(Some(report_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn update(&self, report: &ScoutingReport) -> DomainResult<ScoutingReport> {
        let report_db = ScoutingReportDb::from_domain(report);

        let result = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            UPDATE scouting_reports
            SET grade = $2,
                notes = $3,
                fit_grade = $4,
                injury_concern = $5,
                character_concern = $6,
                updated_at = $7
            WHERE id = $1
            RETURNING id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            "#,
            report_db.id,
            report_db.grade,
            report_db.notes,
            report_db.fit_grade,
            report_db.injury_concern,
            report_db.character_concern,
            report_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM scouting_reports WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        Ok(())
    }
}
