use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::{DomainError, DomainResult};
use domain::models::{DraftSession, SessionStatus};
use domain::repositories::SessionRepository;

#[derive(Debug, Clone, sqlx::FromRow)]
struct DraftSessionDb {
    id: Uuid,
    draft_id: Uuid,
    status: String,
    current_pick_number: i32,
    time_per_pick_seconds: i32,
    auto_pick_enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

impl From<DraftSessionDb> for DraftSession {
    fn from(db: DraftSessionDb) -> Self {
        let status = match db.status.as_str() {
            "NotStarted" => SessionStatus::NotStarted,
            "InProgress" => SessionStatus::InProgress,
            "Paused" => SessionStatus::Paused,
            "Completed" => SessionStatus::Completed,
            _ => SessionStatus::NotStarted, // Default fallback
        };

        DraftSession {
            id: db.id,
            draft_id: db.draft_id,
            status,
            current_pick_number: db.current_pick_number,
            time_per_pick_seconds: db.time_per_pick_seconds,
            auto_pick_enabled: db.auto_pick_enabled,
            created_at: db.created_at,
            updated_at: db.updated_at,
            started_at: db.started_at,
            completed_at: db.completed_at,
        }
    }
}

pub struct SessionRepo {
    pool: PgPool,
}

impl SessionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for SessionRepo {
    async fn create(&self, session: &DraftSession) -> DomainResult<DraftSession> {
        let db_session = sqlx::query_as!(
            DraftSessionDb,
            r#"
            INSERT INTO draft_sessions (
                id, draft_id, status, current_pick_number, time_per_pick_seconds,
                auto_pick_enabled, created_at, updated_at, started_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
            session.id,
            session.draft_id,
            session.status.to_string(),
            session.current_pick_number,
            session.time_per_pick_seconds,
            session.auto_pick_enabled,
            session.created_at,
            session.updated_at,
            session.started_at,
            session.completed_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(db_session.into())
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftSession>> {
        let result = sqlx::query_as!(
            DraftSessionDb,
            r#"
            SELECT * FROM draft_sessions
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_draft_id(&self, draft_id: Uuid) -> DomainResult<Option<DraftSession>> {
        let result = sqlx::query_as!(
            DraftSessionDb,
            r#"
            SELECT * FROM draft_sessions
            WHERE draft_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            draft_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn update(&self, session: &DraftSession) -> DomainResult<DraftSession> {
        let db_session = sqlx::query_as!(
            DraftSessionDb,
            r#"
            UPDATE draft_sessions
            SET status = $2,
                current_pick_number = $3,
                time_per_pick_seconds = $4,
                auto_pick_enabled = $5,
                updated_at = $6,
                started_at = $7,
                completed_at = $8
            WHERE id = $1
            RETURNING *
            "#,
            session.id,
            session.status.to_string(),
            session.current_pick_number,
            session.time_per_pick_seconds,
            session.auto_pick_enabled,
            session.updated_at,
            session.started_at,
            session.completed_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(db_session.into())
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM draft_sessions
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn list(&self) -> DomainResult<Vec<DraftSession>> {
        let sessions = sqlx::query_as!(
            DraftSessionDb,
            r#"
            SELECT * FROM draft_sessions
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(sessions.into_iter().map(Into::into).collect())
    }

    async fn list_by_status(&self, status: &str) -> DomainResult<Vec<DraftSession>> {
        let sessions = sqlx::query_as!(
            DraftSessionDb,
            r#"
            SELECT * FROM draft_sessions
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
            status
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(sessions.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_pool;

    async fn cleanup_sessions(pool: &PgPool) {
        sqlx::query!("DELETE FROM draft_sessions")
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_and_find_session() {
        let pool = get_test_pool().await;
        cleanup_sessions(&pool).await;

        let repo = SessionRepo::new(pool.clone());

        // Create a draft first
        let draft_id = Uuid::new_v4();
        let draft_year = 2026 + (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() % 100) as i32;
        sqlx::query!(
            "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, $2, 'NotStarted', 7, 32)",
            draft_id,
            draft_year
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create session
        let session = DraftSession::new(draft_id, 300, false).unwrap();
        let created = repo.create(&session).await.unwrap();

        assert_eq!(created.id, session.id);
        assert_eq!(created.draft_id, draft_id);
        assert_eq!(created.status, SessionStatus::NotStarted);

        // Find by ID
        let found = repo.find_by_id(session.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, session.id);

        // Find by draft ID
        let found_by_draft = repo.find_by_draft_id(draft_id).await.unwrap();
        assert!(found_by_draft.is_some());
        assert_eq!(found_by_draft.unwrap().draft_id, draft_id);

        cleanup_sessions(&pool).await;
        sqlx::query!("DELETE FROM drafts WHERE id = $1", draft_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_update_session() {
        let pool = get_test_pool().await;
        cleanup_sessions(&pool).await;

        let repo = SessionRepo::new(pool.clone());

        // Create a draft first
        let draft_id = Uuid::new_v4();
        let draft_year = 2026 + (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() % 100) as i32;
        sqlx::query!(
            "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, $2, 'NotStarted', 7, 32)",
            draft_id,
            draft_year
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create and start session
        let mut session = DraftSession::new(draft_id, 300, false).unwrap();
        repo.create(&session).await.unwrap();

        session.start().unwrap();
        let updated = repo.update(&session).await.unwrap();

        assert_eq!(updated.status, SessionStatus::InProgress);
        assert!(updated.started_at.is_some());

        cleanup_sessions(&pool).await;
        sqlx::query!("DELETE FROM drafts WHERE id = $1", draft_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let pool = get_test_pool().await;
        cleanup_sessions(&pool).await;

        let repo = SessionRepo::new(pool.clone());

        // Create drafts
        let draft_id_1 = Uuid::new_v4();
        let draft_id_2 = Uuid::new_v4();
        let base_year = 2026 + (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() % 100) as i32;

        sqlx::query!(
            "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, $2, 'NotStarted', 7, 32), ($3, $4, 'NotStarted', 7, 32)",
            draft_id_1,
            base_year,
            draft_id_2,
            base_year + 1
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create sessions
        let session1 = DraftSession::new(draft_id_1, 300, false).unwrap();
        let mut session2 = DraftSession::new(draft_id_2, 180, true).unwrap();
        session2.start().unwrap();

        repo.create(&session1).await.unwrap();
        repo.create(&session2).await.unwrap();

        // List all
        let all = repo.list().await.unwrap();
        assert_eq!(all.len(), 2);

        // List by status
        let in_progress = repo.list_by_status("InProgress").await.unwrap();
        assert_eq!(in_progress.len(), 1);
        assert_eq!(in_progress[0].status, SessionStatus::InProgress);

        cleanup_sessions(&pool).await;
        sqlx::query!("DELETE FROM drafts WHERE id IN ($1, $2)", draft_id_1, draft_id_2)
            .execute(&pool)
            .await
            .unwrap();
    }
}
