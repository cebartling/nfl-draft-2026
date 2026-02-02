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
        .route("/teams", get(handlers::teams::list_teams).post(handlers::teams::create_team))
        .route("/teams/{id}", get(handlers::teams::get_team))
        // Players
        .route("/players", get(handlers::players::list_players).post(handlers::players::create_player))
        .route("/players/{id}", get(handlers::players::get_player))
        // Drafts
        .route("/drafts", get(handlers::drafts::list_drafts).post(handlers::drafts::create_draft))
        .route("/drafts/{id}", get(handlers::drafts::get_draft))
        .route("/drafts/{id}/initialize", post(handlers::drafts::initialize_draft_picks))
        .route("/drafts/{id}/picks", get(handlers::drafts::get_draft_picks))
        .route("/drafts/{id}/picks/next", get(handlers::drafts::get_next_pick))
        .route("/drafts/{id}/picks/available", get(handlers::drafts::get_available_picks))
        .route("/drafts/{id}/start", post(handlers::drafts::start_draft))
        .route("/drafts/{id}/pause", post(handlers::drafts::pause_draft))
        .route("/drafts/{id}/complete", post(handlers::drafts::complete_draft))
        // Draft Picks
        .route("/picks/{id}/make", post(handlers::drafts::make_pick));

    // Create stateful routes
    let stateful_router = Router::new()
        .route("/health", get(handlers::health::health_check))
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
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
            });

        let pool = db::create_pool(&database_url).await.expect("Failed to create pool");
        let state = AppState::new(pool);

        create_router(state)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = setup_test_router().await;

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
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
