use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::TeamNeed;

/// Repository trait for TeamNeed data access
#[async_trait]
pub trait TeamNeedRepository: Send + Sync {
    /// Create a new team need
    async fn create(&self, need: &TeamNeed) -> DomainResult<TeamNeed>;

    /// Find a team need by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<TeamNeed>>;

    /// Find all team needs for a team, ordered by priority
    async fn find_by_team_id(&self, team_id: Uuid) -> DomainResult<Vec<TeamNeed>>;

    /// Update a team need
    async fn update(&self, need: &TeamNeed) -> DomainResult<TeamNeed>;

    /// Delete a team need
    async fn delete(&self, id: Uuid) -> DomainResult<()>;

    /// Delete all team needs for a team
    async fn delete_by_team_id(&self, team_id: Uuid) -> DomainResult<()>;
}
