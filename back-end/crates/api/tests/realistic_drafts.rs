//! Realistic draft flow acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn test_create_realistic_draft() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft without picks_per_round → realistic draft
    let response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 7
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create realistic draft");

    assert_eq!(response.status(), 201);

    let draft: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let draft_id = draft["id"].as_str().expect("Missing draft id");

    assert_eq!(draft["year"], 2026);
    assert_eq!(draft["is_realistic"], true);
    assert!(draft["picks_per_round"].is_null());
    assert!(draft["total_picks"].is_null());
    assert_eq!(draft["status"], "NotStarted");

    // Verify in database
    let db_draft = sqlx::query!(
        "SELECT picks_per_round FROM drafts WHERE id = $1",
        Uuid::parse_str(draft_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Draft not found in database");

    assert!(db_draft.picks_per_round.is_none());
}

#[tokio::test]
async fn test_initialize_picks_fails_for_realistic_draft() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create two teams
    for (name, abbr, city) in [("Team A", "TMA", "City A"), ("Team B", "TMB", "City B")] {
        let resp = client
            .post(&format!("{}/api/v1/teams", base_url))
            .json(&json!({
                "name": name,
                "abbreviation": abbr,
                "city": city,
                "conference": "AFC",
                "division": "AFC East"
            }))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .expect("Failed to create team");
        assert_eq!(resp.status(), 201);
    }

    // Create realistic draft
    let draft_response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 7
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create draft");
    assert_eq!(draft_response.status(), 201);

    let draft: serde_json::Value = draft_response.json().await.unwrap();
    let draft_id = draft["id"].as_str().unwrap();

    // Try to initialize picks → should fail with 400
    let init_response = client
        .post(&format!(
            "{}/api/v1/drafts/{}/initialize",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send initialize request");

    assert_eq!(init_response.status(), 400);

    let error_body: serde_json::Value = init_response.json().await.unwrap();
    let error_msg = error_body["error"].as_str().unwrap_or("");
    assert!(
        error_msg.to_lowercase().contains("realistic"),
        "Error message should mention 'realistic', got: {}",
        error_msg
    );

    // Verify no picks in database
    let db_pick_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_picks WHERE draft_id = $1",
        Uuid::parse_str(draft_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_pick_count.count.unwrap(), 0);
}

#[tokio::test]
async fn test_realistic_draft_picks_with_trade_metadata() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create two teams
    let team1_id = Uuid::new_v4();
    let team2_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Team Alpha', 'TAL', 'Alpha City', 'AFC', 'AFC East')",
        team1_id
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Team Beta', 'TBE', 'Beta City', 'NFC', 'NFC East')",
        team2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create realistic draft via SQL
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, NULL)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert a traded pick: team1 holds it, originally team2's pick
    let pick_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id, original_team_id, is_compensatory, notes)
           VALUES ($1, $2, 1, 1, 1, $3, $4, true, 'Traded from Team Beta')"#,
        pick_id,
        draft_id,
        team1_id,
        team2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // GET picks via API
    let response = client
        .get(&format!("{}/api/v1/drafts/{}/picks", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get picks");

    assert_eq!(response.status(), 200);

    let picks: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(picks.len(), 1);

    let pick = &picks[0];
    assert_eq!(pick["is_traded"], true);
    assert_eq!(pick["is_compensatory"], true);
    assert_eq!(pick["notes"], "Traded from Team Beta");
    assert_eq!(pick["original_team_id"], team2_id.to_string());
    assert_eq!(pick["team_id"], team1_id.to_string());
}

#[tokio::test]
async fn test_realistic_draft_pick_not_traded() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create team
    let team_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Team Own', 'TOW', 'Own City', 'AFC', 'AFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create realistic draft
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'NotStarted', 7, NULL)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert pick where original_team_id == team_id (not traded)
    let pick_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id, original_team_id, is_compensatory, notes)
           VALUES ($1, $2, 1, 1, 1, $3, $3, false, NULL)"#,
        pick_id,
        draft_id,
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // GET picks via API
    let response = client
        .get(&format!("{}/api/v1/drafts/{}/picks", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get picks");

    assert_eq!(response.status(), 200);

    let picks: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(picks.len(), 1);
    assert_eq!(picks[0]["is_traded"], false);
    assert_eq!(picks[0]["is_compensatory"], false);
}
