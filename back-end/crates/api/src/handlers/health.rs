use axum::Json;
use serde_json::{json, Value};

/// Health check endpoint
/// Returns 200 OK with status information
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = Value)
    ),
    tag = "health"
)]
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "nfl-draft-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        let value = response.0;

        assert_eq!(value["status"], "healthy");
        assert_eq!(value["service"], "nfl-draft-api");
        assert!(value["version"].is_string());
    }
}
