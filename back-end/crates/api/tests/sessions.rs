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
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
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
    // Without controlled_team_ids, should default to empty array
    assert_eq!(session["controlled_team_ids"], json!([]));

    let session_id: Uuid = serde_json::from_value(session["id"].clone()).unwrap();

    // Verify session was created in database
    let db_session = sqlx::query!("SELECT * FROM draft_sessions WHERE id = $1", session_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(db_session.draft_id, draft_id);
    assert_eq!(db_session.status, "NotStarted");
    assert!(db_session.controlled_team_ids.is_empty());

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
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
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
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
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
    let db_session = sqlx::query!("SELECT * FROM draft_sessions WHERE id = $1", session_id)
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
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
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
    let db_session = sqlx::query!("SELECT * FROM draft_sessions WHERE id = $1", session_id)
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
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
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
        .get(&format!(
            "{}/api/v1/sessions/{}/events",
            app_url, session_id
        ))
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
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
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
        .get(&format!(
            "{}/api/v1/sessions/{}/events",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    let events: Vec<Value> = events_response.json().await.unwrap();
    assert_eq!(events.len(), 3); // Created, Started, Paused

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_session_with_default_chart() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a draft first
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create session without specifying chart_type (should default to JimmyJohnson)
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
    assert_eq!(session["chart_type"], "JimmyJohnson");

    let session_id: Uuid = serde_json::from_value(session["id"].clone()).unwrap();

    // Verify in database
    let db_session = sqlx::query!(
        "SELECT chart_type FROM draft_sessions WHERE id = $1",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_session.chart_type, "JimmyJohnson");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_session_with_explicit_chart() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a draft first
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create session with RichHill chart
    let request_body = json!({
        "draft_id": draft_id,
        "time_per_pick_seconds": 300,
        "auto_pick_enabled": false,
        "chart_type": "RichHill"
    });

    let response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let session: Value = response.json().await.unwrap();
    assert_eq!(session["chart_type"], "RichHill");

    let session_id: Uuid = serde_json::from_value(session["id"].clone()).unwrap();

    // Verify in database
    let db_session = sqlx::query!(
        "SELECT chart_type FROM draft_sessions WHERE id = $1",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_session.chart_type, "RichHill");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_session_with_all_chart_types() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let charts = vec![
        "JimmyJohnson",
        "RichHill",
        "ChaseStudartAV",
        "FitzgeraldSpielberger",
        "PffWar",
        "SurplusValue",
    ];

    // Note: Drafts are created in the loop for simplicity. If the test panics,
    // cleanup_database() won't run and these drafts will remain. This is acceptable
    // since cleanup_database() is comprehensive and cleans all tables. A more robust
    // pattern would create all drafts upfront, but this matches the existing test style.
    for (idx, chart) in charts.iter().enumerate() {
        // Create a draft with unique year
        let draft_id = Uuid::new_v4();
        let year = 2026 + idx as i32;
        sqlx::query!(
            "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, $2, 'NotStarted', 7, 32::INTEGER)",
            draft_id,
            year
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create session with specific chart
        let request_body = json!({
            "draft_id": draft_id,
            "time_per_pick_seconds": 300,
            "auto_pick_enabled": false,
            "chart_type": chart
        });

        let response = client
            .post(&format!("{}/api/v1/sessions", app_url))
            .json(&request_body)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let session: Value = response.json().await.unwrap();
        assert_eq!(session["chart_type"], *chart);
    }

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_session_with_controlled_teams() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a draft and some teams
    let draft_id = Uuid::new_v4();
    let team_id_1 = Uuid::new_v4();
    let team_id_2 = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO teams (id, name, city, abbreviation, conference, division) VALUES ($1, 'Titans', 'Tennessee', 'TEN', 'AFC', 'AFC South'), ($2, 'Browns', 'Cleveland', 'CLE', 'AFC', 'AFC North')",
        team_id_1,
        team_id_2
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add draft picks so controlled teams are valid draft participants
    let pick_id_1 = Uuid::new_v4();
    let pick_id_2 = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3), ($4, $2, 1, 2, 2, $5)",
        pick_id_1,
        draft_id,
        team_id_1,
        pick_id_2,
        team_id_2
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create session with controlled_team_ids
    let request_body = json!({
        "draft_id": draft_id,
        "time_per_pick_seconds": 300,
        "auto_pick_enabled": true,
        "controlled_team_ids": [team_id_1, team_id_2]
    });

    let response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let session: Value = response.json().await.unwrap();
    let controlled_ids: Vec<String> =
        serde_json::from_value(session["controlled_team_ids"].clone()).unwrap();
    assert_eq!(controlled_ids.len(), 2);
    assert!(controlled_ids.contains(&team_id_1.to_string()));
    assert!(controlled_ids.contains(&team_id_2.to_string()));

    let session_id: Uuid = serde_json::from_value(session["id"].clone()).unwrap();

    // Verify in database
    let db_session = sqlx::query!(
        "SELECT controlled_team_ids FROM draft_sessions WHERE id = $1",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_session.controlled_team_ids.len(), 2);
    assert!(db_session.controlled_team_ids.contains(&team_id_1));
    assert!(db_session.controlled_team_ids.contains(&team_id_2));

    // Verify session_created event was recorded with controlled_team_ids
    let event = sqlx::query!(
        "SELECT event_type, event_data FROM draft_events WHERE session_id = $1 AND event_type = 'SessionCreated'",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(event.event_type, "SessionCreated");
    let event_data: Value = event.event_data;
    let settings = &event_data["settings"];
    assert!(settings["controlled_team_ids"].is_array());
    let event_team_ids: Vec<String> =
        serde_json::from_value(settings["controlled_team_ids"].clone()).unwrap();
    assert_eq!(event_team_ids.len(), 2);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_controlled_teams_persist_through_lifecycle() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a draft and a team
    let draft_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO teams (id, name, city, abbreviation, conference, division) VALUES ($1, 'Giants', 'New York', 'NYG', 'NFC', 'NFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add draft pick so controlled team is a valid draft participant
    let pick_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3)",
        pick_id,
        draft_id,
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create session with a controlled team
    let create_response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&json!({
            "draft_id": draft_id,
            "time_per_pick_seconds": 300,
            "auto_pick_enabled": true,
            "controlled_team_ids": [team_id]
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
    let started: Value = start_response.json().await.unwrap();
    let controlled_ids: Vec<String> =
        serde_json::from_value(started["controlled_team_ids"].clone()).unwrap();
    assert_eq!(controlled_ids.len(), 1);
    assert_eq!(controlled_ids[0], team_id.to_string());

    // Pause session
    let pause_response = client
        .post(&format!("{}/api/v1/sessions/{}/pause", app_url, session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(pause_response.status(), StatusCode::OK);
    let paused: Value = pause_response.json().await.unwrap();
    let controlled_ids: Vec<String> =
        serde_json::from_value(paused["controlled_team_ids"].clone()).unwrap();
    assert_eq!(controlled_ids.len(), 1);
    assert_eq!(controlled_ids[0], team_id.to_string());

    // Get session - verify controlled teams still there
    let get_response = client
        .get(&format!("{}/api/v1/sessions/{}", app_url, session_id))
        .send()
        .await
        .unwrap();

    let fetched: Value = get_response.json().await.unwrap();
    let controlled_ids: Vec<String> =
        serde_json::from_value(fetched["controlled_team_ids"].clone()).unwrap();
    assert_eq!(controlled_ids.len(), 1);
    assert_eq!(controlled_ids[0], team_id.to_string());

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_advance_pick() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft, team, and session in InProgress state
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 7, 32::INTEGER)",
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

    // Advance pick
    let response = client
        .post(&format!(
            "{}/api/v1/sessions/{}/advance-pick",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let session: Value = response.json().await.unwrap();
    assert_eq!(session["current_pick_number"], 2);

    // Verify in database
    let db_session = sqlx::query!(
        "SELECT current_pick_number FROM draft_sessions WHERE id = $1",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_session.current_pick_number, 2);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_advance_pick_requires_in_progress() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft and session in NotStarted state
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
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

    // Try to advance pick — should fail since session is not InProgress
    let response = client
        .post(&format!(
            "{}/api/v1/sessions/{}/advance-pick",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    // Should return an error (400 or 409)
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::CONFLICT
    );

    // Verify pick number unchanged in database
    let db_session = sqlx::query!(
        "SELECT current_pick_number FROM draft_sessions WHERE id = $1",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_session.current_pick_number, 1);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_auto_pick_run_stops_at_controlled_team() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft, two teams, players, picks, session with controlled_team_ids
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let ai_team_id = Uuid::new_v4();
    let user_team_id = Uuid::new_v4();
    let pick_1_id = Uuid::new_v4();
    let pick_2_id = Uuid::new_v4();
    let player_1_id = Uuid::new_v4();
    let player_2_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 2::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO teams (id, name, city, abbreviation, conference, division) VALUES ($1, 'AI Team', 'Test', 'AIT', 'AFC', 'AFC East'), ($2, 'User Team', 'Test', 'USR', 'NFC', 'NFC East')",
        ai_team_id,
        user_team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create players
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year) VALUES ($1, 'Player', 'One', 'QB', 2026), ($2, 'Player', 'Two', 'RB', 2026)",
        player_1_id,
        player_2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Pick 1 = AI team, Pick 2 = user-controlled team
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3)",
        pick_1_id,
        draft_id,
        ai_team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 2, 2, $3)",
        pick_2_id,
        draft_id,
        user_team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Session with user controlling user_team_id, current pick = 1
    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled, controlled_team_ids) VALUES ($1, $2, 'InProgress', 1, 300, true, $3)",
        session_id,
        draft_id,
        &[user_team_id]
    )
    .execute(&pool)
    .await
    .unwrap();

    // Run auto-pick
    let response = client
        .post(&format!(
            "{}/api/v1/sessions/{}/auto-pick-run",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let result: Value = response.json().await.unwrap();

    // Should have made 1 pick (AI team's pick) and stopped at user team's pick
    let picks_made = result["picks_made"].as_array().unwrap();
    assert_eq!(picks_made.len(), 1);

    // Session should be at pick 2 (user-controlled team's turn)
    assert_eq!(result["session"]["current_pick_number"], 2);

    // Verify pick 1 was made (has player_id) in database
    let db_pick_1 = sqlx::query!("SELECT player_id FROM draft_picks WHERE id = $1", pick_1_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_pick_1.player_id.is_some());

    // Verify pick 2 was NOT made (user-controlled)
    let db_pick_2 = sqlx::query!("SELECT player_id FROM draft_picks WHERE id = $1", pick_2_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_pick_2.player_id.is_none());

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_auto_pick_run_empty_when_user_controlled_first() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // First pick belongs to user-controlled team — auto-pick-run should do nothing
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let user_team_id = Uuid::new_v4();
    let pick_1_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 1::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO teams (id, name, city, abbreviation, conference, division) VALUES ($1, 'User Team', 'Test', 'USR', 'NFC', 'NFC East')",
        user_team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3)",
        pick_1_id,
        draft_id,
        user_team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled, controlled_team_ids) VALUES ($1, $2, 'InProgress', 1, 300, true, $3)",
        session_id,
        draft_id,
        &[user_team_id]
    )
    .execute(&pool)
    .await
    .unwrap();

    // Run auto-pick
    let response = client
        .post(&format!(
            "{}/api/v1/sessions/{}/auto-pick-run",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let result: Value = response.json().await.unwrap();

    // No picks should have been made
    let picks_made = result["picks_made"].as_array().unwrap();
    assert!(picks_made.is_empty());

    // Session should still be at pick 1
    assert_eq!(result["session"]["current_pick_number"], 1);

    // Verify pick was NOT made in database
    let db_pick = sqlx::query!("SELECT player_id FROM draft_picks WHERE id = $1", pick_1_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_pick.player_id.is_none());

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_start_session_transitions_draft_to_in_progress() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft in NotStarted state
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
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

    // Verify draft starts as NotStarted
    let db_draft = sqlx::query!("SELECT status FROM drafts WHERE id = $1", draft_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_draft.status, "NotStarted");

    // Start session
    let response = client
        .post(&format!("{}/api/v1/sessions/{}/start", app_url, session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let session: Value = response.json().await.unwrap();
    assert_eq!(session["status"], "InProgress");

    // Verify both session AND draft are now InProgress in the database
    let db_session = sqlx::query!("SELECT status FROM draft_sessions WHERE id = $1", session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_session.status, "InProgress");

    let db_draft = sqlx::query!("SELECT status FROM drafts WHERE id = $1", draft_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_draft.status, "InProgress");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_start_session_skips_draft_already_in_progress() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft already InProgress (e.g., resumed after pause)
    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 7, 32::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Session was paused and is being restarted
    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled) VALUES ($1, $2, 'Paused', 1, 300, false)",
        session_id,
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Start session (resume from paused)
    let response = client
        .post(&format!("{}/api/v1/sessions/{}/start", app_url, session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Draft should still be InProgress (not double-started)
    let db_draft = sqlx::query!("SELECT status FROM drafts WHERE id = $1", draft_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_draft.status, "InProgress");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_session_with_nonexistent_draft() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Try to create session with a draft_id that doesn't exist
    let fake_draft_id = Uuid::new_v4();

    let request_body = json!({
        "draft_id": fake_draft_id,
        "time_per_pick_seconds": 300,
        "auto_pick_enabled": false
    });

    let response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_session_with_nonexistent_team() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a draft but use a fake team_id in controlled_team_ids
    let draft_id = Uuid::new_v4();
    let fake_team_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let request_body = json!({
        "draft_id": draft_id,
        "time_per_pick_seconds": 300,
        "auto_pick_enabled": false,
        "controlled_team_ids": [fake_team_id]
    });

    let response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_get_session_by_draft_id() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create a session via API
    let create_response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&json!({
            "draft_id": draft_id,
            "time_per_pick_seconds": 300,
            "auto_pick_enabled": false,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created: Value = create_response.json().await.unwrap();
    let session_id: Uuid = serde_json::from_value(created["id"].clone()).unwrap();

    // Look up session by draft ID
    let response = client
        .get(&format!("{}/api/v1/drafts/{}/session", app_url, draft_id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let session: Value = response.json().await.unwrap();
    let returned_id: Uuid = serde_json::from_value(session["id"].clone()).unwrap();
    assert_eq!(returned_id, session_id);
    assert_eq!(session["draft_id"], draft_id.to_string());
    assert_eq!(session["status"], "NotStarted");

    // 404 for non-existent draft
    let missing = client
        .get(&format!(
            "{}/api/v1/drafts/{}/session",
            app_url,
            Uuid::new_v4()
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(missing.status(), StatusCode::NOT_FOUND);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_duplicate_session_returns_409() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a draft
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, 32::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let request_body = json!({
        "draft_id": draft_id,
        "time_per_pick_seconds": 300,
        "auto_pick_enabled": false
    });

    // First session creation — should succeed
    let first_response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(first_response.status(), StatusCode::CREATED);

    // Second session for the same draft — should fail with 409
    let second_response = client
        .post(&format!("{}/api/v1/sessions", app_url))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(second_response.status(), StatusCode::CONFLICT);

    // Verify only one session exists in the database
    let session_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_sessions WHERE draft_id = $1",
        draft_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(session_count.count.unwrap(), 1);

    common::cleanup_database(&pool).await;
}
