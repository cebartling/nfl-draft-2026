use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use crate::openapi::ApiDoc;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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
        let state = AppState::new(pool);

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
