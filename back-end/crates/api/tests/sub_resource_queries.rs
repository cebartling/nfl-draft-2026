//! Filtered sub-resource endpoint acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn test_get_pending_trades_for_team() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create two teams
    let team1_id = Uuid::new_v4();
    let team2_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Trade Team A', 'TTA', 'Trade City A', 'AFC', 'AFC East')",
        team1_id
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Trade Team B', 'TTB', 'Trade City B', 'NFC', 'NFC East')",
        team2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create draft and session for trade context
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 2::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let session_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled) VALUES ($1, $2, 'InProgress', 1, 300, false)",
        session_id,
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create picks for both teams
    let pick1_id = Uuid::new_v4();
    let pick2_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3)",
        pick1_id,
        draft_id,
        team1_id
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 2, 2, $3)",
        pick2_id,
        draft_id,
        team2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Propose a trade via API
    let trade_response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id,
            "from_team_id": team1_id,
            "to_team_id": team2_id,
            "from_team_picks": [pick1_id],
            "to_team_picks": [pick2_id]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose trade");
    assert_eq!(trade_response.status(), 201);

    // The to_team (team2) should see the pending trade
    let team2_pending = client
        .get(&format!(
            "{}/api/v1/teams/{}/pending-trades",
            base_url, team2_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team2 pending trades");
    assert_eq!(team2_pending.status(), 200);
    let team2_trades: Vec<serde_json::Value> = team2_pending.json().await.unwrap();
    assert_eq!(team2_trades.len(), 1);

    // The from_team (team1) should NOT see it as pending (only to_team receives pending trades)
    let team1_pending = client
        .get(&format!(
            "{}/api/v1/teams/{}/pending-trades",
            base_url, team1_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team1 pending trades");
    assert_eq!(team1_pending.status(), 200);
    let team1_trades: Vec<serde_json::Value> = team1_pending.json().await.unwrap();
    assert_eq!(team1_trades.len(), 0);
}

#[tokio::test]
async fn test_get_pending_trades_empty() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let team_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'No Trades Team', 'NTT', 'No Trade City', 'AFC', 'AFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .get(&format!(
            "{}/api/v1/teams/{}/pending-trades",
            base_url, team_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get pending trades");

    assert_eq!(response.status(), 200);

    let trades: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(trades.is_empty());
}

#[tokio::test]
async fn test_get_team_scouting_reports_empty() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let team_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Scout Empty Team', 'SET', 'Scout City', 'NFC', 'NFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .get(&format!(
            "{}/api/v1/teams/{}/scouting-reports",
            base_url, team_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get scouting reports");

    assert_eq!(response.status(), 200);

    let reports: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(reports.is_empty());
}

#[tokio::test]
async fn test_get_player_combine_results_empty() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let player_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Combine', 'Empty', 'QB', 2026, true)",
        player_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .get(&format!(
            "{}/api/v1/players/{}/combine-results",
            base_url, player_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get combine results");

    assert_eq!(response.status(), 200);

    let results: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(results.is_empty());
}
