//! Error handling acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_error_handling() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Try to get non-existent team
    let response = client
        .get(&format!(
            "{}/api/v1/teams/00000000-0000-0000-0000-000000000000",
            base_url
        ))
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

    // Verify multiple drafts can be created for the same year
    let first_draft = client
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
    assert_eq!(first_draft.status(), 201);

    let second_draft = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create second draft");
    assert_eq!(second_draft.status(), 201);

    // Verify both drafts exist in database
    let db_draft_count = sqlx::query!("SELECT COUNT(*) as count FROM drafts WHERE year = 2026")
        .fetch_one(&pool)
        .await
        .expect("Failed to count drafts");
    assert_eq!(db_draft_count.count.unwrap(), 2);
}
