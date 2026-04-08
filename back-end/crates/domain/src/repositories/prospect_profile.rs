use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::ProspectProfile;

/// Repository trait for ProspectProfile data access.
#[async_trait]
pub trait ProspectProfileRepository: Send + Sync {
    /// Insert a new prospect profile, or update the existing one keyed by
    /// (player_id, source). Returns the persisted row.
    async fn upsert(&self, profile: &ProspectProfile) -> DomainResult<ProspectProfile>;

    /// Look up the latest profile for a given player. If multiple sources
    /// exist, returns the most recently scraped one.
    async fn find_latest_by_player(
        &self,
        player_id: Uuid,
    ) -> DomainResult<Option<ProspectProfile>>;

    /// Look up a specific profile by (player_id, source).
    async fn find_by_player_and_source(
        &self,
        player_id: Uuid,
        source: &str,
    ) -> DomainResult<Option<ProspectProfile>>;

    /// All profiles from a given source, ordered by overall_rank then position_rank.
    async fn find_by_source(&self, source: &str) -> DomainResult<Vec<ProspectProfile>>;

    /// Delete every profile for the given source. Returns rows affected.
    async fn delete_by_source(&self, source: &str) -> DomainResult<u64>;
}
