use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::RankingSource;

/// Repository trait for RankingSource data access
#[async_trait]
pub trait RankingSourceRepository: Send + Sync {
    /// Create a new ranking source
    async fn create(&self, source: &RankingSource) -> DomainResult<RankingSource>;

    /// Find a ranking source by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<RankingSource>>;

    /// Find a ranking source by name
    async fn find_by_name(&self, name: &str) -> DomainResult<Option<RankingSource>>;

    /// Find all ranking sources
    async fn find_all(&self) -> DomainResult<Vec<RankingSource>>;
}
