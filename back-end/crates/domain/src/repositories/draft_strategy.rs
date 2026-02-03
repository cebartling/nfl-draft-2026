use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::DraftStrategy;

/// Repository trait for DraftStrategy data access
#[async_trait]
pub trait DraftStrategyRepository: Send + Sync {
    /// Create a new draft strategy
    async fn create(&self, strategy: &DraftStrategy) -> DomainResult<DraftStrategy>;

    /// Find a draft strategy by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftStrategy>>;

    /// Find a draft strategy by team and draft
    async fn find_by_team_and_draft(
        &self,
        team_id: Uuid,
        draft_id: Uuid,
    ) -> DomainResult<Option<DraftStrategy>>;

    /// Find all draft strategies for a draft
    async fn find_by_draft_id(&self, draft_id: Uuid) -> DomainResult<Vec<DraftStrategy>>;

    /// Update a draft strategy
    async fn update(&self, strategy: &DraftStrategy) -> DomainResult<DraftStrategy>;

    /// Delete a draft strategy
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}
