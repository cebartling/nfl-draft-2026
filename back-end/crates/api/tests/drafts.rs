//! Draft flow acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_draft_flow() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create two teams
    let team1_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Team A",
            "abbreviation": "TMA",
            "city": "City A",
            "conference": "AFC",
            "division": "AFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team 1");
    assert_eq!(team1_response.status(), 201);

    let team2_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Team B",
            "abbreviation": "TMB",
            "city": "City B",
            "conference": "NFC",
            "division": "NFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team 2");
    assert_eq!(team2_response.status(), 201);

    // Verify teams were persisted in database
    let db_team_count = sqlx::query!("SELECT COUNT(*) as count FROM teams")
        .fetch_one(&pool)
        .await
        .expect("Failed to count teams");
    assert_eq!(db_team_count.count.unwrap(), 2);

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe",
            "position": "QB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(player_response.status(), 201);
    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Verify player was persisted in database
    let db_player = sqlx::query!(
        "SELECT first_name, last_name, position FROM players WHERE id = $1",
        uuid::Uuid::parse_str(player_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Player not found in database");
    assert_eq!(db_player.first_name, "John");
    assert_eq!(db_player.last_name, "Doe");
    assert_eq!(db_player.position, "QB");

    // Create draft
    let draft_response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "name": "Test Draft",
            "year": 2026,
            "rounds": 1,
            "picks_per_round": 2
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create draft");
    assert_eq!(draft_response.status(), 201);

    let draft: serde_json::Value = draft_response.json().await.expect("Failed to parse JSON");
    let draft_id = draft["id"].as_str().expect("Missing draft id");
    assert_eq!(draft["year"], 2026);
    assert_eq!(draft["status"], "NotStarted");
    assert_eq!(draft["total_picks"], 2);

    // Verify draft was persisted in database with correct status
    let db_draft = sqlx::query!(
        "SELECT year, status, rounds, picks_per_round FROM drafts WHERE id = $1",
        uuid::Uuid::parse_str(draft_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Draft not found in database");
    assert_eq!(db_draft.year, 2026);
    assert_eq!(db_draft.status, "NotStarted");
    assert_eq!(db_draft.rounds, 1);
    assert_eq!(db_draft.picks_per_round, Some(2));

    // Initialize draft picks
    let init_response = client
        .post(&format!(
            "{}/api/v1/drafts/{}/initialize",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to initialize picks");
    assert_eq!(init_response.status(), 201);

    let picks: Vec<serde_json::Value> = init_response.json().await.expect("Failed to parse JSON");
    assert_eq!(picks.len(), 2);
    assert_eq!(picks[0]["round"], 1);
    assert_eq!(picks[0]["pick_number"], 1);
    assert_eq!(picks[0]["overall_pick"], 1);

    // Verify picks were persisted in database
    let db_pick_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_picks WHERE draft_id = $1",
        uuid::Uuid::parse_str(draft_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count picks");
    assert_eq!(db_pick_count.count.unwrap(), 2);

    // Get next pick
    let next_pick_response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/picks/next",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get next pick");
    assert_eq!(next_pick_response.status(), 200);

    let next_pick: serde_json::Value = next_pick_response
        .json()
        .await
        .expect("Failed to parse JSON");
    let pick_id = next_pick["id"].as_str().expect("Missing pick id");

    // Start draft
    let start_response = client
        .post(&format!("{}/api/v1/drafts/{}/start", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to start draft");
    assert_eq!(start_response.status(), 200);

    let started_draft: serde_json::Value =
        start_response.json().await.expect("Failed to parse JSON");
    assert_eq!(started_draft["status"], "InProgress");

    // Verify draft status was updated in database
    let db_status = sqlx::query!(
        "SELECT status FROM drafts WHERE id = $1",
        uuid::Uuid::parse_str(draft_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch draft status");
    assert_eq!(db_status.status, "InProgress");

    // Make pick
    let make_pick_response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_id))
        .json(&json!({
            "player_id": player_id
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to make pick");
    assert_eq!(make_pick_response.status(), 200);

    let made_pick: serde_json::Value = make_pick_response
        .json()
        .await
        .expect("Failed to parse JSON");
    assert_eq!(made_pick["player_id"], player_id);
    assert!(made_pick["picked_at"].is_string());

    // Verify pick was updated in database
    let db_pick = sqlx::query!(
        "SELECT player_id, picked_at FROM draft_picks WHERE id = $1",
        uuid::Uuid::parse_str(pick_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch pick");
    assert_eq!(
        db_pick.player_id,
        Some(uuid::Uuid::parse_str(player_id).expect("Invalid UUID"))
    );
    assert!(db_pick.picked_at.is_some());

    // Pause draft
    let pause_response = client
        .post(&format!("{}/api/v1/drafts/{}/pause", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to pause draft");
    assert_eq!(pause_response.status(), 200);

    let paused_draft: serde_json::Value =
        pause_response.json().await.expect("Failed to parse JSON");
    assert_eq!(paused_draft["status"], "Paused");

    // Verify pause status in database
    let db_paused = sqlx::query!(
        "SELECT status FROM drafts WHERE id = $1",
        uuid::Uuid::parse_str(draft_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch draft status");
    assert_eq!(db_paused.status, "Paused");

    // Resume draft (start again)
    let resume_response = client
        .post(&format!("{}/api/v1/drafts/{}/start", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to resume draft");
    assert_eq!(resume_response.status(), 200);

    // Complete draft
    let complete_response = client
        .post(&format!("{}/api/v1/drafts/{}/complete", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to complete draft");
    assert_eq!(complete_response.status(), 200);

    let completed_draft: serde_json::Value = complete_response
        .json()
        .await
        .expect("Failed to parse JSON");
    assert_eq!(completed_draft["status"], "Completed");

    // Verify completed status in database
    let db_completed = sqlx::query!(
        "SELECT status FROM drafts WHERE id = $1",
        uuid::Uuid::parse_str(draft_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch draft status");
    assert_eq!(db_completed.status, "Completed");
}
