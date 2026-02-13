use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};
use crate::models::{Draft, DraftPick, Player, Team};
use crate::repositories::{
    DraftPickRepository, DraftRepository, PlayerRepository, TeamRepository, TeamSeasonRepository,
};
use crate::services::AutoPickService;

/// Draft engine service for managing draft operations
pub struct DraftEngine {
    draft_repo: Arc<dyn DraftRepository>,
    pick_repo: Arc<dyn DraftPickRepository>,
    team_repo: Arc<dyn TeamRepository>,
    player_repo: Arc<dyn PlayerRepository>,
    team_season_repo: Option<Arc<dyn TeamSeasonRepository>>,
    auto_pick_service: Option<Arc<AutoPickService>>,
}

impl DraftEngine {
    pub fn new(
        draft_repo: Arc<dyn DraftRepository>,
        pick_repo: Arc<dyn DraftPickRepository>,
        team_repo: Arc<dyn TeamRepository>,
        player_repo: Arc<dyn PlayerRepository>,
    ) -> Self {
        Self {
            draft_repo,
            pick_repo,
            team_repo,
            player_repo,
            team_season_repo: None,
            auto_pick_service: None,
        }
    }

    pub fn with_team_season_repo(
        mut self,
        team_season_repo: Arc<dyn TeamSeasonRepository>,
    ) -> Self {
        self.team_season_repo = Some(team_season_repo);
        self
    }

    pub fn with_auto_pick(mut self, auto_pick_service: Arc<AutoPickService>) -> Self {
        self.auto_pick_service = Some(auto_pick_service);
        self
    }

    /// Create a new custom draft with fixed picks per round
    pub async fn create_draft(
        &self,
        name: String,
        year: i32,
        rounds: i32,
        picks_per_round: i32,
    ) -> DomainResult<Draft> {
        let draft = Draft::new(name, year, rounds, picks_per_round)?;
        self.draft_repo.create(&draft).await
    }

    /// Create a realistic draft with variable-length rounds (picks loaded from data)
    pub async fn create_realistic_draft(
        &self,
        name: String,
        year: i32,
        rounds: i32,
    ) -> DomainResult<Draft> {
        let draft = Draft::new_realistic(name, year, rounds)?;
        self.draft_repo.create(&draft).await
    }

