use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::{Draft, DraftPick, DraftStatus};
use domain::repositories::{DraftPickRepository, DraftRepository};

use crate::errors::DbError;
use crate::models::{DraftDb, DraftPickDb};

/// SQLx implementation of DraftRepository
pub struct SqlxDraftRepository {
    pool: PgPool,
}

impl SqlxDraftRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DraftRepository for SqlxDraftRepository {
    async fn create(&self, draft: &Draft) -> DomainResult<Draft> {
        let draft_db = DraftDb::from_domain(draft);

        let result = sqlx::query_as!(
            DraftDb,
            r#"
            INSERT INTO drafts (id, name, year, status, rounds, picks_per_round, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, year, status, rounds, picks_per_round, created_at, updated_at
            "#,
            draft_db.id,
            draft_db.name,
            draft_db.year,
            draft_db.status,
            draft_db.rounds,
            draft_db.picks_per_round,
            draft_db.created_at,
            draft_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Draft>> {
        let result = sqlx::query_as!(
            DraftDb,
            r#"
            SELECT id, name, year, status, rounds, picks_per_round, created_at, updated_at
            FROM drafts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(draft_db) => Ok(Some(draft_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_year(&self, year: i32) -> DomainResult<Vec<Draft>> {
        let results = sqlx::query_as!(
            DraftDb,
            r#"
            SELECT id, name, year, status, rounds, picks_per_round, created_at, updated_at
            FROM drafts
            WHERE year = $1
            ORDER BY created_at DESC
            "#,
            year
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|db| db.to_domain())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    async fn find_all(&self) -> DomainResult<Vec<Draft>> {
        let results = sqlx::query_as!(
            DraftDb,
            r#"
            SELECT id, name, year, status, rounds, picks_per_round, created_at, updated_at
            FROM drafts
            ORDER BY year DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|db| db.to_domain())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    async fn find_by_status(&self, status: DraftStatus) -> DomainResult<Vec<Draft>> {
        let status_str = status.to_string();
        let results = sqlx::query_as!(
            DraftDb,
            r#"
            SELECT id, name, year, status, rounds, picks_per_round, created_at, updated_at
            FROM drafts
            WHERE status = $1
            ORDER BY year DESC
            "#,
            status_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|db| db.to_domain())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    async fn update(&self, draft: &Draft) -> DomainResult<Draft> {
        let draft_db = DraftDb::from_domain(draft);

        let result = sqlx::query_as!(
            DraftDb,
            r#"
            UPDATE drafts
            SET name = $2, status = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, name, year, status, rounds, picks_per_round, created_at, updated_at
            "#,
            draft_db.id,
            draft_db.name,
            draft_db.status,
            draft_db.updated_at
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?
        .ok_or_else(|| DbError::NotFound(format!("Draft with id {} not found", draft_db.id)))?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM drafts WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("Draft with id {} not found", id)).into());
        }

        Ok(())
    }
}

/// SQLx implementation of DraftPickRepository
pub struct SqlxDraftPickRepository {
    pool: PgPool,
}

impl SqlxDraftPickRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DraftPickRepository for SqlxDraftPickRepository {
    async fn create(&self, pick: &DraftPick) -> DomainResult<DraftPick> {
        let pick_db = DraftPickDb::from_domain(pick);

        let result = sqlx::query_as!(
            DraftPickDb,
            r#"
            INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at
            "#,
            pick_db.id,
            pick_db.draft_id,
            pick_db.round,
            pick_db.pick_number,
            pick_db.overall_pick,
            pick_db.team_id,
            pick_db.player_id,
            pick_db.picked_at,
            pick_db.original_team_id,
            pick_db.is_compensatory,
            pick_db.notes,
            pick_db.created_at,
            pick_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(
                        "Draft pick with this round/pick number already exists".to_string()
                    );
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn create_many(&self, picks: &[DraftPick]) -> DomainResult<Vec<DraftPick>> {
        let mut tx = self.pool.begin().await.map_err(DbError::DatabaseError)?;

        let mut created_picks = Vec::new();

        for pick in picks {
            let pick_db = DraftPickDb::from_domain(pick);

            let result = sqlx::query_as!(
                DraftPickDb,
                r#"
                INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at
                "#,
                pick_db.id,
                pick_db.draft_id,
                pick_db.round,
                pick_db.pick_number,
                pick_db.overall_pick,
                pick_db.team_id,
                pick_db.player_id,
                pick_db.picked_at,
                pick_db.original_team_id,
                pick_db.is_compensatory,
                pick_db.notes,
                pick_db.created_at,
                pick_db.updated_at
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(DbError::DatabaseError)?;

            created_picks.push(result.to_domain()?);
        }

        tx.commit().await.map_err(DbError::DatabaseError)?;

        Ok(created_picks)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftPick>> {
        let result = sqlx::query_as!(
            DraftPickDb,
            r#"
            SELECT id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at
            FROM draft_picks
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(pick_db) => Ok(Some(pick_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_draft_id(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>> {
        let results = sqlx::query_as!(
            DraftPickDb,
            r#"
            SELECT id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at
            FROM draft_picks
            WHERE draft_id = $1
            ORDER BY overall_pick ASC
            "#,
            draft_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|db| db.to_domain())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    async fn find_by_draft_and_round(
        &self,
        draft_id: Uuid,
        round: i32,
    ) -> DomainResult<Vec<DraftPick>> {
        let results = sqlx::query_as!(
            DraftPickDb,
            r#"
            SELECT id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at
            FROM draft_picks
            WHERE draft_id = $1 AND round = $2
            ORDER BY pick_number ASC
            "#,
            draft_id,
            round
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|db| db.to_domain())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    async fn find_by_draft_and_team(
        &self,
        draft_id: Uuid,
        team_id: Uuid,
    ) -> DomainResult<Vec<DraftPick>> {
        let results = sqlx::query_as!(
            DraftPickDb,
            r#"
            SELECT id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at
            FROM draft_picks
            WHERE draft_id = $1 AND team_id = $2
            ORDER BY overall_pick ASC
            "#,
            draft_id,
            team_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|db| db.to_domain())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    async fn find_next_pick(&self, draft_id: Uuid) -> DomainResult<Option<DraftPick>> {
        let result = sqlx::query_as!(
            DraftPickDb,
            r#"
            SELECT id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at
            FROM draft_picks
            WHERE draft_id = $1 AND player_id IS NULL
            ORDER BY overall_pick ASC
            LIMIT 1
            "#,
            draft_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(pick_db) => Ok(Some(pick_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_available_picks(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>> {
        let results = sqlx::query_as!(
            DraftPickDb,
            r#"
            SELECT id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at
            FROM draft_picks
            WHERE draft_id = $1 AND player_id IS NULL
            ORDER BY overall_pick ASC
            "#,
            draft_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|db| db.to_domain())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    async fn update(&self, pick: &DraftPick) -> DomainResult<DraftPick> {
        let pick_db = DraftPickDb::from_domain(pick);

        let result = sqlx::query_as!(
            DraftPickDb,
            r#"
            UPDATE draft_picks
            SET player_id = $2, picked_at = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, draft_id, round, pick_number, overall_pick, team_id, player_id, picked_at, original_team_id, is_compensatory, notes, created_at, updated_at
            "#,
            pick_db.id,
            pick_db.player_id,
            pick_db.picked_at,
            pick_db.updated_at
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?
        .ok_or_else(|| DbError::NotFound(format!("Draft pick with id {} not found", pick_db.id)))?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM draft_picks WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("Draft pick with id {} not found", id)).into());
        }

        Ok(())
    }

    async fn delete_by_draft_id(&self, draft_id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM draft_picks WHERE draft_id = $1
            "#,
            draft_id
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
    use domain::models::{Conference, Division, Team};
    use domain::repositories::TeamRepository;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        create_pool(&database_url)
            .await
            .expect("Failed to create pool")
    }

    async fn cleanup(pool: &PgPool) {
        sqlx::query!("DELETE FROM draft_picks")
            .execute(pool)
            .await
            .expect("Failed to cleanup picks");
        sqlx::query!("DELETE FROM drafts")
            .execute(pool)
            .await
            .expect("Failed to cleanup drafts");
        sqlx::query!("DELETE FROM teams")
            .execute(pool)
            .await
            .expect("Failed to cleanup teams");
    }

    #[tokio::test]
    async fn test_create_draft() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let repo = SqlxDraftRepository::new(pool);
        let draft = Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap();

        let result = repo.create(&draft).await;
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(created.year, 2026);
        assert_eq!(created.status, DraftStatus::NotStarted);
    }

    #[tokio::test]
    async fn test_find_draft_by_id() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let repo = SqlxDraftRepository::new(pool);
        let draft = Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap();
        let created = repo.create(&draft).await.unwrap();

        let result = repo.find_by_id(created.id).await;
        assert!(result.is_ok());

        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_find_draft_by_year() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let repo = SqlxDraftRepository::new(pool);
        let draft = Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap();
        repo.create(&draft).await.unwrap();

        let result = repo.find_by_year(2026).await;
        assert!(result.is_ok());

        let found = result.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].year, 2026);
    }

    #[tokio::test]
    async fn test_multiple_drafts_same_year() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let repo = SqlxDraftRepository::new(pool);
        let draft1 = Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap();
        repo.create(&draft1).await.unwrap();

        let draft2 = Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap();
        let result = repo.create(&draft2).await;
        assert!(result.is_ok());

        let drafts = repo.find_by_year(2026).await.unwrap();
        assert_eq!(drafts.len(), 2);
    }

    #[tokio::test]
    async fn test_update_draft() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let repo = SqlxDraftRepository::new(pool);
        let draft = Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap();
        let mut created = repo.create(&draft).await.unwrap();

        created.start().unwrap();
        let result = repo.update(&created).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.status, DraftStatus::InProgress);
    }

    #[tokio::test]
    async fn test_create_draft_pick() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        // Create draft and team first
        let draft_repo = SqlxDraftRepository::new(pool.clone());
        let draft = Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap();
        let created_draft = draft_repo.create(&draft).await.unwrap();

        let team_repo = SqlxTeamRepository::new(pool.clone());
        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();
        let created_team = team_repo.create(&team).await.unwrap();

        // Create pick
        let pick_repo = SqlxDraftPickRepository::new(pool);
        let pick = DraftPick::new(created_draft.id, 1, 1, 1, created_team.id).unwrap();

        let result = pick_repo.create(&pick).await;
        assert!(result.is_ok());

        let created_pick = result.unwrap();
        assert_eq!(created_pick.round, 1);
        assert_eq!(created_pick.pick_number, 1);
    }

    #[tokio::test]
    async fn test_find_next_pick() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        // Setup draft and team
        let draft_repo = SqlxDraftRepository::new(pool.clone());
        let draft = Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap();
        let created_draft = draft_repo.create(&draft).await.unwrap();

        let team_repo = SqlxTeamRepository::new(pool.clone());
        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();
        let created_team = team_repo.create(&team).await.unwrap();

        // Create two picks
        let pick_repo = SqlxDraftPickRepository::new(pool);
        let pick1 = DraftPick::new(created_draft.id, 1, 1, 1, created_team.id).unwrap();
        let pick2 = DraftPick::new(created_draft.id, 1, 2, 2, created_team.id).unwrap();

        pick_repo.create(&pick1).await.unwrap();
        pick_repo.create(&pick2).await.unwrap();

        // Next pick should be pick1
        let result = pick_repo.find_next_pick(created_draft.id).await;
        assert!(result.is_ok());

        let next = result.unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().overall_pick, 1);
    }
}
