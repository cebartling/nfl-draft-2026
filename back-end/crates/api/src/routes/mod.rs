use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use crate::openapi::ApiDoc;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    create_router_with_cors(state, &[])
}

pub fn create_router_with_cors(state: AppState, cors_origins: &[String]) -> Router {
    let seed_api_key_header = "X-Seed-Api-Key".parse().unwrap();
    let allowed_methods = [Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS];
    let allowed_headers = [CONTENT_TYPE, AUTHORIZATION, seed_api_key_header];

    let cors = if cors_origins.is_empty() {
        // Default development origins
        let origins: Vec<HeaderValue> = [
            "http://localhost:5173",
            "http://localhost:3000",
            "http://localhost:8080",
        ]
        .iter()
        .map(|o| o.parse().unwrap())
        .collect();

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
    } else {
        let origins: Vec<HeaderValue> = cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
    };

    // API v1 routes
    let api_routes = Router::new()
        // Teams
        .route(
            "/teams",
            get(handlers::teams::list_teams).post(handlers::teams::create_team),
        )
        .route("/teams/{id}", get(handlers::teams::get_team))
        .route(
            "/teams/{team_id}/scouting-reports",
            get(handlers::scouting_reports::get_team_scouting_reports),
        )
        .route(
            "/teams/{team_id}/needs",
            get(handlers::team_needs::list_team_needs),
        )
        .route(
            "/teams/{team_id}/seasons/{year}",
            get(handlers::team_seasons::get_team_season),
        )
        // Players
        .route(
            "/players",
            get(handlers::players::list_players).post(handlers::players::create_player),
        )
        .route("/players/{id}", get(handlers::players::get_player))
        .route(
            "/players/{player_id}/combine-results",
            get(handlers::combine_results::get_player_combine_results),
        )
        .route(
            "/players/{player_id}/scouting-reports",
            get(handlers::scouting_reports::get_player_scouting_reports),
        )
        .route(
            "/players/{player_id}/rankings",
            get(handlers::rankings::get_player_rankings),
        )
        .route(
            "/players/{player_id}/ras",
            get(handlers::ras::get_player_ras),
        )
        // Drafts
        .route(
            "/drafts",
            get(handlers::drafts::list_drafts).post(handlers::drafts::create_draft),
        )
        .route("/drafts/{id}", get(handlers::drafts::get_draft))
        .route(
            "/drafts/{id}/initialize",
            post(handlers::drafts::initialize_draft_picks),
        )
        .route("/drafts/{id}/picks", get(handlers::drafts::get_draft_picks))
        .route(
            "/drafts/{id}/picks/next",
            get(handlers::drafts::get_next_pick),
        )
        .route(
            "/drafts/{id}/picks/available",
            get(handlers::drafts::get_available_picks),
        )
        .route(
            "/drafts/{id}/available-players",
            get(handlers::drafts::get_available_players),
        )
        .route(
            "/drafts/{id}/session",
            get(handlers::sessions::get_session_by_draft),
        )
        .route("/drafts/{id}/start", post(handlers::drafts::start_draft))
        .route("/drafts/{id}/pause", post(handlers::drafts::pause_draft))
        .route(
            "/drafts/{id}/complete",
            post(handlers::drafts::complete_draft),
        )
        // Draft Picks
        .route("/picks/{id}/make", post(handlers::drafts::make_pick))
        // Draft Sessions
        .route("/sessions", post(handlers::sessions::create_session))
        .route("/sessions/{id}", get(handlers::sessions::get_session))
        .route(
            "/sessions/{id}/start",
            post(handlers::sessions::start_session),
        )
        .route(
            "/sessions/{id}/pause",
            post(handlers::sessions::pause_session),
        )
        .route(
            "/sessions/{id}/events",
            get(handlers::sessions::get_session_events),
        )
        .route(
            "/sessions/{id}/auto-pick-run",
            post(handlers::sessions::auto_pick_run),
        )
        .route(
            "/sessions/{id}/advance-pick",
            post(handlers::sessions::advance_pick),
        )
        // Combine Results
        .route(
            "/combine-results",
            post(handlers::combine_results::create_combine_results),
        )
        .route(
            "/combine-results/{id}",
            get(handlers::combine_results::get_combine_results)
                .put(handlers::combine_results::update_combine_results)
                .delete(handlers::combine_results::delete_combine_results),
        )
        // Scouting Reports
        .route(
            "/scouting-reports",
            post(handlers::scouting_reports::create_scouting_report),
        )
        .route(
            "/scouting-reports/{id}",
            get(handlers::scouting_reports::get_scouting_report)
                .put(handlers::scouting_reports::update_scouting_report)
                .delete(handlers::scouting_reports::delete_scouting_report),
        )
        // Team Needs
        .route("/team-needs", post(handlers::team_needs::create_team_need))
        .route(
            "/team-needs/{id}",
            get(handlers::team_needs::get_team_need)
                .put(handlers::team_needs::update_team_need)
                .delete(handlers::team_needs::delete_team_need),
        )
        // Trades
        .route("/trades", post(handlers::trades::propose_trade))
        .route("/trades/{id}", get(handlers::trades::get_trade))
        .route("/trades/{id}/accept", post(handlers::trades::accept_trade))
        .route("/trades/{id}/reject", post(handlers::trades::reject_trade))
        .route(
            "/teams/{team_id}/pending-trades",
            get(handlers::trades::get_pending_trades),
        )
        // Team Seasons
        .route(
            "/team-seasons",
            get(handlers::team_seasons::list_team_seasons),
        )
        .route("/draft-order", get(handlers::team_seasons::get_draft_order))
        // Rankings
        .route("/rankings", get(handlers::rankings::get_all_rankings))
        .route(
            "/ranking-sources",
            get(handlers::rankings::list_ranking_sources),
        )
        .route(
            "/ranking-sources/{source_id}/rankings",
            get(handlers::rankings::get_source_rankings),
        )
        // Combine Percentiles
        .route(
            "/combine-percentiles",
            get(handlers::combine_percentiles::get_combine_percentiles),
        )
        // Admin
        .route("/admin/seed-players", post(handlers::seed::seed_players))
        .route("/admin/seed-teams", post(handlers::seed::seed_teams))
        .route(
            "/admin/seed-team-seasons",
            post(handlers::seed::seed_team_seasons),
        )
        .route("/admin/seed-rankings", post(handlers::seed::seed_rankings))
        .route(
            "/admin/seed-combine-percentiles",
            post(handlers::seed::seed_combine_percentiles),
        )
        .route(
            "/admin/seed-combine-data",
            post(handlers::seed::seed_combine_data),
        )
        .route(
            "/admin/seed-feldman-freaks",
            post(handlers::seed::seed_feldman_freaks),
        )
        .route(
            "/admin/seed-percentiles",
            post(handlers::combine_percentiles::seed_percentiles),
        )
        .route(
            "/admin/percentiles",
            delete(handlers::combine_percentiles::delete_all_percentiles),
        );

    // Create stateful routes
    let stateful_router = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/ws", get(handlers::websocket::ws_handler))
        .nest("/api/v1", api_routes)
        .with_state(state);

    // Swagger UI router (stateless)
    let swagger_router: Router = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .into();

    // Merge routers and add layers
    stateful_router
        .merge(swagger_router)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn setup_test_router() -> Router {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        let pool = db::create_pool(&database_url)
            .await
            .expect("Failed to create pool");
        let state = AppState::new(pool, None);

        create_router(state)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = setup_test_router().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_teams_endpoint_exists() {
        let app = setup_test_router().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/teams")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_players_endpoint_exists() {
        let app = setup_test_router().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/players")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
