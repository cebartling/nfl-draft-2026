use reqwest::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_create_session() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a draft first
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create session
    let request_body = json!({
        "draft_id": draft_id,
        "time_per_pick_seconds": 300,
        "auto_pick_enabled": false
    });

    let response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let session: Value = response.json().await.unwrap();
    assert_eq!(session["draft_id"], draft_id.to_string());
    assert_eq!(session["status"], "NotStarted");
    assert_eq!(session["time_per_pick_seconds"], 300);
    assert_eq!(session["auto_pick_enabled"], false);

    let session_id: Uuid = serde_json::from_value(session["id"].clone()).unwrap();

    // Verify session was created in database
    let db_session = sqlx::query!(
        "SELECT * FROM draft_sessions WHERE id = $1",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_session.draft_id, draft_id);
    assert_eq!(db_session.status, "NotStarted");

    // Verify session created event was recorded
    let event_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_events WHERE session_id = $1 AND event_type = 'SessionCreated'",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(event_count.count.unwrap(), 1);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_get_session() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft and session directly in database
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled) VALUES ($1, $2, 'NotStarted', 1, 300, false)",
        session_id,
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Get session via API
    let response = client
        .get(&format!("{}/api/v1/sessions/{}", app_url, session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let session: Value = response.json().await.unwrap();
    assert_eq!(session["id"], session_id.to_string());
    assert_eq!(session["draft_id"], draft_id.to_string());
    assert_eq!(session["status"], "NotStarted");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_start_session() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft and session
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled) VALUES ($1, $2, 'NotStarted', 1, 300, false)",
        session_id,
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Start session
    let response = client
        .post(&format!("{}/api/v1/sessions/{}/start", app_url, session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let session: Value = response.json().await.unwrap();
    assert_eq!(session["status"], "InProgress");

    // Verify in database
    let db_session = sqlx::query!(
        "SELECT * FROM draft_sessions WHERE id = $1",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_session.status, "InProgress");
    assert!(db_session.started_at.is_some());

    // Verify event was recorded
    let event_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_events WHERE session_id = $1 AND event_type = 'SessionStarted'",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(event_count.count.unwrap(), 1);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_pause_session() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft and session in progress
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled) VALUES ($1, $2, 'InProgress', 1, 300, false)",
        session_id,
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Pause session
    let response = client
        .post(&format!("{}/api/v1/sessions/{}/pause", app_url, session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let session: Value = response.json().await.unwrap();
    assert_eq!(session["status"], "Paused");

    // Verify in database
    let db_session = sqlx::query!(
        "SELECT * FROM draft_sessions WHERE id = $1",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_session.status, "Paused");

    // Verify event was recorded
    let event_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_events WHERE session_id = $1 AND event_type = 'SessionPaused'",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(event_count.count.unwrap(), 1);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_get_session_events() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft and session
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled) VALUES ($1, $2, 'NotStarted', 1, 300, false)",
        session_id,
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create some events
    sqlx::query!(
        "INSERT INTO draft_events (session_id, event_type, event_data) VALUES ($1, 'SessionCreated', $2)",
        session_id,
        json!({"draft_id": draft_id})
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_events (session_id, event_type, event_data) VALUES ($1, 'SessionStarted', $2)",
        session_id,
        json!({})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Get events via API
    let response = client
        .get(&format!("{}/api/v1/sessions/{}/events", app_url, session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let events: Vec<Value> = response.json().await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["event_type"], "SessionCreated");
    assert_eq!(events[1]["event_type"], "SessionStarted");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_session_lifecycle() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a draft
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create session
    let create_response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&json!({
            "draft_id": draft_id,
            "time_per_pick_seconds": 300,
            "auto_pick_enabled": false
        }))
        .send()
        .await
        .unwrap();

    let session: Value = create_response.json().await.unwrap();
    let session_id: Uuid = serde_json::from_value(session["id"].clone()).unwrap();

    // Start session
    let start_response = client
        .post(&format!("{}/api/v1/sessions/{}/start", app_url, session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(start_response.status(), StatusCode::OK);
    let started_session: Value = start_response.json().await.unwrap();
    assert_eq!(started_session["status"], "InProgress");

    // Pause session
    let pause_response = client
        .post(&format!("{}/api/v1/sessions/{}/pause", app_url, session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(pause_response.status(), StatusCode::OK);
    let paused_session: Value = pause_response.json().await.unwrap();
    assert_eq!(paused_session["status"], "Paused");

    // Verify all events were recorded
    let events_response = client
        .get(&format!("{}/api/v1/sessions/{}/events", app_url, session_id))
        .send()
        .await
        .unwrap();

    let events: Vec<Value> = events_response.json().await.unwrap();
    assert_eq!(events.len(), 3); // Created, Started, Paused

    common::cleanup_database(&pool).await;
}
