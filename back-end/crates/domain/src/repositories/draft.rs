use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::{Draft, DraftPick, DraftStatus};

/// Repository trait for Draft data access
///
/// This trait defines the interface for persisting and retrieving drafts.
/// Concrete implementations will be provided in the `db` crate.
#[async_trait]
pub trait DraftRepository: Send + Sync {
    /// Create a new draft
    async fn create(&self, draft: &Draft) -> DomainResult<Draft>;

    /// Find a draft by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Draft>>;

    /// Find drafts by year
    async fn find_by_year(&self, year: i32) -> DomainResult<Vec<Draft>>;

    /// Get all drafts
    async fn find_all(&self) -> DomainResult<Vec<Draft>>;

    /// Get drafts by status
    async fn find_by_status(&self, status: DraftStatus) -> DomainResult<Vec<Draft>>;

    /// Update a draft
    async fn update(&self, draft: &Draft) -> DomainResult<Draft>;

    /// Delete a draft
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}

/// Repository trait for DraftPick data access
///
/// This trait defines the interface for persisting and retrieving draft picks.
/// Concrete implementations will be provided in the `db` crate.
#[async_trait]
pub trait DraftPickRepository: Send + Sync {
    /// Create a new draft pick
    async fn create(&self, pick: &DraftPick) -> DomainResult<DraftPick>;

    /// Create multiple draft picks
    async fn create_many(&self, picks: &[DraftPick]) -> DomainResult<Vec<DraftPick>>;

    /// Find a draft pick by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftPick>>;

    /// Get all picks for a draft
    async fn find_by_draft_id(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>>;

    /// Get picks for a draft in a specific round
    async fn find_by_draft_and_round(
        &self,
        draft_id: Uuid,
        round: i32,
    ) -> DomainResult<Vec<DraftPick>>;

    /// Get picks for a specific team in a draft
    async fn find_by_draft_and_team(
        &self,
        draft_id: Uuid,
        team_id: Uuid,
    ) -> DomainResult<Vec<DraftPick>>;

    /// Get the next available pick for a draft
    async fn find_next_pick(&self, draft_id: Uuid) -> DomainResult<Option<DraftPick>>;

    /// Get all available (unpicked) picks for a draft
    async fn find_available_picks(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>>;

    /// Update a draft pick (e.g., after making a selection)
    async fn update(&self, pick: &DraftPick) -> DomainResult<DraftPick>;

    /// Delete a draft pick
    async fn delete(&self, id: Uuid) -> DomainResult<()>;

    /// Delete all picks for a draft
    async fn delete_by_draft_id(&self, draft_id: Uuid) -> DomainResult<()>;
}
