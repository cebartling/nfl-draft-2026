//! Draft flow acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_draft_flow() {
    let base_url = common::spawn_app().await;
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

    // Create draft
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

    let draft: serde_json::Value = draft_response.json().await.expect("Failed to parse JSON");
    let draft_id = draft["id"].as_str().expect("Missing draft id");
    assert_eq!(draft["year"], 2026);
    assert_eq!(draft["status"], "NotStarted");
    assert_eq!(draft["total_picks"], 2);

    // Initialize draft picks
    let init_response = client
        .post(&format!("{}/api/v1/drafts/{}/initialize", base_url, draft_id))
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

    // Get next pick
    let next_pick_response = client
        .get(&format!("{}/api/v1/drafts/{}/picks/next", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get next pick");
    assert_eq!(next_pick_response.status(), 200);

    let next_pick: serde_json::Value = next_pick_response.json().await.expect("Failed to parse JSON");
    let pick_id = next_pick["id"].as_str().expect("Missing pick id");

    // Start draft
    let start_response = client
        .post(&format!("{}/api/v1/drafts/{}/start", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to start draft");
    assert_eq!(start_response.status(), 200);

    let started_draft: serde_json::Value = start_response.json().await.expect("Failed to parse JSON");
    assert_eq!(started_draft["status"], "InProgress");

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

    let made_pick: serde_json::Value = make_pick_response.json().await.expect("Failed to parse JSON");
    assert_eq!(made_pick["player_id"], player_id);
    assert!(made_pick["picked_at"].is_string());

    // Pause draft
    let pause_response = client
        .post(&format!("{}/api/v1/drafts/{}/pause", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to pause draft");
    assert_eq!(pause_response.status(), 200);

    let paused_draft: serde_json::Value = pause_response.json().await.expect("Failed to parse JSON");
    assert_eq!(paused_draft["status"], "Paused");

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

    let completed_draft: serde_json::Value = complete_response.json().await.expect("Failed to parse JSON");
    assert_eq!(completed_draft["status"], "Completed");
}
