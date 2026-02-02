//! List endpoints acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_list_endpoints() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create some test data
    client
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
        .expect("Failed to create team");

    client
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

    client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 1
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create draft");

    // List teams via API
    let teams_response = client
        .get(&format!("{}/api/v1/teams", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list teams");
    assert_eq!(teams_response.status(), 200);
    let teams: Vec<serde_json::Value> = teams_response.json().await.expect("Failed to parse JSON");
    assert!(!teams.is_empty());

    // Verify teams count in database
    let db_team_count = sqlx::query!("SELECT COUNT(*) as count FROM teams")
        .fetch_one(&pool)
        .await
        .expect("Failed to count teams");
    assert_eq!(teams.len() as i64, db_team_count.count.unwrap());

    // List players via API
    let players_response = client
        .get(&format!("{}/api/v1/players", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list players");
    assert_eq!(players_response.status(), 200);
    let players: Vec<serde_json::Value> = players_response.json().await.expect("Failed to parse JSON");
    assert!(!players.is_empty());

    // Verify players count in database
    let db_player_count = sqlx::query!("SELECT COUNT(*) as count FROM players")
        .fetch_one(&pool)
        .await
        .expect("Failed to count players");
    assert_eq!(players.len() as i64, db_player_count.count.unwrap());

    // List drafts via API
    let drafts_response = client
        .get(&format!("{}/api/v1/drafts", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list drafts");
    assert_eq!(drafts_response.status(), 200);
    let drafts: Vec<serde_json::Value> = drafts_response.json().await.expect("Failed to parse JSON");
    assert!(!drafts.is_empty());

    // Verify drafts count in database
    let db_draft_count = sqlx::query!("SELECT COUNT(*) as count FROM drafts")
        .fetch_one(&pool)
        .await
        .expect("Failed to count drafts");
    assert_eq!(drafts.len() as i64, db_draft_count.count.unwrap());
}
