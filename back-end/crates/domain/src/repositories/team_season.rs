use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::TeamSeason;

/// Repository trait for TeamSeason data access
///
/// This trait defines the interface for persisting and retrieving team season records.
/// Concrete implementations will be provided in the `db` crate.
#[async_trait]
pub trait TeamSeasonRepository: Send + Sync {
    /// Create a new team season record
    async fn create(&self, season: &TeamSeason) -> DomainResult<TeamSeason>;

    /// Create or update a team season record (upsert)
    async fn upsert(&self, season: &TeamSeason) -> DomainResult<TeamSeason>;

    /// Find a team season by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<TeamSeason>>;

    /// Find a team season by team ID and year
    async fn find_by_team_and_year(
        &self,
        team_id: Uuid,
        year: i32,
    ) -> DomainResult<Option<TeamSeason>>;

    /// Find all team seasons for a given year
    async fn find_by_year(&self, year: i32) -> DomainResult<Vec<TeamSeason>>;

    /// Find all team seasons for a given year ordered by draft position
    async fn find_by_year_ordered_by_draft_position(
        &self,
        year: i32,
    ) -> DomainResult<Vec<TeamSeason>>;

    /// Delete all team seasons for a given year
    async fn delete_by_year(&self, year: i32) -> DomainResult<()>;

    /// Delete a team season by ID
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}
