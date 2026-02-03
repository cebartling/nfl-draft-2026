use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use domain::errors::{DomainError, DomainResult};
use domain::models::{DraftEvent, EventType};
use domain::repositories::EventRepository;

#[derive(Debug, Clone, sqlx::FromRow)]
struct DraftEventDb {
    id: Uuid,
    session_id: Uuid,
    event_type: String,
    event_data: JsonValue,
    created_at: DateTime<Utc>,
}

impl TryFrom<DraftEventDb> for DraftEvent {
    type Error = DomainError;

    fn try_from(db: DraftEventDb) -> Result<Self, Self::Error> {
        let event_type = EventType::from_str(&db.event_type)?;

        Ok(DraftEvent {
            id: db.id,
            session_id: db.session_id,
            event_type,
            event_data: db.event_data,
            created_at: db.created_at,
        })
    }
}

pub struct EventRepo {
    pool: PgPool,
}

impl EventRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for EventRepo {
    async fn create(&self, event: &DraftEvent) -> DomainResult<DraftEvent> {
        let db_event = sqlx::query_as!(
            DraftEventDb,
            r#"
            INSERT INTO draft_events (id, session_id, event_type, event_data, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, session_id, event_type, event_data, created_at
            "#,
            event.id,
            event.session_id,
            event.event_type.to_string(),
            event.event_data,
            event.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        db_event.try_into()
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftEvent>> {
        let result = sqlx::query_as!(
            DraftEventDb,
            r#"
            SELECT id, session_id, event_type, event_data, created_at
            FROM draft_events
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        match result {
            Some(db_event) => Ok(Some(db_event.try_into()?)),
            None => Ok(None),
        }
    }

    async fn list_by_session(&self, session_id: Uuid) -> DomainResult<Vec<DraftEvent>> {
        let events = sqlx::query_as!(
            DraftEventDb,
            r#"
            SELECT id, session_id, event_type, event_data, created_at
            FROM draft_events
            WHERE session_id = $1
            ORDER BY created_at ASC
            "#,
            session_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        events
            .into_iter()
            .map(|db| db.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn list_by_session_and_type(
        &self,
        session_id: Uuid,
        event_type: &str,
    ) -> DomainResult<Vec<DraftEvent>> {
        let events = sqlx::query_as!(
            DraftEventDb,
            r#"
            SELECT id, session_id, event_type, event_data, created_at
            FROM draft_events
            WHERE session_id = $1 AND event_type = $2
            ORDER BY created_at ASC
            "#,
            session_id,
            event_type
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        events
            .into_iter()
            .map(|db| db.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn count_by_session(&self, session_id: Uuid) -> DomainResult<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM draft_events
            WHERE session_id = $1
            "#,
            session_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(result.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_pool;

    async fn cleanup_events(pool: &PgPool) {
        sqlx::query!("DELETE FROM draft_events")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM draft_sessions")
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_and_find_event() {
        let pool = get_test_pool().await;
        cleanup_events(&pool).await;

        let repo = EventRepo::new(pool.clone());

        // Create draft and session first
        let draft_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32)",
            draft_id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled) VALUES ($1, $2, 'NotStarted', 1, 300, false)",
            session_id,
            draft_id
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create event
        let event = DraftEvent::session_created(session_id, draft_id, serde_json::json!({"test": "data"}));
        let created = repo.create(&event).await.unwrap();

        assert_eq!(created.id, event.id);
        assert_eq!(created.session_id, session_id);
        assert_eq!(created.event_type, EventType::SessionCreated);

        // Find by ID
        let found = repo.find_by_id(event.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, event.id);

        cleanup_events(&pool).await;
        sqlx::query!("DELETE FROM drafts WHERE id = $1", draft_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_list_by_session() {
        let pool = get_test_pool().await;
        cleanup_events(&pool).await;

        let repo = EventRepo::new(pool.clone());

        // Create draft and session first
        let draft_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32)",
            draft_id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled) VALUES ($1, $2, 'NotStarted', 1, 300, false)",
            session_id,
            draft_id
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create events
        let event1 = DraftEvent::session_created(session_id, draft_id, serde_json::json!({}));
        let event2 = DraftEvent::session_started(session_id);
        let event3 = DraftEvent::clock_update(session_id, 120);

        repo.create(&event1).await.unwrap();
        repo.create(&event2).await.unwrap();
        repo.create(&event3).await.unwrap();

        // List all events for session
        let events = repo.list_by_session(session_id).await.unwrap();
        assert_eq!(events.len(), 3);

        // Events should be ordered by created_at
        assert_eq!(events[0].event_type, EventType::SessionCreated);
        assert_eq!(events[1].event_type, EventType::SessionStarted);
        assert_eq!(events[2].event_type, EventType::ClockUpdate);

        // Count events
        let count = repo.count_by_session(session_id).await.unwrap();
        assert_eq!(count, 3);

        cleanup_events(&pool).await;
        sqlx::query!("DELETE FROM drafts WHERE id = $1", draft_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_list_by_session_and_type() {
        let pool = get_test_pool().await;
        cleanup_events(&pool).await;

        let repo = EventRepo::new(pool.clone());

        // Create draft and session first
        let draft_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32)",
            draft_id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled) VALUES ($1, $2, 'NotStarted', 1, 300, false)",
            session_id,
            draft_id
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create mixed events
        let event1 = DraftEvent::clock_update(session_id, 120);
        let event2 = DraftEvent::session_started(session_id);
        let event3 = DraftEvent::clock_update(session_id, 60);

        repo.create(&event1).await.unwrap();
        repo.create(&event2).await.unwrap();
        repo.create(&event3).await.unwrap();

        // List only ClockUpdate events
        let clock_events = repo
            .list_by_session_and_type(session_id, "ClockUpdate")
            .await
            .unwrap();

        assert_eq!(clock_events.len(), 2);
        assert!(clock_events.iter().all(|e| e.event_type == EventType::ClockUpdate));

        cleanup_events(&pool).await;
        sqlx::query!("DELETE FROM drafts WHERE id = $1", draft_id)
            .execute(&pool)
            .await
            .unwrap();
    }
}
