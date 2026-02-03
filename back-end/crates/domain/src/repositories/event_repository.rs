use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::DraftEvent;

#[async_trait]
pub trait EventRepository: Send + Sync {
    /// Record a new draft event
    async fn create(&self, event: &DraftEvent) -> DomainResult<DraftEvent>;

    /// Find an event by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftEvent>>;

    /// List all events for a session (ordered by creation time)
    async fn list_by_session(&self, session_id: Uuid) -> DomainResult<Vec<DraftEvent>>;

    /// List events by session and type
    async fn list_by_session_and_type(
        &self,
        session_id: Uuid,
        event_type: &str,
    ) -> DomainResult<Vec<DraftEvent>>;

    /// Count events for a session
    async fn count_by_session(&self, session_id: Uuid) -> DomainResult<i64>;
}