    /// Initialize draft picks for a draft
    /// This creates picks for all teams in standard draft order (reverse standings)
    /// If team_season_repo is configured and standings data exists, uses standings-based order
    /// Otherwise, falls back to default team order
    pub async fn initialize_picks(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>> {
        // Get the draft
        let draft = self.draft_repo.find_by_id(draft_id).await?.ok_or_else(|| {
            DomainError::NotFound(format!("Draft with id {} not found", draft_id))
        })?;

        // Check if picks have already been initialized
        let existing_picks = self.pick_repo.find_by_draft_id(draft_id).await?;
        if !existing_picks.is_empty() {
            return Err(DomainError::ValidationError(format!(
                "Draft picks have already been initialized for draft {}",
                draft_id
            )));
        }

        // Get teams in draft order (standings-based if available, otherwise default)
        let teams_in_order = self.get_teams_in_draft_order(draft.year).await?;

        if teams_in_order.is_empty() {
            return Err(DomainError::ValidationError(
                "Cannot initialize draft picks: no teams found".to_string(),
            ));
        }

        let picks_per_round = draft.picks_per_round.unwrap_or(teams_in_order.len() as i32);

        // Validate picks_per_round matches team count
        if teams_in_order.len() != picks_per_round as usize {
            return Err(DomainError::ValidationError(format!(
                "Draft configured for {} picks per round but {} teams exist",
                picks_per_round,
                teams_in_order.len()
            )));
        }

        // Generate all picks
        let mut picks = Vec::new();
        let mut overall_pick = 1;

        for round in 1..=draft.rounds {
            for (pick_num, team) in teams_in_order.iter().enumerate() {
                let pick = DraftPick::new(
                    draft_id,
                    round,
                    (pick_num + 1) as i32,
                    overall_pick,
                    team.id,
                )?;
                picks.push(pick);
                overall_pick += 1;
            }
        }

        // Create all picks in database
        self.pick_repo.create_many(&picks).await
    }

    /// Get teams in draft order
    /// Uses standings from previous season if available, otherwise returns default team order
    async fn get_teams_in_draft_order(&self, draft_year: i32) -> DomainResult<Vec<Team>> {
        // Get all teams first
        let all_teams = self.team_repo.find_all().await?;
        let team_count = all_teams.len();

        if all_teams.is_empty() {
            return Ok(vec![]);
        }

        // Try to get standings-based order if team_season_repo is configured
        if let Some(ref season_repo) = self.team_season_repo {
            let standings_year = draft_year - 1; // 2026 draft uses 2025 standings
            let seasons = season_repo
                .find_by_year_ordered_by_draft_position(standings_year)
                .await?;

            let seasons_count = seasons.len();

            // If we have standings for all teams, use that order
            if seasons_count == team_count {
                // Build team order from standings
                let team_map: std::collections::HashMap<Uuid, Team> =
                    all_teams.iter().map(|t| (t.id, t.clone())).collect();

                let ordered_teams: Vec<Team> = seasons
                    .iter()
                    .filter_map(|season| team_map.get(&season.team_id).cloned())
                    .collect();

                if ordered_teams.len() == team_map.len() {
                    tracing::info!(
                        "Using standings-based draft order from {} season",
                        standings_year
                    );
                    return Ok(ordered_teams);
                }
            }

            // Partial standings data - log warning and fall back
            if seasons_count > 0 {
                tracing::warn!(
                    "Partial standings data found for {} ({} of {} teams). Using default team order.",
                    standings_year,
                    seasons_count,
                    team_count
                );
            }
        }

        // Fall back to default team order
        tracing::info!(
            "No standings data for {} draft year. Using default team order.",
            draft_year
        );
        Ok(all_teams)
    }

    /// Get next available pick in the draft
    pub async fn get_next_pick(&self, draft_id: Uuid) -> DomainResult<Option<DraftPick>> {
        self.pick_repo.find_next_pick(draft_id).await
    }

    /// Get all available picks for a draft
    pub async fn get_available_picks(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>> {
        self.pick_repo.find_available_picks(draft_id).await
    }

    /// Get all picks for a draft
    pub async fn get_all_picks(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>> {
        self.pick_repo.find_by_draft_id(draft_id).await
    }

    /// Get available players for drafting (not yet picked in this draft)
    pub async fn get_available_players(
        &self,
        draft_id: Uuid,
        draft_year: i32,
    ) -> DomainResult<Vec<Player>> {
        // Get all players eligible for the draft year
        let all_players = self.player_repo.find_by_draft_year(draft_year).await?;

        // Get all picks for this draft
        let picks = self.pick_repo.find_by_draft_id(draft_id).await?;

        // Get IDs of already picked players
        let picked_player_ids: std::collections::HashSet<Uuid> =
            picks.iter().filter_map(|pick| pick.player_id).collect();

        // Filter out picked players
        let available_players = all_players
            .into_iter()
            .filter(|player| !picked_player_ids.contains(&player.id))
            .collect();

        Ok(available_players)
    }

    /// Make a draft pick
    pub async fn make_pick(&self, pick_id: Uuid, player_id: Uuid) -> DomainResult<DraftPick> {
        // Get the pick
        let mut pick =
            self.pick_repo.find_by_id(pick_id).await?.ok_or_else(|| {
                DomainError::NotFound(format!("Pick with id {} not found", pick_id))
            })?;

        // Verify player exists
        let player = self
            .player_repo
            .find_by_id(player_id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!("Player with id {} not found", player_id))
            })?;

        // Verify player is draft eligible for this draft year
        let draft = self
            .draft_repo
            .find_by_id(pick.draft_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("Draft not found".to_string()))?;

        if player.draft_year != draft.year {
            return Err(DomainError::ValidationError(format!(
                "Player is eligible for {} draft, not {}",
                player.draft_year, draft.year
            )));
        }

        if !player.draft_eligible {
            return Err(DomainError::ValidationError(
                "Player is not draft eligible".to_string(),
            ));
        }

        // Verify player has not already been drafted in this draft
        let existing_picks = self.pick_repo.find_by_draft_id(pick.draft_id).await?;
        let already_drafted = existing_picks
            .iter()
            .any(|p| p.player_id == Some(player_id));
        if already_drafted {
            return Err(DomainError::PlayerAlreadyDrafted(format!(
                "Player {} has already been drafted in this draft",
                player.full_name()
            )));
        }

        // Make the pick
        pick.make_pick(player_id)?;

        // Update in database
        self.pick_repo.update(&pick).await
    }

    /// Start a draft
    pub async fn start_draft(&self, draft_id: Uuid) -> DomainResult<Draft> {
        let mut draft = self.draft_repo.find_by_id(draft_id).await?.ok_or_else(|| {
            DomainError::NotFound(format!("Draft with id {} not found", draft_id))
        })?;

        draft.start()?;
        self.draft_repo.update(&draft).await
    }

    /// Pause a draft
    pub async fn pause_draft(&self, draft_id: Uuid) -> DomainResult<Draft> {
        let mut draft = self.draft_repo.find_by_id(draft_id).await?.ok_or_else(|| {
            DomainError::NotFound(format!("Draft with id {} not found", draft_id))
        })?;

        draft.pause()?;
        self.draft_repo.update(&draft).await
    }

    /// Complete a draft
    pub async fn complete_draft(&self, draft_id: Uuid) -> DomainResult<Draft> {
        let mut draft = self.draft_repo.find_by_id(draft_id).await?.ok_or_else(|| {
            DomainError::NotFound(format!("Draft with id {} not found", draft_id))
        })?;

        draft.complete()?;
        self.draft_repo.update(&draft).await
    }

    /// Get draft by ID
    pub async fn get_draft(&self, draft_id: Uuid) -> DomainResult<Option<Draft>> {
        self.draft_repo.find_by_id(draft_id).await
    }

    /// Get drafts by year
    pub async fn get_drafts_by_year(&self, year: i32) -> DomainResult<Vec<Draft>> {
        self.draft_repo.find_by_year(year).await
    }

    /// Get all drafts
    pub async fn get_all_drafts(&self) -> DomainResult<Vec<Draft>> {
        self.draft_repo.find_all().await
    }

    /// Execute an auto-pick decision for a given pick
    /// This uses the AI draft engine to select the best available player
    pub async fn execute_auto_pick(&self, pick_id: Uuid) -> DomainResult<DraftPick> {
        let auto_pick_service = self.auto_pick_service.as_ref().ok_or_else(|| {
            DomainError::InternalError("Auto-pick service not configured".to_string())
        })?;

        // Get the pick
        let pick =
            self.pick_repo.find_by_id(pick_id).await?.ok_or_else(|| {
                DomainError::NotFound(format!("Pick with id {} not found", pick_id))
            })?;

        // Get the draft
        let draft = self
            .draft_repo
            .find_by_id(pick.draft_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("Draft not found".to_string()))?;

        // Retry loop: if the chosen player was already drafted (race condition),
        // re-fetch available players and try again.
        const MAX_RETRIES: usize = 3;
        for attempt in 0..MAX_RETRIES {
            let available_players = self
                .get_available_players(pick.draft_id, draft.year)
                .await?;

            if available_players.is_empty() {
                return Err(DomainError::ValidationError(
                    "No available players to pick from".to_string(),
                ));
            }

            // Use auto-pick service to decide
            let (selected_player_id, _scores) = auto_pick_service
                .decide_pick(pick.team_id, pick.draft_id, &available_players)
                .await?;

            // Make the pick — retry if player was already drafted (race condition)
            match self.make_pick(pick_id, selected_player_id).await {
                Ok(result) => return Ok(result),
                Err(DomainError::PlayerAlreadyDrafted(_)) if attempt < MAX_RETRIES - 1 => {
                    tracing::warn!(
                        "Auto-pick retry {}: player already drafted, re-fetching",
                        attempt + 1
                    );
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(DomainError::InternalError(
            "Auto-pick failed after retries".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Conference, Division, Position, Team};
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        DraftRepo {}
        #[async_trait::async_trait]
        impl DraftRepository for DraftRepo {
            async fn create(&self, draft: &Draft) -> DomainResult<Draft>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Draft>>;
            async fn find_by_year(&self, year: i32) -> DomainResult<Vec<Draft>>;
            async fn find_all(&self) -> DomainResult<Vec<Draft>>;
            async fn find_by_status(&self, status: crate::models::DraftStatus) -> DomainResult<Vec<Draft>>;
            async fn update(&self, draft: &Draft) -> DomainResult<Draft>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
        }
    }

    mock! {
        DraftPickRepo {}
        #[async_trait::async_trait]
        impl DraftPickRepository for DraftPickRepo {
            async fn create(&self, pick: &DraftPick) -> DomainResult<DraftPick>;
            async fn create_many(&self, picks: &[DraftPick]) -> DomainResult<Vec<DraftPick>>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DraftPick>>;
            async fn find_by_draft_id(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>>;
            async fn find_by_draft_and_round(&self, draft_id: Uuid, round: i32) -> DomainResult<Vec<DraftPick>>;
            async fn find_by_draft_and_team(&self, draft_id: Uuid, team_id: Uuid) -> DomainResult<Vec<DraftPick>>;
            async fn find_next_pick(&self, draft_id: Uuid) -> DomainResult<Option<DraftPick>>;
            async fn find_available_picks(&self, draft_id: Uuid) -> DomainResult<Vec<DraftPick>>;
            async fn update(&self, pick: &DraftPick) -> DomainResult<DraftPick>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
            async fn delete_by_draft_id(&self, draft_id: Uuid) -> DomainResult<()>;
        }
    }

    mock! {
        TeamRepo {}
        #[async_trait::async_trait]
        impl TeamRepository for TeamRepo {
            async fn create(&self, team: &Team) -> DomainResult<Team>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Team>>;
            async fn find_by_abbreviation(&self, abbreviation: &str) -> DomainResult<Option<Team>>;
            async fn find_all(&self) -> DomainResult<Vec<Team>>;
            async fn update(&self, team: &Team) -> DomainResult<Team>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
        }
    }

    mock! {
        PlayerRepo {}
        #[async_trait::async_trait]
        impl PlayerRepository for PlayerRepo {
            async fn create(&self, player: &Player) -> DomainResult<Player>;
            async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Player>>;
            async fn find_all(&self) -> DomainResult<Vec<Player>>;
            async fn find_by_position(&self, position: Position) -> DomainResult<Vec<Player>>;
            async fn find_by_draft_year(&self, year: i32) -> DomainResult<Vec<Player>>;
            async fn find_draft_eligible(&self, year: i32) -> DomainResult<Vec<Player>>;
            async fn update(&self, player: &Player) -> DomainResult<Player>;
            async fn delete(&self, id: Uuid) -> DomainResult<()>;
        }
    }

    #[tokio::test]
    async fn test_create_draft() {
        let mut draft_repo = MockDraftRepo::new();

        draft_repo.expect_create().returning(|d| Ok(d.clone()));

        let engine = DraftEngine::new(
            Arc::new(draft_repo),
            Arc::new(MockDraftPickRepo::new()),
            Arc::new(MockTeamRepo::new()),
            Arc::new(MockPlayerRepo::new()),
        );

        let result = engine
            .create_draft("Test Draft".to_string(), 2026, 7, 32)
            .await;
        assert!(result.is_ok());

        let draft = result.unwrap();
        assert_eq!(draft.name, "Test Draft");
        assert_eq!(draft.year, 2026);
        assert_eq!(draft.rounds, 7);
        assert_eq!(draft.picks_per_round, Some(32));
    }

    #[tokio::test]
    async fn test_create_multiple_drafts_same_year() {
        let mut draft_repo = MockDraftRepo::new();

        draft_repo.expect_create().returning(|d| Ok(d.clone()));

        let engine = DraftEngine::new(
            Arc::new(draft_repo),
            Arc::new(MockDraftPickRepo::new()),
            Arc::new(MockTeamRepo::new()),
            Arc::new(MockPlayerRepo::new()),
        );

        let result1 = engine
            .create_draft("Draft 1".to_string(), 2026, 7, 32)
            .await;
        assert!(result1.is_ok());

        let result2 = engine
            .create_draft("Draft 2".to_string(), 2026, 7, 32)
            .await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_initialize_picks() {
        let draft = Draft::new("Test Draft".to_string(), 2026, 7, 2).unwrap();
        let draft_id = draft.id;

        let team1 = Team::new(
            "Team A".to_string(),
            "TMA".to_string(),
            "City A".to_string(),
            Conference::AFC,
            Division::AFCEast,
        )
        .unwrap();

        let team2 = Team::new(
            "Team B".to_string(),
            "TMB".to_string(),
            "City B".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();

        let mut draft_repo = MockDraftRepo::new();
        draft_repo
            .expect_find_by_id()
            .with(eq(draft_id))
            .returning(move |_| Ok(Some(draft.clone())));

        let mut team_repo = MockTeamRepo::new();
        team_repo
            .expect_find_all()
            .returning(move || Ok(vec![team1.clone(), team2.clone()]));

        let mut pick_repo = MockDraftPickRepo::new();
        pick_repo
            .expect_find_by_draft_id()
            .with(eq(draft_id))
            .returning(|_| Ok(vec![])); // No existing picks
        pick_repo
            .expect_create_many()
            .returning(|picks| Ok(picks.to_vec()));

        let engine = DraftEngine::new(
            Arc::new(draft_repo),
            Arc::new(pick_repo),
            Arc::new(team_repo),
            Arc::new(MockPlayerRepo::new()),
        );

        let result = engine.initialize_picks(draft_id).await;
        assert!(result.is_ok());

        let picks = result.unwrap();
        // 2 teams * 7 rounds = 14 picks
        assert_eq!(picks.len(), 14);
        assert_eq!(picks[0].overall_pick, 1);
        assert_eq!(picks[13].overall_pick, 14);
    }

    // --- make_pick tests ---

    fn make_test_draft() -> Draft {
        Draft::new("Test Draft".to_string(), 2026, 7, 32).unwrap()
    }

    fn make_test_player(draft_year: i32, eligible: bool) -> Player {
        let mut player = Player::new(
            "John".to_string(),
            "Doe".to_string(),
            Position::QB,
            draft_year,
        )
        .unwrap();
        player.draft_eligible = eligible;
        player
    }

    #[tokio::test]
    async fn test_make_pick_success() {
        let draft = make_test_draft();
        let draft_id = draft.id;
        let team_id = Uuid::new_v4();
        let pick = DraftPick::new(draft_id, 1, 1, 1, team_id).unwrap();
        let pick_id = pick.id;
        let player = make_test_player(2026, true);
        let player_id = player.id;

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_c = pick.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_id))
            .returning(move |_| Ok(Some(pick_c.clone())));
        pick_repo
            .expect_find_by_draft_id()
            .with(eq(draft_id))
            .returning(|_| Ok(vec![])); // No existing picks
        pick_repo
            .expect_update()
            .returning(|p| Ok(p.clone()));

        let mut draft_repo = MockDraftRepo::new();
        draft_repo
            .expect_find_by_id()
            .with(eq(draft_id))
            .returning(move |_| Ok(Some(draft.clone())));

        let mut player_repo = MockPlayerRepo::new();
        let player_c = player.clone();
        player_repo
            .expect_find_by_id()
            .with(eq(player_id))
            .returning(move |_| Ok(Some(player_c.clone())));

        let engine = DraftEngine::new(
            Arc::new(draft_repo),
            Arc::new(pick_repo),
            Arc::new(MockTeamRepo::new()),
            Arc::new(player_repo),
        );

        let result = engine.make_pick(pick_id, player_id).await;
        assert!(result.is_ok());
        let made_pick = result.unwrap();
        assert_eq!(made_pick.player_id, Some(player_id));
    }

    #[tokio::test]
    async fn test_make_pick_pick_not_found() {
        let pick_id = Uuid::new_v4();

        let mut pick_repo = MockDraftPickRepo::new();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_id))
            .returning(|_| Ok(None));

        let engine = DraftEngine::new(
            Arc::new(MockDraftRepo::new()),
            Arc::new(pick_repo),
            Arc::new(MockTeamRepo::new()),
            Arc::new(MockPlayerRepo::new()),
        );

        let result = engine.make_pick(pick_id, Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_make_pick_player_not_found() {
        let draft = make_test_draft();
        let pick = DraftPick::new(draft.id, 1, 1, 1, Uuid::new_v4()).unwrap();
        let pick_id = pick.id;
        let player_id = Uuid::new_v4();

        let mut pick_repo = MockDraftPickRepo::new();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_id))
            .returning(move |_| Ok(Some(pick.clone())));

        let mut player_repo = MockPlayerRepo::new();
        player_repo
            .expect_find_by_id()
            .with(eq(player_id))
            .returning(|_| Ok(None));

        let engine = DraftEngine::new(
            Arc::new(MockDraftRepo::new()),
            Arc::new(pick_repo),
            Arc::new(MockTeamRepo::new()),
            Arc::new(player_repo),
        );

        let result = engine.make_pick(pick_id, player_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_make_pick_wrong_draft_year() {
        let draft = make_test_draft(); // year = 2026
        let draft_id = draft.id;
        let pick = DraftPick::new(draft_id, 1, 1, 1, Uuid::new_v4()).unwrap();
        let pick_id = pick.id;
        let player = make_test_player(2025, true); // Wrong year
        let player_id = player.id;

        let mut pick_repo = MockDraftPickRepo::new();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_id))
            .returning(move |_| Ok(Some(pick.clone())));

        let mut draft_repo = MockDraftRepo::new();
        draft_repo
            .expect_find_by_id()
            .with(eq(draft_id))
            .returning(move |_| Ok(Some(draft.clone())));

        let mut player_repo = MockPlayerRepo::new();
        let player_c = player.clone();
        player_repo
            .expect_find_by_id()
            .with(eq(player_id))
            .returning(move |_| Ok(Some(player_c.clone())));

        let engine = DraftEngine::new(
            Arc::new(draft_repo),
            Arc::new(pick_repo),
            Arc::new(MockTeamRepo::new()),
            Arc::new(player_repo),
        );

        let result = engine.make_pick(pick_id, player_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => assert!(msg.contains("2025")),
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_make_pick_player_not_eligible() {
        let draft = make_test_draft();
        let draft_id = draft.id;
        let pick = DraftPick::new(draft_id, 1, 1, 1, Uuid::new_v4()).unwrap();
        let pick_id = pick.id;
        let player = make_test_player(2026, false); // Not eligible
        let player_id = player.id;

        let mut pick_repo = MockDraftPickRepo::new();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_id))
            .returning(move |_| Ok(Some(pick.clone())));

        let mut draft_repo = MockDraftRepo::new();
        draft_repo
            .expect_find_by_id()
            .with(eq(draft_id))
            .returning(move |_| Ok(Some(draft.clone())));

        let mut player_repo = MockPlayerRepo::new();
        let player_c = player.clone();
        player_repo
            .expect_find_by_id()
            .with(eq(player_id))
            .returning(move |_| Ok(Some(player_c.clone())));

        let engine = DraftEngine::new(
            Arc::new(draft_repo),
            Arc::new(pick_repo),
            Arc::new(MockTeamRepo::new()),
            Arc::new(player_repo),
        );

        let result = engine.make_pick(pick_id, player_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => assert!(msg.contains("not draft eligible")),
            e => panic!("Expected ValidationError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_make_pick_player_already_drafted() {
        let draft = make_test_draft();
        let draft_id = draft.id;
        let pick = DraftPick::new(draft_id, 1, 2, 2, Uuid::new_v4()).unwrap();
        let pick_id = pick.id;
        let player = make_test_player(2026, true);
        let player_id = player.id;

        // Another pick already has this player
        let mut existing_pick = DraftPick::new(draft_id, 1, 1, 1, Uuid::new_v4()).unwrap();
        existing_pick.make_pick(player_id).unwrap();

        let mut pick_repo = MockDraftPickRepo::new();
        let pick_c = pick.clone();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_id))
            .returning(move |_| Ok(Some(pick_c.clone())));
        pick_repo
            .expect_find_by_draft_id()
            .with(eq(draft_id))
            .returning(move |_| Ok(vec![existing_pick.clone()]));

        let mut draft_repo = MockDraftRepo::new();
        draft_repo
            .expect_find_by_id()
            .with(eq(draft_id))
            .returning(move |_| Ok(Some(draft.clone())));

        let mut player_repo = MockPlayerRepo::new();
        let player_c = player.clone();
        player_repo
            .expect_find_by_id()
            .with(eq(player_id))
            .returning(move |_| Ok(Some(player_c.clone())));

        let engine = DraftEngine::new(
            Arc::new(draft_repo),
            Arc::new(pick_repo),
            Arc::new(MockTeamRepo::new()),
            Arc::new(player_repo),
        );

        let result = engine.make_pick(pick_id, player_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::PlayerAlreadyDrafted(_)
        ));
    }

    #[tokio::test]
    async fn test_make_pick_draft_not_found() {
        let draft_id = Uuid::new_v4();
        let pick = DraftPick::new(draft_id, 1, 1, 1, Uuid::new_v4()).unwrap();
        let pick_id = pick.id;
        let player = make_test_player(2026, true);
        let player_id = player.id;

        let mut pick_repo = MockDraftPickRepo::new();
        pick_repo
            .expect_find_by_id()
            .with(eq(pick_id))
            .returning(move |_| Ok(Some(pick.clone())));

        let mut draft_repo = MockDraftRepo::new();
        draft_repo
            .expect_find_by_id()
            .with(eq(draft_id))
            .returning(|_| Ok(None));

        let mut player_repo = MockPlayerRepo::new();
        let player_c = player.clone();
        player_repo
            .expect_find_by_id()
            .with(eq(player_id))
            .returning(move |_| Ok(Some(player_c.clone())));

        let engine = DraftEngine::new(
            Arc::new(draft_repo),
            Arc::new(pick_repo),
            Arc::new(MockTeamRepo::new()),
            Arc::new(player_repo),
        );

        let result = engine.make_pick(pick_id, player_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_execute_auto_pick_no_service() {
        let engine = DraftEngine::new(
            Arc::new(MockDraftRepo::new()),
            Arc::new(MockDraftPickRepo::new()),
            Arc::new(MockTeamRepo::new()),
            Arc::new(MockPlayerRepo::new()),
        );
        // No auto_pick_service configured

        let result = engine.execute_auto_pick(Uuid::new_v4()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::InternalError(msg) => {
                assert!(msg.contains("Auto-pick service not configured"))
            }
            e => panic!("Expected InternalError, got {:?}", e),
        }
    }
}
