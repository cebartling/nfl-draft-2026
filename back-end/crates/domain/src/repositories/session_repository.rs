use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::DraftSession;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Create a new draft session
    async fn create(&self, session: &DraftSession) -> DomainResult<DraftSession>;

    /// Find a session by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftSession>>;

    /// Find a session by draft ID
    async fn find_by_draft_id(&self, draft_id: Uuid) -> DomainResult<Option<DraftSession>>;

    /// Update an existing session
    async fn update(&self, session: &DraftSession) -> DomainResult<DraftSession>;

    /// Delete a session
    async fn delete(&self, id: Uuid) -> DomainResult<()>;

    /// List all sessions
    async fn list(&self) -> DomainResult<Vec<DraftSession>>;

    /// List sessions by status
    async fn list_by_status(&self, status: &str) -> DomainResult<Vec<DraftSession>>;
}
