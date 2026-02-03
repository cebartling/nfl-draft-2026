//! Scouting Reports CRUD acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_create_and_get_scouting_report() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a team
    let team_response = client
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

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Scout",
            "last_name": "Report",
            "position": "QB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create scouting report
    let create_response = client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team_id,
            "grade": 8.5,
            "notes": "Excellent arm strength and accuracy",
            "fit_grade": "A",
            "injury_concern": false,
            "character_concern": false
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create scouting report");

    assert_eq!(create_response.status(), 201);

    let created_report: serde_json::Value =
        create_response.json().await.expect("Failed to parse JSON");
    let report_id = created_report["id"].as_str().expect("Missing report id");

    // Validate report was persisted in database
    let db_report = sqlx::query!(
        r#"
        SELECT id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern
        FROM scouting_reports
        WHERE id = $1
        "#,
        uuid::Uuid::parse_str(report_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Scouting report not found in database");

    assert_eq!(
        db_report.player_id,
        uuid::Uuid::parse_str(player_id).unwrap()
    );
    assert_eq!(db_report.team_id, uuid::Uuid::parse_str(team_id).unwrap());
    assert_eq!(db_report.grade, 8.5);
    assert_eq!(
        db_report.notes,
        Some("Excellent arm strength and accuracy".to_string())
    );
    assert_eq!(db_report.fit_grade, Some("A".to_string()));
    assert_eq!(db_report.injury_concern, false);
    assert_eq!(db_report.character_concern, false);

    // Get scouting report via API
    let get_response = client
        .get(&format!(
            "{}/api/v1/scouting-reports/{}",
            base_url, report_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get scouting report");

    assert_eq!(get_response.status(), 200);

    let report: serde_json::Value = get_response.json().await.expect("Failed to parse JSON");
    assert_eq!(report["player_id"], player_id);
    assert_eq!(report["team_id"], team_id);
    assert_eq!(report["grade"], 8.5);
    assert_eq!(report["fit_grade"], "A");

    // Verify API response matches database
    assert_eq!(report["grade"].as_f64().unwrap(), db_report.grade);
    assert_eq!(
        report["injury_concern"].as_bool().unwrap(),
        db_report.injury_concern
    );

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_get_team_scouting_reports() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a team
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "New York Giants",
            "abbreviation": "NYG",
            "city": "New York",
            "conference": "NFC",
            "division": "NFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Create two players
    let player1_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Player",
            "last_name": "One",
            "position": "QB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player1: serde_json::Value = player1_response.json().await.expect("Failed to parse JSON");
    let player1_id = player1["id"].as_str().expect("Missing player id");

    let player2_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Player",
            "last_name": "Two",
            "position": "WR",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player2: serde_json::Value = player2_response.json().await.expect("Failed to parse JSON");
    let player2_id = player2["id"].as_str().expect("Missing player id");

    // Create scouting reports for both players
    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player1_id,
            "team_id": team_id,
            "grade": 9.0
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create scouting report");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player2_id,
            "team_id": team_id,
            "grade": 7.5
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create scouting report");

    // Get all scouting reports for team
    let list_response = client
        .get(&format!(
            "{}/api/v1/teams/{}/scouting-reports",
            base_url, team_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team scouting reports");

    assert_eq!(list_response.status(), 200);

    let reports_list: Vec<serde_json::Value> =
        list_response.json().await.expect("Failed to parse JSON");
    assert_eq!(reports_list.len(), 2);

    // Verify database count matches
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM scouting_reports WHERE team_id = $1",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count scouting reports");

    assert_eq!(db_count.count, Some(2));

    // Verify ordering (highest grade first)
    assert_eq!(reports_list[0]["grade"], 9.0);
    assert_eq!(reports_list[1]["grade"], 7.5);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_get_player_scouting_reports() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create two teams
    let team1_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Team One",
            "abbreviation": "TM1",
            "city": "City One",
            "conference": "AFC",
            "division": "AFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team1: serde_json::Value = team1_response.json().await.expect("Failed to parse JSON");
    let team1_id = team1["id"].as_str().expect("Missing team id");

    let team2_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Team Two",
            "abbreviation": "TM2",
            "city": "City Two",
            "conference": "NFC",
            "division": "NFC West"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team2: serde_json::Value = team2_response.json().await.expect("Failed to parse JSON");
    let team2_id = team2["id"].as_str().expect("Missing team id");

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Multi",
            "last_name": "Scout",
            "position": "RB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create scouting reports from both teams
    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team1_id,
            "grade": 8.0
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create scouting report");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team2_id,
            "grade": 7.0
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create scouting report");

    // Get all scouting reports for player
    let list_response = client
        .get(&format!(
            "{}/api/v1/players/{}/scouting-reports",
            base_url, player_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get player scouting reports");

    assert_eq!(list_response.status(), 200);

    let reports_list: Vec<serde_json::Value> =
        list_response.json().await.expect("Failed to parse JSON");
    assert_eq!(reports_list.len(), 2);

    // Verify database count matches
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM scouting_reports WHERE player_id = $1",
        uuid::Uuid::parse_str(player_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count scouting reports");

    assert_eq!(db_count.count, Some(2));

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_update_scouting_report() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team and player
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Update Team",
            "abbreviation": "UPD",
            "city": "Update City",
            "conference": "AFC",
            "division": "AFC North"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Update",
            "last_name": "Player",
            "position": "TE",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create scouting report
    let create_response = client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team_id,
            "grade": 7.0
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create scouting report");

    let created: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let report_id = created["id"].as_str().expect("Missing report id");

    // Update scouting report
    let update_response = client
        .put(&format!(
            "{}/api/v1/scouting-reports/{}",
            base_url, report_id
        ))
        .json(&json!({
            "grade": 8.5,
            "notes": "Improved after further review",
            "fit_grade": "A",
            "injury_concern": true
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to update scouting report");

    assert_eq!(update_response.status(), 200);

    let updated: serde_json::Value = update_response.json().await.expect("Failed to parse JSON");
    assert_eq!(updated["grade"], 8.5);
    assert_eq!(updated["fit_grade"], "A");
    assert_eq!(updated["injury_concern"], true);

    // Verify in database
    let db_report = sqlx::query!(
        "SELECT grade, notes, fit_grade, injury_concern FROM scouting_reports WHERE id = $1",
        uuid::Uuid::parse_str(report_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Scouting report not found in database");

    assert_eq!(db_report.grade, 8.5);
    assert_eq!(
        db_report.notes,
        Some("Improved after further review".to_string())
    );
    assert_eq!(db_report.fit_grade, Some("A".to_string()));
    assert_eq!(db_report.injury_concern, true);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_delete_scouting_report() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team and player
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Delete Team",
            "abbreviation": "DEL",
            "city": "Delete City",
            "conference": "NFC",
            "division": "NFC South"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Delete",
            "last_name": "Player",
            "position": "LB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create scouting report
    let create_response = client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team_id,
            "grade": 6.5
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create scouting report");

    let created: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let report_id = created["id"].as_str().expect("Missing report id");

    // Delete scouting report
    let delete_response = client
        .delete(&format!(
            "{}/api/v1/scouting-reports/{}",
            base_url, report_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to delete scouting report");

    assert_eq!(delete_response.status(), 204);

    // Verify deletion in database
    let db_result = sqlx::query!(
        "SELECT id FROM scouting_reports WHERE id = $1",
        uuid::Uuid::parse_str(report_id).unwrap()
    )
    .fetch_optional(&pool)
    .await
    .expect("Database query failed");

    assert!(db_result.is_none());

    // Verify 404 on subsequent GET
    let get_response = client
        .get(&format!(
            "{}/api/v1/scouting-reports/{}",
            base_url, report_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get scouting report");

    assert_eq!(get_response.status(), 404);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_duplicate_team_player_error() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team and player
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Dup Team",
            "abbreviation": "DUP",
            "city": "Dup City",
            "conference": "AFC",
            "division": "AFC West"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Dup",
            "last_name": "Player",
            "position": "CB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create first scouting report
    let first_response = client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team_id,
            "grade": 7.0
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create scouting report");

    assert_eq!(first_response.status(), 201);

    // Attempt to create duplicate (same team, same player)
    let duplicate_response = client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team_id,
            "grade": 8.0
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send duplicate request");

    assert_eq!(duplicate_response.status(), 409);

    // Verify only one entry in database
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM scouting_reports WHERE team_id = $1 AND player_id = $2",
        uuid::Uuid::parse_str(team_id).unwrap(),
        uuid::Uuid::parse_str(player_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count scouting reports");

    assert_eq!(db_count.count, Some(1));

    common::cleanup_database(&pool).await;
}
