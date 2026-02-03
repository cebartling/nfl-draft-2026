//! Team Needs CRUD acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_create_and_get_team_need() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a team
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Philadelphia Eagles",
            "abbreviation": "PHI",
            "city": "Philadelphia",
            "conference": "NFC",
            "division": "NFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Create team need
    let create_response = client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "QB",
            "priority": 10
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need");

    assert_eq!(create_response.status(), 201);

    let created_need: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let need_id = created_need["id"].as_str().expect("Missing need id");

    // Validate need was persisted in database
    let db_need = sqlx::query!(
        "SELECT id, team_id, position, priority FROM team_needs WHERE id = $1",
        uuid::Uuid::parse_str(need_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Team need not found in database");

    assert_eq!(db_need.team_id, uuid::Uuid::parse_str(team_id).unwrap());
    assert_eq!(db_need.position, "QB");
    assert_eq!(db_need.priority, 10);

    // Get team need via API
    let get_response = client
        .get(&format!("{}/api/v1/team-needs/{}", base_url, need_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team need");

    assert_eq!(get_response.status(), 200);

    let need: serde_json::Value = get_response.json().await.expect("Failed to parse JSON");
    assert_eq!(need["team_id"], team_id);
    assert_eq!(need["position"], "QB");
    assert_eq!(need["priority"], 10);

    // Verify API response matches database
    assert_eq!(need["position"].as_str().unwrap(), db_need.position);
    assert_eq!(need["priority"].as_i64().unwrap(), db_need.priority as i64);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_list_team_needs() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a team
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Washington Commanders",
            "abbreviation": "WAS",
            "city": "Washington",
            "conference": "NFC",
            "division": "NFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Create multiple team needs with different priorities
    client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "QB",
            "priority": 10
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need");

    client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "WR",
            "priority": 8
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need");

    client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "DE",
            "priority": 5
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need");

    // Get all team needs
    let list_response = client
        .get(&format!("{}/api/v1/teams/{}/needs", base_url, team_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team needs");

    assert_eq!(list_response.status(), 200);

    let needs_list: Vec<serde_json::Value> = list_response.json().await.expect("Failed to parse JSON");
    assert_eq!(needs_list.len(), 3);

    // Verify database count matches
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM team_needs WHERE team_id = $1",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count team needs");

    assert_eq!(db_count.count, Some(3));

    // Verify ordering (lowest priority number = highest priority = first)
    assert_eq!(needs_list[0]["priority"], 5);
    assert_eq!(needs_list[0]["position"], "DE");
    assert_eq!(needs_list[1]["priority"], 8);
    assert_eq!(needs_list[1]["position"], "WR");
    assert_eq!(needs_list[2]["priority"], 10);
    assert_eq!(needs_list[2]["position"], "QB");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_update_team_need() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team and team need
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Update Team",
            "abbreviation": "UPD",
            "city": "Update City",
            "conference": "AFC",
            "division": "AFC South"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    let create_response = client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "RB",
            "priority": 8
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need");

    let created: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let need_id = created["id"].as_str().expect("Missing need id");

    // Update team need priority
    let update_response = client
        .put(&format!("{}/api/v1/team-needs/{}", base_url, need_id))
        .json(&json!({
            "priority": 3
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to update team need");

    assert_eq!(update_response.status(), 200);

    let updated: serde_json::Value = update_response.json().await.expect("Failed to parse JSON");
    assert_eq!(updated["priority"], 3);
    assert_eq!(updated["position"], "RB"); // Position should not change

    // Verify in database
    let db_need = sqlx::query!(
        "SELECT priority, position FROM team_needs WHERE id = $1",
        uuid::Uuid::parse_str(need_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Team need not found in database");

    assert_eq!(db_need.priority, 3);
    assert_eq!(db_need.position, "RB");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_delete_team_need() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team and team need
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Delete Team",
            "abbreviation": "DEL",
            "city": "Delete City",
            "conference": "NFC",
            "division": "NFC North"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    let create_response = client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "CB",
            "priority": 7
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need");

    let created: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let need_id = created["id"].as_str().expect("Missing need id");

    // Delete team need
    let delete_response = client
        .delete(&format!("{}/api/v1/team-needs/{}", base_url, need_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to delete team need");

    assert_eq!(delete_response.status(), 204);

    // Verify deletion in database
    let db_result = sqlx::query!(
        "SELECT id FROM team_needs WHERE id = $1",
        uuid::Uuid::parse_str(need_id).unwrap()
    )
    .fetch_optional(&pool)
    .await
    .expect("Database query failed");

    assert!(db_result.is_none());

    // Verify 404 on subsequent GET
    let get_response = client
        .get(&format!("{}/api/v1/team-needs/{}", base_url, need_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team need");

    assert_eq!(get_response.status(), 404);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_duplicate_team_position_error() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Dup Team",
            "abbreviation": "DUP",
            "city": "Dup City",
            "conference": "AFC",
            "division": "AFC North"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Create first team need
    let first_response = client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "OT",
            "priority": 6
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need");

    assert_eq!(first_response.status(), 201);

    // Attempt to create duplicate (same team, same position)
    let duplicate_response = client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "OT",
            "priority": 3
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send duplicate request");

    assert_eq!(duplicate_response.status(), 409);

    // Verify only one entry in database
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM team_needs WHERE team_id = $1 AND position = 'OT'",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count team needs");

    assert_eq!(db_count.count, Some(1));

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_invalid_priority_validation() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Invalid Team",
            "abbreviation": "INV",
            "city": "Invalid City",
            "conference": "NFC",
            "division": "NFC West"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Attempt to create team need with priority 0 (invalid, must be 1-10)
    let invalid_response = client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "S",
            "priority": 0
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(invalid_response.status(), 400);

    // Attempt to create team need with priority 11 (invalid, must be 1-10)
    let invalid_response2 = client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "S",
            "priority": 11
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(invalid_response2.status(), 400);

    // Verify no entries in database
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM team_needs WHERE team_id = $1",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count team needs");

    assert_eq!(db_count.count, Some(0));

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_team_needs_cascade_delete() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Cascade Team",
            "abbreviation": "CAS",
            "city": "Cascade City",
            "conference": "AFC",
            "division": "AFC West"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Create team need
    client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "K",
            "priority": 1
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need");

    // Verify team need exists
    let count_before = sqlx::query!(
        "SELECT COUNT(*) as count FROM team_needs WHERE team_id = $1",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count team needs");

    assert_eq!(count_before.count, Some(1));

    // Delete team (should cascade delete team needs)
    sqlx::query!(
        "DELETE FROM teams WHERE id = $1",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .execute(&pool)
    .await
    .expect("Failed to delete team");

    // Verify team need was cascade deleted
    let count_after = sqlx::query!(
        "SELECT COUNT(*) as count FROM team_needs WHERE team_id = $1",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count team needs");

    assert_eq!(count_after.count, Some(0));

    common::cleanup_database(&pool).await;
}
