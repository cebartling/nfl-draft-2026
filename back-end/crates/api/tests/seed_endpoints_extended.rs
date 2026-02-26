//! Extended admin seed endpoint acceptance tests
//! Covers: seed-rankings, seed-combine-percentiles, seed-combine-data

mod common;

use std::time::Duration;

#[tokio::test]
async fn test_seed_rankings_succeeds() {
    let (base_url, pool) = common::spawn_app_with_seed_key("test-seed-key").await;
    let client = common::create_client();

    // Seed players first (rankings match by player name)
    let seed_players_response = client
        .post(&format!("{}/api/v1/admin/seed-players", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to seed players");
    assert_eq!(seed_players_response.status(), 200);

    // Seed teams (rankings may create scouting reports)
    let seed_teams_response = client
        .post(&format!("{}/api/v1/admin/seed-teams", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to seed teams");
    assert_eq!(seed_teams_response.status(), 200);

    // Seed rankings
    let response = client
        .post(&format!("{}/api/v1/admin/seed-rankings", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .expect("Failed to seed rankings");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(
        body["success_count"].as_u64().unwrap() > 0,
        "Expected some rankings to be seeded, got: {}",
        body
    );

    // Verify ranking_sources exist in database
    let source_count = sqlx::query!("SELECT COUNT(*) as count FROM ranking_sources")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(
        source_count.count.unwrap() > 0,
        "Expected ranking sources in DB"
    );

    // Verify prospect_rankings exist in database
    let ranking_count = sqlx::query!("SELECT COUNT(*) as count FROM prospect_rankings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(
        ranking_count.count.unwrap() > 0,
        "Expected prospect rankings in DB"
    );
}

#[tokio::test]
async fn test_seed_rankings_401_without_key() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("correct-key").await;
    let client = common::create_client();

    let response = client
        .post(&format!("{}/api/v1/admin/seed-rankings", base_url))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_seed_combine_percentiles_succeeds() {
    let (base_url, pool) = common::spawn_app_with_seed_key("test-seed-key").await;
    let client = common::create_client();

    let response = client
        .post(&format!(
            "{}/api/v1/admin/seed-combine-percentiles",
            base_url
        ))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to seed combine percentiles");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(
        body["success_count"].as_u64().unwrap() > 0,
        "Expected some percentiles to be seeded"
    );

    // Verify combine_percentiles exist in database
    let db_count = sqlx::query!("SELECT COUNT(*) as count FROM combine_percentiles")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_count.count.unwrap() > 0);
}

#[tokio::test]
async fn test_seed_combine_percentiles_401_without_key() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("correct-key").await;
    let client = common::create_client();

    let response = client
        .post(&format!(
            "{}/api/v1/admin/seed-combine-percentiles",
            base_url
        ))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_seed_combine_data_succeeds() {
    let (base_url, pool) = common::spawn_app_with_seed_key("test-seed-key").await;
    let client = common::create_client();

    // Seed players first (combine data matches by player name)
    let seed_players_response = client
        .post(&format!("{}/api/v1/admin/seed-players", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to seed players");
    assert_eq!(seed_players_response.status(), 200);

    // Seed combine data
    let response = client
        .post(&format!("{}/api/v1/admin/seed-combine-data", base_url))
        .header("X-Seed-Api-Key", "test-seed-key")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to seed combine data");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(
        body["success_count"].as_u64().unwrap() > 0,
        "Expected some combine data to be seeded"
    );

    // Verify combine_results exist in database
    let db_count = sqlx::query!("SELECT COUNT(*) as count FROM combine_results")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_count.count.unwrap() > 0);
}

#[tokio::test]
async fn test_seed_combine_data_401_without_key() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("correct-key").await;
    let client = common::create_client();

    let response = client
        .post(&format!("{}/api/v1/admin/seed-combine-data", base_url))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}
