//! Next/available pick endpoint acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

/// Helper: create teams, a draft, and initialize picks. Returns (draft_id, pick_ids).
async fn setup_draft_with_picks(
    base_url: &str,
    client: &reqwest::Client,
    num_teams: usize,
    rounds: i32,
) -> (String, Vec<String>) {
    // Create teams
    for i in 0..num_teams {
        let resp = client
            .post(&format!("{}/api/v1/teams", base_url))
            .json(&json!({
                "name": format!("PQ Team {}", i),
                "abbreviation": format!("P{:02}", i),
                "city": format!("City {}", i),
                "conference": if i % 2 == 0 { "AFC" } else { "NFC" },
                "division": if i % 2 == 0 { "AFC East" } else { "NFC East" }
            }))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .expect("Failed to create team");
        assert_eq!(resp.status(), 201);
    }

    // Create draft
    let draft_response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": rounds,
            "picks_per_round": num_teams as i32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create draft");
    assert_eq!(draft_response.status(), 201);

    let draft: serde_json::Value = draft_response.json().await.unwrap();
    let draft_id = draft["id"].as_str().unwrap().to_string();

    // Initialize picks
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

    let picks: Vec<serde_json::Value> = init_response.json().await.unwrap();
    let pick_ids: Vec<String> = picks
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_string())
        .collect();

    (draft_id, pick_ids)
}

#[tokio::test]
async fn test_get_next_pick() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let (draft_id, _pick_ids) = setup_draft_with_picks(&base_url, &client, 2, 1).await;

    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/picks/next",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get next pick");

    assert_eq!(response.status(), 200);

    let next_pick: serde_json::Value = response.json().await.unwrap();
    assert_eq!(next_pick["overall_pick"], 1);
}

#[tokio::test]
async fn test_get_next_pick_advances_after_made() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let (draft_id, pick_ids) = setup_draft_with_picks(&base_url, &client, 2, 1).await;

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Next",
            "last_name": "Pick",
            "position": "QB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(player_response.status(), 201);
    let player: serde_json::Value = player_response.json().await.unwrap();
    let player_id = player["id"].as_str().unwrap();

    // Make pick #1
    let make_response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_ids[0]))
        .json(&json!({ "player_id": player_id }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to make pick");
    assert_eq!(make_response.status(), 200);

    // Next pick should now be #2
    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/picks/next",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get next pick");

    assert_eq!(response.status(), 200);

    let next_pick: serde_json::Value = response.json().await.unwrap();
    assert_eq!(next_pick["overall_pick"], 2);
}

#[tokio::test]
async fn test_get_next_pick_returns_null_when_all_made() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let (draft_id, pick_ids) = setup_draft_with_picks(&base_url, &client, 1, 1).await;

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Only",
            "last_name": "Player",
            "position": "RB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(player_response.status(), 201);
    let player: serde_json::Value = player_response.json().await.unwrap();
    let player_id = player["id"].as_str().unwrap();

    // Make the only pick
    let make_response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_ids[0]))
        .json(&json!({ "player_id": player_id }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to make pick");
    assert_eq!(make_response.status(), 200);

    // Next pick should be null
    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/picks/next",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get next pick");

    assert_eq!(response.status(), 200);

    let next_pick: serde_json::Value = response.json().await.unwrap();
    assert!(next_pick.is_null());
}

#[tokio::test]
async fn test_get_available_picks() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let (draft_id, _pick_ids) = setup_draft_with_picks(&base_url, &client, 2, 1).await;

    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/picks/available",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get available picks");

    assert_eq!(response.status(), 200);

    let available: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(available.len(), 2);
}

#[tokio::test]
async fn test_get_available_picks_decrements() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let (draft_id, pick_ids) = setup_draft_with_picks(&base_url, &client, 2, 1).await;

    // Create a player and make first pick
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Dec",
            "last_name": "Player",
            "position": "WR",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");
    assert_eq!(player_response.status(), 201);
    let player: serde_json::Value = player_response.json().await.unwrap();
    let player_id = player["id"].as_str().unwrap();

    let make_response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_ids[0]))
        .json(&json!({ "player_id": player_id }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to make pick");
    assert_eq!(make_response.status(), 200);

    // Available should now be 1
    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/picks/available",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get available picks");

    assert_eq!(response.status(), 200);

    let available: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(available.len(), 1);
}

#[tokio::test]
async fn test_get_all_draft_picks_ordered() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let (draft_id, _pick_ids) = setup_draft_with_picks(&base_url, &client, 2, 2).await;

    let response = client
        .get(&format!("{}/api/v1/drafts/{}/picks", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get all picks");

    assert_eq!(response.status(), 200);

    let picks: Vec<serde_json::Value> = response.json().await.unwrap();
    // 2 teams * 2 rounds = 4 picks
    assert_eq!(picks.len(), 4);

    // Verify ordered by overall_pick
    for (i, pick) in picks.iter().enumerate() {
        assert_eq!(pick["overall_pick"], (i + 1) as i64);
    }
}
