use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::models::ScoutingReport;

/// Repository trait for ScoutingReport data access
#[async_trait]
pub trait ScoutingReportRepository: Send + Sync {
    /// Create a new scouting report
    async fn create(&self, report: &ScoutingReport) -> DomainResult<ScoutingReport>;

    /// Find a scouting report by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<ScoutingReport>>;

    /// Find all scouting reports for a team
    async fn find_by_team_id(&self, team_id: Uuid) -> DomainResult<Vec<ScoutingReport>>;

    /// Find all scouting reports for a player
    async fn find_by_player_id(&self, player_id: Uuid) -> DomainResult<Vec<ScoutingReport>>;

    /// Find a specific scouting report for a team and player
    async fn find_by_team_and_player(
        &self,
        team_id: Uuid,
        player_id: Uuid,
    ) -> DomainResult<Option<ScoutingReport>>;

    /// Update a scouting report
    async fn update(&self, report: &ScoutingReport) -> DomainResult<ScoutingReport>;

    /// Delete a scouting report
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}
