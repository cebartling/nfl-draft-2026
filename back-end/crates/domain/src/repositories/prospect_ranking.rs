use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::{PlayerRankingWithSource, ProspectRanking};

/// Repository trait for ProspectRanking data access
#[async_trait]
pub trait ProspectRankingRepository: Send + Sync {
    /// Create a batch of prospect rankings
    async fn create_batch(&self, rankings: &[ProspectRanking]) -> DomainResult<usize>;

    /// Find all rankings for a player with source names pre-joined
    async fn find_by_player_with_source(
        &self,
        player_id: Uuid,
    ) -> DomainResult<Vec<PlayerRankingWithSource>>;

    /// Find all rankings across all sources with source names pre-joined
    async fn find_all_with_source(&self) -> DomainResult<Vec<PlayerRankingWithSource>>;

    /// Find all rankings for a player (across all sources)
    async fn find_by_player(&self, player_id: Uuid) -> DomainResult<Vec<ProspectRanking>>;

    /// Find all rankings for a source (the full big board)
    async fn find_by_source(&self, source_id: Uuid) -> DomainResult<Vec<ProspectRanking>>;

    /// Delete all rankings for a source
    async fn delete_by_source(&self, source_id: Uuid) -> DomainResult<u64>;
}
