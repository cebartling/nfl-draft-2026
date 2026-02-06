//! Admin seed endpoint acceptance tests

mod common;

use std::time::Duration;

#[tokio::test]
async fn test_seed_teams_404_when_no_key_configured() {
    // Standard spawn_app() passes None for seed_api_key
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .post(&format!("{}/api/v1/admin/seed-teams", base_url))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_seed_teams_401_with_wrong_key() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("correct-key").await;
    let client = common::create_client();

    let response = client
        .post(&format!("{}/api/v1/admin/seed-teams", base_url))
        .header("X-Seed-Api-Key", "wrong-key")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_seed_teams_401_with_missing_key() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("correct-key").await;
    let client = common::create_client();

    // No X-Seed-Api-Key header
    let response = client
        .post(&format!("{}/api/v1/admin/seed-teams", base_url))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_seed_teams_succeeds() {
    let (base_url, pool) = common::spawn_app_with_seed_key("test-seed-key").await;
    let client = common::create_client();

    let response = client
        .post(&format!("{}/api/v1/admin/seed-teams", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(
        body["success_count"].as_u64().unwrap() > 0,
        "Expected some teams to be seeded"
    );

    // Verify teams exist in database
    let db_count = sqlx::query!("SELECT COUNT(*) as count FROM teams")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_count.count.unwrap() > 0);
}

#[tokio::test]
async fn test_seed_players_succeeds() {
    let (base_url, pool) = common::spawn_app_with_seed_key("test-seed-key").await;
    let client = common::create_client();

    let response = client
        .post(&format!("{}/api/v1/admin/seed-players", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(
        body["success_count"].as_u64().unwrap() > 0,
        "Expected some players to be seeded"
    );

    // Verify players exist in database
    let db_count = sqlx::query!("SELECT COUNT(*) as count FROM players")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_count.count.unwrap() > 0);
}

#[tokio::test]
async fn test_seed_team_seasons_succeeds() {
    let (base_url, pool) = common::spawn_app_with_seed_key("test-seed-key").await;
    let client = common::create_client();

    // Seed teams first (team_seasons requires teams)
    let seed_teams_response = client
        .post(&format!("{}/api/v1/admin/seed-teams", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to seed teams");
    assert_eq!(seed_teams_response.status(), 200);

    // Now seed team seasons
    let response = client
        .post(&format!("{}/api/v1/admin/seed-team-seasons", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(
        body["success_count"].as_u64().unwrap() > 0,
        "Expected some team seasons to be seeded"
    );

    // Verify team_seasons exist in database
    let db_count = sqlx::query!("SELECT COUNT(*) as count FROM team_seasons")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_count.count.unwrap() > 0);
}

#[tokio::test]
async fn test_seed_teams_idempotent() {
    let (base_url, pool) = common::spawn_app_with_seed_key("test-seed-key").await;
    let client = common::create_client();

    // First seed
    let first_response = client
        .post(&format!("{}/api/v1/admin/seed-teams", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send first seed");
    assert_eq!(first_response.status(), 200);

    let first_body: serde_json::Value = first_response.json().await.unwrap();
    let _first_success = first_body["success_count"].as_u64().unwrap();

    // Count teams after first seed
    let count_after_first = sqlx::query!("SELECT COUNT(*) as count FROM teams")
        .fetch_one(&pool)
        .await
        .unwrap()
        .count
        .unwrap();

    // Second seed
    let second_response = client
        .post(&format!("{}/api/v1/admin/seed-teams", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send second seed");
    assert_eq!(second_response.status(), 200);

    let second_body: serde_json::Value = second_response.json().await.unwrap();
    let second_skipped = second_body["skipped_count"].as_u64().unwrap();

    // Second call should skip everything
    assert!(
        second_skipped > 0,
        "Expected skipped_count > 0 on second seed, got {}",
        second_skipped
    );

    // DB count should be unchanged
    let count_after_second = sqlx::query!("SELECT COUNT(*) as count FROM teams")
        .fetch_one(&pool)
        .await
        .unwrap()
        .count
        .unwrap();
    assert_eq!(count_after_first, count_after_second);
}
