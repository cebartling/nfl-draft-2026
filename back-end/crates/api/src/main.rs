use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::{AppState, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Starting NFL Draft API server");
    tracing::info!("Server will listen on: {}", config.server_address());

    // Create database pool
    let pool = db::create_pool(&config.database.url).await?;
    tracing::info!("Database connection pool created");

    // Create application state
    let state = AppState::new(pool, config.seed_api_key.clone());

    // Create router
    let app = api::routes::create_router(state);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&config.server_address()).await?;
    tracing::info!("Server listening on {}", config.server_address());

    // Run the server
    axum::serve(listener, app).await?;

    Ok(())
}
