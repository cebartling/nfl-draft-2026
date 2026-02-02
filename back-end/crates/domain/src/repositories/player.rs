use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::{Player, Position};

/// Repository trait for Player data access
#[async_trait]
pub trait PlayerRepository: Send + Sync {
    /// Create a new player
    async fn create(&self, player: &Player) -> DomainResult<Player>;

    /// Find a player by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Player>>;

    /// Get all players
    async fn find_all(&self) -> DomainResult<Vec<Player>>;

    /// Find players by position
    async fn find_by_position(&self, position: Position) -> DomainResult<Vec<Player>>;

    /// Find players by draft year
    async fn find_by_draft_year(&self, year: i32) -> DomainResult<Vec<Player>>;

    /// Find draft eligible players
    async fn find_draft_eligible(&self, year: i32) -> DomainResult<Vec<Player>>;

    /// Update a player
    async fn update(&self, player: &Player) -> DomainResult<Player>;

    /// Delete a player
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}
