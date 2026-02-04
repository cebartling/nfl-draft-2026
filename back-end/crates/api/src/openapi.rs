use utoipa::OpenApi;

use crate::handlers::{drafts, health, players, seed, teams, trades};
use domain::models::{ChartType, Conference, Division, DraftStatus, Position};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "NFL Draft Simulator API",
        version = "0.1.0",
        description = "API for simulating NFL drafts with real-time updates, AI-driven team decision-making, and comprehensive scouting systems",
        contact(
            name = "NFL Draft Simulator Team",
            email = "team@nfldraft.example.com"
        )
    ),
    paths(
        // Health
        health::health_check,

        // Teams
        teams::list_teams,
        teams::get_team,
        teams::create_team,

        // Players
        players::list_players,
        players::get_player,
        players::create_player,

        // Drafts
        drafts::create_draft,
        drafts::list_drafts,
        drafts::get_draft,
        drafts::initialize_draft_picks,
        drafts::get_draft_picks,
        drafts::get_next_pick,
        drafts::get_available_picks,
        drafts::start_draft,
        drafts::pause_draft,
        drafts::complete_draft,

        // Picks
        drafts::make_pick,

        // Trades
        trades::propose_trade,
        trades::accept_trade,
        trades::reject_trade,
        trades::get_trade,
        trades::get_pending_trades,

        // Admin
        seed::seed_players,
    ),
    components(
        schemas(
            // Domain models
            ChartType,
            Conference,
            Division,
            Position,
            DraftStatus,

            // Team types
            teams::TeamResponse,
            teams::CreateTeamRequest,

            // Player types
            players::PlayerResponse,
            players::CreatePlayerRequest,

            // Draft types
            drafts::DraftResponse,
            drafts::CreateDraftRequest,
            drafts::DraftPickResponse,
            drafts::MakePickRequest,

            // Trade types
            trades::TradeResponse,
            trades::TradeProposalResponse,
            trades::ProposeTradeRequest,
            trades::TradeActionRequest,

            // Admin types
            seed::SeedResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "teams", description = "NFL team management"),
        (name = "players", description = "Player management and scouting"),
        (name = "drafts", description = "Draft management and lifecycle"),
        (name = "picks", description = "Draft pick operations"),
        (name = "trades", description = "Draft pick trading operations"),
        (name = "admin", description = "Administrative operations"),
    )
)]
pub struct ApiDoc;
