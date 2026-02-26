use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::FeldmanFreak;

/// Repository trait for Feldman Freaks data access
#[async_trait]
pub trait FeldmanFreakRepository: Send + Sync {
    /// Create a new Feldman Freak entry
    async fn create(&self, freak: &FeldmanFreak) -> DomainResult<FeldmanFreak>;

    /// Find a Feldman Freak entry by player ID
    async fn find_by_player(&self, player_id: Uuid) -> DomainResult<Option<FeldmanFreak>>;

    /// Find all Feldman Freak entries for a given year
    async fn find_by_year(&self, year: i32) -> DomainResult<Vec<FeldmanFreak>>;

    /// Find all Feldman Freak entries
    async fn find_all(&self) -> DomainResult<Vec<FeldmanFreak>>;

    /// Delete all Feldman Freak entries for a given year
    async fn delete_by_year(&self, year: i32) -> DomainResult<u64>;
}
