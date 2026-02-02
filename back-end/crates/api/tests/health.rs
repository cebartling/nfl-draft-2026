//! Health check acceptance tests

mod common;

use std::time::Duration;

#[tokio::test]
async fn test_health_check() {
    let base_url = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!("{}/health", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "healthy");
}
