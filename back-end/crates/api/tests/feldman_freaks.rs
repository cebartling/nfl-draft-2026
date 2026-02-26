//! Feldman Freaks endpoint acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_list_feldman_freaks_empty() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!(
            "{}/api/v1/feldman-freaks?year=2026",
            base_url
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(body.is_empty(), "Expected empty list when no data seeded");
}

#[tokio::test]
async fn test_list_feldman_freaks_with_data() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a player first
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Travis",
            "last_name": "Hunter",
            "position": "CB",
            "draft_year": 2026,
            "college": "Colorado"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(player_response.status(), 201);
    let player: serde_json::Value = player_response.json().await.unwrap();
    let player_id = player["id"].as_str().unwrap();

    // Insert a feldman_freaks record directly in DB
    let player_uuid: uuid::Uuid = player_id.parse().unwrap();
    sqlx::query!(
        "INSERT INTO feldman_freaks (player_id, year, rank, description, article_url) VALUES ($1, $2, $3, $4, $5)",
        player_uuid,
        2026,
        1,
        "Elite two-way player with rare athleticism",
        "https://example.com/feldman-freaks"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert feldman_freaks record");

    // Query the endpoint
    let response = client
        .get(&format!(
            "{}/api/v1/feldman-freaks?year=2026",
            base_url
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(body.len(), 1, "Expected 1 feldman freak entry");

    let entry = &body[0];
    assert_eq!(entry["player_id"].as_str().unwrap(), player_id);
    assert_eq!(entry["rank"].as_i64().unwrap(), 1);
    assert_eq!(
        entry["description"].as_str().unwrap(),
        "Elite two-way player with rare athleticism"
    );
    assert_eq!(
        entry["article_url"].as_str().unwrap(),
        "https://example.com/feldman-freaks"
    );

    // Verify DB consistency
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM feldman_freaks WHERE year = 2026"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_count.count.unwrap(), 1);
}

#[tokio::test]
async fn test_list_feldman_freaks_wrong_year_returns_empty() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a player and feldman freak for 2026
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Shedeur",
            "last_name": "Sanders",
            "position": "QB",
            "draft_year": 2026,
            "college": "Colorado"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(player_response.status(), 201);
    let player: serde_json::Value = player_response.json().await.unwrap();
    let player_uuid: uuid::Uuid = player["id"].as_str().unwrap().parse().unwrap();

    sqlx::query!(
        "INSERT INTO feldman_freaks (player_id, year, rank, description) VALUES ($1, $2, $3, $4)",
        player_uuid,
        2026,
        1,
        "Test entry"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert feldman_freaks record");

    // Query with a different year
    let response = client
        .get(&format!(
            "{}/api/v1/feldman-freaks?year=2025",
            base_url
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);
    let body: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(body.is_empty(), "Expected empty list for year with no data");
}

#[tokio::test]
async fn test_list_feldman_freaks_multiple_entries_ordered_by_rank() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create two players
    let player1_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Player",
            "last_name": "One",
            "position": "QB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(player1_response.status(), 201);
    let player1: serde_json::Value = player1_response.json().await.unwrap();
    let player1_uuid: uuid::Uuid = player1["id"].as_str().unwrap().parse().unwrap();

    let player2_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Player",
            "last_name": "Two",
            "position": "DE",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(player2_response.status(), 201);
    let player2: serde_json::Value = player2_response.json().await.unwrap();
    let player2_uuid: uuid::Uuid = player2["id"].as_str().unwrap().parse().unwrap();

    // Insert in reverse order (rank 2 first, then rank 1)
    sqlx::query!(
        "INSERT INTO feldman_freaks (player_id, year, rank, description) VALUES ($1, $2, $3, $4)",
        player2_uuid,
        2026,
        2,
        "Second ranked freak"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO feldman_freaks (player_id, year, rank, description) VALUES ($1, $2, $3, $4)",
        player1_uuid,
        2026,
        1,
        "Top ranked freak"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Query
    let response = client
        .get(&format!(
            "{}/api/v1/feldman-freaks?year=2026",
            base_url
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(body.len(), 2);

    // Verify ordered by rank ascending
    assert_eq!(body[0]["rank"].as_i64().unwrap(), 1);
    assert_eq!(body[1]["rank"].as_i64().unwrap(), 2);

    // Verify DB count matches
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM feldman_freaks WHERE year = 2026"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_count.count.unwrap(), 2);
}

#[tokio::test]
async fn test_seed_feldman_freaks_succeeds() {
    let (base_url, pool) = common::spawn_app_with_seed_key("test-seed-key").await;
    let client = common::create_client();

    // Seed players first (feldman_freaks references players by name matching)
    let seed_players_response = client
        .post(&format!("{}/api/v1/admin/seed-players", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to seed players");
    assert_eq!(seed_players_response.status(), 200);

    // Seed feldman freaks
    let response = client
        .post(&format!(
            "{}/api/v1/admin/seed-feldman-freaks",
            base_url
        ))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to seed feldman freaks");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(
        body["success_count"].as_u64().unwrap() > 0,
        "Expected some feldman freaks to be seeded"
    );

    // Verify data in database
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM feldman_freaks WHERE year = 2026"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(db_count.count.unwrap() > 0);

    // Verify the API endpoint returns the seeded data
    let list_response = client
        .get(&format!(
            "{}/api/v1/feldman-freaks?year=2026",
            base_url
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list feldman freaks");

    assert_eq!(list_response.status(), 200);
    let freaks: Vec<serde_json::Value> = list_response.json().await.unwrap();
    assert!(
        !freaks.is_empty(),
        "Expected seeded feldman freaks to appear in API response"
    );

    // Verify structure of returned entries
    let first = &freaks[0];
    assert!(first["player_id"].is_string());
    assert!(first["rank"].is_number());
    assert!(first["description"].is_string());
}
