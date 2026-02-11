//! Acceptance tests for ranking endpoints

mod common;

use serde_json::json;
use std::time::Duration;

/// Helper: create a player via API and return its UUID
async fn create_player(client: &reqwest::Client, base_url: &str) -> String {
    let resp = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Shedeur",
            "last_name": "Sanders",
            "position": "QB",
            "college": "Colorado",
            "draft_year": 2026,
            "draft_eligible": true
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

/// Helper: create a second player via API
async fn create_player2(client: &reqwest::Client, base_url: &str) -> String {
    let resp = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Cam",
            "last_name": "Ward",
            "position": "QB",
            "college": "Miami",
            "draft_year": 2026,
            "draft_eligible": true
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

/// Helper: insert a ranking source directly into the database
async fn insert_ranking_source(
    pool: &sqlx::PgPool,
    name: &str,
    url: Option<&str>,
) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO ranking_sources (id, name, url) VALUES ($1, $2, $3)",
        id,
        name,
        url
    )
    .execute(pool)
    .await
    .expect("Failed to insert ranking source");
    id
}

/// Helper: insert a prospect ranking directly into the database
async fn insert_ranking(
    pool: &sqlx::PgPool,
    source_id: uuid::Uuid,
    player_id: uuid::Uuid,
    rank: i32,
) {
    let id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO prospect_rankings (id, ranking_source_id, player_id, rank, scraped_at) VALUES ($1, $2, $3, $4, '2026-02-11')",
        id,
        source_id,
        player_id,
        rank
    )
    .execute(pool)
    .await
    .expect("Failed to insert ranking");
}

#[tokio::test]
async fn test_list_ranking_sources_empty() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let resp = client
        .get(&format!("{}/api/v1/ranking-sources", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list ranking sources");

    assert_eq!(resp.status(), 200);
    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_list_ranking_sources_with_data() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let source_id = insert_ranking_source(&pool, "Tankathon", Some("https://tankathon.com")).await;

    let resp = client
        .get(&format!("{}/api/v1/ranking-sources", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list ranking sources");

    assert_eq!(resp.status(), 200);
    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["name"], "Tankathon");
    assert_eq!(body[0]["url"], "https://tankathon.com");
    assert_eq!(body[0]["id"], source_id.to_string());
}

#[tokio::test]
async fn test_get_player_rankings_empty() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let player_id = create_player(&client, &base_url).await;

    let resp = client
        .get(&format!(
            "{}/api/v1/players/{}/rankings",
            base_url, player_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get player rankings");

    assert_eq!(resp.status(), 200);
    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_get_player_rankings_with_data() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let player_id = create_player(&client, &base_url).await;
    let player_uuid = uuid::Uuid::parse_str(&player_id).unwrap();

    let source1_id = insert_ranking_source(&pool, "Tankathon", None).await;
    let source2_id = insert_ranking_source(&pool, "Walter Football", None).await;

    insert_ranking(&pool, source1_id, player_uuid, 1).await;
    insert_ranking(&pool, source2_id, player_uuid, 3).await;

    let resp = client
        .get(&format!(
            "{}/api/v1/players/{}/rankings",
            base_url, player_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get player rankings");

    assert_eq!(resp.status(), 200);
    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 2);

    // Should be ordered by rank
    assert_eq!(body[0]["rank"], 1);
    assert_eq!(body[0]["source_name"], "Tankathon");
    assert_eq!(body[1]["rank"], 3);
    assert_eq!(body[1]["source_name"], "Walter Football");
}

#[tokio::test]
async fn test_get_source_rankings() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let player1_id = create_player(&client, &base_url).await;
    let player2_id = create_player2(&client, &base_url).await;
    let player1_uuid = uuid::Uuid::parse_str(&player1_id).unwrap();
    let player2_uuid = uuid::Uuid::parse_str(&player2_id).unwrap();

    let source_id = insert_ranking_source(&pool, "Tankathon", None).await;

    insert_ranking(&pool, source_id, player1_uuid, 1).await;
    insert_ranking(&pool, source_id, player2_uuid, 2).await;

    let resp = client
        .get(&format!(
            "{}/api/v1/ranking-sources/{}/rankings",
            base_url, source_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get source rankings");

    assert_eq!(resp.status(), 200);
    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 2);

    // Should be ordered by rank
    assert_eq!(body[0]["rank"], 1);
    assert_eq!(body[0]["player_id"], player1_id);
    assert_eq!(body[1]["rank"], 2);
    assert_eq!(body[1]["player_id"], player2_id);
}

#[tokio::test]
async fn test_get_source_rankings_nonexistent_source_returns_404() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let fake_id = uuid::Uuid::new_v4();

    let resp = client
        .get(&format!(
            "{}/api/v1/ranking-sources/{}/rankings",
            base_url, fake_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get source rankings");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_get_all_rankings() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let player1_id = create_player(&client, &base_url).await;
    let player2_id = create_player2(&client, &base_url).await;
    let player1_uuid = uuid::Uuid::parse_str(&player1_id).unwrap();
    let player2_uuid = uuid::Uuid::parse_str(&player2_id).unwrap();

    let source1_id = insert_ranking_source(&pool, "Tankathon", None).await;
    let source2_id = insert_ranking_source(&pool, "Walter Football", None).await;

    insert_ranking(&pool, source1_id, player1_uuid, 1).await;
    insert_ranking(&pool, source1_id, player2_uuid, 2).await;
    insert_ranking(&pool, source2_id, player1_uuid, 2).await;
    insert_ranking(&pool, source2_id, player2_uuid, 1).await;

    let resp = client
        .get(&format!("{}/api/v1/rankings", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get all rankings");

    assert_eq!(resp.status(), 200);
    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 4);

    // Verify each entry has the expected fields
    for entry in &body {
        assert!(entry["player_id"].is_string());
        assert!(entry["source_name"].is_string());
        assert!(entry["rank"].is_number());
        assert!(entry["scraped_at"].is_string());
    }

    // Verify the Tankathon entries come first (ordered by source name, then rank)
    assert_eq!(body[0]["source_name"], "Tankathon");
    assert_eq!(body[0]["rank"], 1);
    assert_eq!(body[1]["source_name"], "Tankathon");
    assert_eq!(body[1]["rank"], 2);
    assert_eq!(body[2]["source_name"], "Walter Football");
    assert_eq!(body[2]["rank"], 1);
    assert_eq!(body[3]["source_name"], "Walter Football");
    assert_eq!(body[3]["rank"], 2);
}

#[tokio::test]
async fn test_get_all_rankings_empty() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let resp = client
        .get(&format!("{}/api/v1/rankings", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get all rankings");

    assert_eq!(resp.status(), 200);
    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_ranking_sources_database_consistency() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let source_id =
        insert_ranking_source(&pool, "Test Source", Some("https://example.com")).await;

    // Verify via API
    let resp = client
        .get(&format!("{}/api/v1/ranking-sources", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list ranking sources");

    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 1);

    // Verify API response matches database
    let db_source = sqlx::query!(
        "SELECT id, name, url, description FROM ranking_sources WHERE id = $1",
        source_id
    )
    .fetch_one(&pool)
    .await
    .expect("Source not found in database");

    assert_eq!(body[0]["name"].as_str().unwrap(), db_source.name);
    assert_eq!(
        body[0]["url"].as_str().unwrap(),
        db_source.url.as_deref().unwrap()
    );
    assert_eq!(body[0]["id"].as_str().unwrap(), source_id.to_string());
}
