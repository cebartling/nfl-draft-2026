use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::CombinePercentile;

/// Repository trait for CombinePercentile data access
#[async_trait]
pub trait CombinePercentileRepository: Send + Sync {
    /// Find all percentiles
    async fn find_all(&self) -> DomainResult<Vec<CombinePercentile>>;

    /// Find percentiles by position
    async fn find_by_position(&self, position: &str) -> DomainResult<Vec<CombinePercentile>>;

    /// Find a specific percentile by position and measurement
    async fn find_by_position_and_measurement(
        &self,
        position: &str,
        measurement: &str,
    ) -> DomainResult<Option<CombinePercentile>>;

    /// Upsert a percentile record (insert or update on conflict)
    async fn upsert(&self, percentile: &CombinePercentile) -> DomainResult<CombinePercentile>;

    /// Delete all percentile records
    async fn delete_all(&self) -> DomainResult<u64>;

    /// Delete a percentile by ID
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}
