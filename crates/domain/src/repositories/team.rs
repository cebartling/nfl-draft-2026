use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::Team;

/// Repository trait for Team data access
///
/// This trait defines the interface for persisting and retrieving teams.
/// Concrete implementations will be provided in the `db` crate.
#[async_trait]
pub trait TeamRepository: Send + Sync {
    /// Create a new team
    async fn create(&self, team: &Team) -> DomainResult<Team>;

    /// Find a team by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Team>>;

    /// Find a team by abbreviation
    async fn find_by_abbreviation(&self, abbreviation: &str) -> DomainResult<Option<Team>>;

    /// Get all teams
    async fn find_all(&self) -> DomainResult<Vec<Team>>;

    /// Update a team
    async fn update(&self, team: &Team) -> DomainResult<Team>;

    /// Delete a team
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}
