//! Common test utilities for acceptance tests

use reqwest::Client;
use std::time::Duration;
use tokio::sync::oneshot;

/// Spawns the API server on an ephemeral port and returns the base URL and database pool
#[allow(dead_code)]
pub async fn spawn_app() -> (String, sqlx::PgPool) {
    // Setup database
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
    });

    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create pool");

    // Cleanup database
    cleanup_database(&pool).await;

    let state = api::state::AppState::new(pool.clone(), None);
    let app = api::routes::create_router(state);

    // Bind to ephemeral port (port 0)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to ephemeral port");

    let addr = listener.local_addr().expect("Failed to get local address");
    let base_url = format!("http://{}", addr);

    // Create channel to notify when server is ready
    let (tx, rx) = oneshot::channel();

    // Spawn server in background task
    tokio::spawn(async move {
        // Notify that server is about to start
        tx.send(()).unwrap();

        axum::serve(listener, app)
            .await
            .expect("Server failed to start");
    });

    // Wait for server to be ready
    rx.await.expect("Server failed to start");

    // Give server a moment to fully initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    (base_url, pool)
}

/// Cleans up the test database by deleting all data in the correct order
pub async fn cleanup_database(pool: &sqlx::PgPool) {
    // Delete in order of foreign key dependencies
    sqlx::query!("DELETE FROM pick_trade_details")
        .execute(pool)
        .await
        .expect("Failed to cleanup pick_trade_details");
    sqlx::query!("DELETE FROM pick_trades")
        .execute(pool)
        .await
        .expect("Failed to cleanup pick_trades");
    sqlx::query!("DELETE FROM draft_strategies")
        .execute(pool)
        .await
        .expect("Failed to cleanup draft_strategies");
    sqlx::query!("DELETE FROM draft_events")
        .execute(pool)
        .await
        .expect("Failed to cleanup draft_events");
    sqlx::query!("DELETE FROM draft_sessions")
        .execute(pool)
        .await
        .expect("Failed to cleanup draft_sessions");
    sqlx::query!("DELETE FROM draft_picks")
        .execute(pool)
        .await
        .expect("Failed to cleanup picks");
    sqlx::query!("DELETE FROM drafts")
        .execute(pool)
        .await
        .expect("Failed to cleanup drafts");
    sqlx::query!("DELETE FROM prospect_rankings")
        .execute(pool)
        .await
        .expect("Failed to cleanup prospect_rankings");
    sqlx::query!("DELETE FROM ranking_sources")
        .execute(pool)
        .await
        .expect("Failed to cleanup ranking_sources");
    sqlx::query!("DELETE FROM scouting_reports")
        .execute(pool)
        .await
        .expect("Failed to cleanup scouting_reports");
    sqlx::query!("DELETE FROM combine_results")
        .execute(pool)
        .await
        .expect("Failed to cleanup combine_results");
    sqlx::query!("DELETE FROM team_needs")
        .execute(pool)
        .await
        .expect("Failed to cleanup team_needs");
    sqlx::query!("DELETE FROM team_seasons")
        .execute(pool)
        .await
        .expect("Failed to cleanup team_seasons");
    sqlx::query!("DELETE FROM players")
        .execute(pool)
        .await
        .expect("Failed to cleanup players");
    sqlx::query!("DELETE FROM teams")
        .execute(pool)
        .await
        .expect("Failed to cleanup teams");
}

/// Creates a configured reqwest client with sensible defaults
#[allow(dead_code)]
pub fn create_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client")
}

/// Spawns the API server with a configured seed API key
#[allow(dead_code)]
pub async fn spawn_app_with_seed_key(key: &str) -> (String, sqlx::PgPool) {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
    });

    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create pool");

    cleanup_database(&pool).await;

    let state = api::state::AppState::new(pool.clone(), Some(key.to_string()));
    let app = api::routes::create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to ephemeral port");

    let addr = listener.local_addr().expect("Failed to get local address");
    let base_url = format!("http://{}", addr);

    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        tx.send(()).unwrap();
        axum::serve(listener, app)
            .await
            .expect("Server failed to start");
    });

    rx.await.expect("Server failed to start");
    tokio::time::sleep(Duration::from_millis(100)).await;

    (base_url, pool)
}

/// Sets up a test database pool (without spawning HTTP server)
/// Useful for integration tests that don't need HTTP
#[allow(dead_code)]
pub async fn setup_test_pool() -> sqlx::PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
    });

    db::create_pool(&database_url)
        .await
        .expect("Failed to create test pool")
}
