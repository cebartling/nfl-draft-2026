//! Player CRUD acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_create_and_get_player() {
    let (base_url, pool) = common::spawn_app().await;
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

    let created_player: serde_json::Value =
        create_response.json().await.expect("Failed to parse JSON");
    let player_id = created_player["id"].as_str().expect("Missing player id");

    // Validate player was persisted in database
    let db_player = sqlx::query!(
        "SELECT id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year FROM players WHERE id = $1",
        uuid::Uuid::parse_str(player_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Player not found in database");

    assert_eq!(db_player.first_name, "John");
    assert_eq!(db_player.last_name, "Doe");
    assert_eq!(db_player.position, "QB");
    assert_eq!(db_player.college, Some("Texas".to_string()));
    assert_eq!(db_player.height_inches, Some(75));
    assert_eq!(db_player.weight_pounds, Some(220));
    assert_eq!(db_player.draft_year, 2026);

    // Get player via API
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

    // Verify API response matches database
    assert_eq!(player["first_name"].as_str().unwrap(), db_player.first_name);
    assert_eq!(player["last_name"].as_str().unwrap(), db_player.last_name);
    assert_eq!(player["position"].as_str().unwrap(), db_player.position);
}
