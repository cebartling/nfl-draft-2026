//! Draft state transition acceptance tests — invalid transitions should return 400

mod common;

use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

/// Helper: create a draft via SQL in a specific status
async fn create_draft_with_status(pool: &sqlx::PgPool, status: &str) -> Uuid {
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, $2, 1, 2::INTEGER)",
        draft_id,
        status
    )
    .execute(pool)
    .await
    .unwrap();
    draft_id
}

#[tokio::test]
async fn test_pause_not_started_draft_returns_400() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = create_draft_with_status(&pool, "NotStarted").await;

    let response = client
        .post(&format!("{}/api/v1/drafts/{}/pause", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);

    // Verify DB status unchanged
    let db_draft = sqlx::query!(
        "SELECT status FROM drafts WHERE id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_draft.status, "NotStarted");
}

#[tokio::test]
async fn test_complete_not_started_draft_returns_400() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = create_draft_with_status(&pool, "NotStarted").await;

    let response = client
        .post(&format!(
            "{}/api/v1/drafts/{}/complete",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);

    let db_draft = sqlx::query!(
        "SELECT status FROM drafts WHERE id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_draft.status, "NotStarted");
}

#[tokio::test]
async fn test_start_already_in_progress_draft_returns_400() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = create_draft_with_status(&pool, "InProgress").await;

    let response = client
        .post(&format!("{}/api/v1/drafts/{}/start", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);

    let db_draft = sqlx::query!(
        "SELECT status FROM drafts WHERE id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_draft.status, "InProgress");
}

#[tokio::test]
async fn test_pause_already_paused_draft_returns_400() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = create_draft_with_status(&pool, "Paused").await;

    let response = client
        .post(&format!("{}/api/v1/drafts/{}/pause", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);

    let db_draft = sqlx::query!(
        "SELECT status FROM drafts WHERE id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_draft.status, "Paused");
}

#[tokio::test]
async fn test_start_completed_draft_returns_400() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = create_draft_with_status(&pool, "Completed").await;

    let response = client
        .post(&format!("{}/api/v1/drafts/{}/start", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);

    let db_draft = sqlx::query!(
        "SELECT status FROM drafts WHERE id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_draft.status, "Completed");
}

#[tokio::test]
async fn test_reinitialize_picks_returns_400() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create 2 teams
    for (name, abbr) in [("Team X", "TMX"), ("Team Y", "TMY")] {
        client
            .post(&format!("{}/api/v1/teams", base_url))
            .json(&json!({
                "name": name,
                "abbreviation": abbr,
                "city": "City",
                "conference": "AFC",
                "division": "AFC East"
            }))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .expect("Failed to create team");
    }

    // Create draft with picks_per_round=2
    let draft_response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 1,
            "picks_per_round": 2
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create draft");
    assert_eq!(draft_response.status(), 201);

    let draft: serde_json::Value = draft_response.json().await.unwrap();
    let draft_id = draft["id"].as_str().unwrap();

    // First initialize → 201
    let first_init = client
        .post(&format!(
            "{}/api/v1/drafts/{}/initialize",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to initialize picks");
    assert_eq!(first_init.status(), 201);

    let picks: Vec<serde_json::Value> = first_init.json().await.unwrap();
    let original_count = picks.len();

    // Second initialize → 400
    let second_init = client
        .post(&format!(
            "{}/api/v1/drafts/{}/initialize",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send second initialize");
    assert_eq!(second_init.status(), 400);

    // Verify pick count unchanged
    let db_pick_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_picks WHERE draft_id = $1",
        Uuid::parse_str(draft_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_pick_count.count.unwrap() as usize, original_count);
}
