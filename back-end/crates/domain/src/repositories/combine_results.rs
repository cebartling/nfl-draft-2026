use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::CombineResults;

/// Repository trait for CombineResults data access
#[async_trait]
pub trait CombineResultsRepository: Send + Sync {
    /// Create new combine results
    async fn create(&self, results: &CombineResults) -> DomainResult<CombineResults>;

    /// Find combine results by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<CombineResults>>;

    /// Find all combine results for a player
    async fn find_by_player_id(&self, player_id: Uuid) -> DomainResult<Vec<CombineResults>>;

    /// Find combine results for a player and year
    async fn find_by_player_and_year(
        &self,
        player_id: Uuid,
        year: i32,
    ) -> DomainResult<Option<CombineResults>>;

    /// Find combine results for a player, year, and source
    async fn find_by_player_year_source(
        &self,
        player_id: Uuid,
        year: i32,
        source: &str,
    ) -> DomainResult<Option<CombineResults>>;

    /// Update combine results
    async fn update(&self, results: &CombineResults) -> DomainResult<CombineResults>;

    /// Delete combine results
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}
