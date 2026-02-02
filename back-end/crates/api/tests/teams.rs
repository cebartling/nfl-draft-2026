//! Team CRUD acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_create_and_get_team() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create team
    let create_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Dallas Cowboys",
            "abbreviation": "DAL",
            "city": "Dallas",
            "conference": "NFC",
            "division": "NFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    assert_eq!(create_response.status(), 201);

    let created_team: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let team_id = created_team["id"].as_str().expect("Missing team id");

    // Validate team was persisted in database
    let db_team = sqlx::query!(
        "SELECT id, name, abbreviation, city, conference, division FROM teams WHERE id = $1",
        uuid::Uuid::parse_str(team_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Team not found in database");

    assert_eq!(db_team.name, "Dallas Cowboys");
    assert_eq!(db_team.abbreviation, "DAL");
    assert_eq!(db_team.city, "Dallas");
    assert_eq!(db_team.conference, "NFC");
    assert_eq!(db_team.division, "NFC East");

    // Get team via API
    let get_response = client
        .get(&format!("{}/api/v1/teams/{}", base_url, team_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team");

    assert_eq!(get_response.status(), 200);

    let team: serde_json::Value = get_response.json().await.expect("Failed to parse JSON");
    assert_eq!(team["name"], "Dallas Cowboys");
    assert_eq!(team["abbreviation"], "DAL");

    // Verify API response matches database
    assert_eq!(team["name"].as_str().unwrap(), db_team.name);
    assert_eq!(team["abbreviation"].as_str().unwrap(), db_team.abbreviation);
}
