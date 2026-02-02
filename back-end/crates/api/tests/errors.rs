//! Error handling acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_error_handling() {
    let base_url = common::spawn_app().await;
    let client = common::create_client();

    // Try to get non-existent team
    let response = client
        .get(&format!("{}/api/v1/teams/00000000-0000-0000-0000-000000000000", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), 404);

    // Try to create team with invalid data
    let response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "",
            "abbreviation": "DAL",
            "city": "Dallas",
            "conference": "NFC",
            "division": "NFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), 400);

    // Try to create duplicate draft year
    client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create first draft");

    let response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), 409);
}
