//! Draft creation validation acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_create_draft_invalid_year() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "name": "Test Draft",
            "year": 1999,
            "rounds": 7,
            "picks_per_round": 32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_create_draft_invalid_rounds() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "name": "Test Draft",
            "year": 2026,
            "rounds": 0,
            "picks_per_round": 32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_create_draft_invalid_picks_per_round() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "name": "Test Draft",
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 0
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_create_multiple_drafts_same_year() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create first draft
    let first_response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "name": "First Draft",
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create first draft");
    assert_eq!(first_response.status(), 201);

    // Create second draft for same year
    let second_response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "name": "Second Draft",
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create second draft");
    assert_eq!(second_response.status(), 201);

    // Verify both drafts exist
    let db_count = sqlx::query!("SELECT COUNT(*) as count FROM drafts WHERE year = 2026")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_count.count.unwrap(), 2);
}
