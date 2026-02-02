//! Player CRUD acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_create_and_get_player() {
    let base_url = common::spawn_app().await;
    let client = common::create_client();

    // Create player
    let create_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe",
            "position": "QB",
            "college": "Texas",
            "height_inches": 75,
            "weight_pounds": 220,
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    assert_eq!(create_response.status(), 201);

    let created_player: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let player_id = created_player["id"].as_str().expect("Missing player id");

    // Get player
    let get_response = client
        .get(&format!("{}/api/v1/players/{}", base_url, player_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get player");

    assert_eq!(get_response.status(), 200);

    let player: serde_json::Value = get_response.json().await.expect("Failed to parse JSON");
    assert_eq!(player["first_name"], "John");
    assert_eq!(player["last_name"], "Doe");
    assert_eq!(player["position"], "QB");
}
