//! Team Scouting Workflow Integration Tests
//!
//! Tests cross-feature workflows involving teams, team needs, and scouting reports.

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_team_needs_to_scouting_workflow() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Step 1: Create a team
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Arizona Cardinals",
            "abbreviation": "ARI",
            "city": "Arizona",
            "conference": "NFC",
            "division": "NFC West"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    assert_eq!(team_response.status(), 201);
    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Step 2: Add team needs
    let need1_response = client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "QB",
            "priority": 1,
            "notes": "Need franchise quarterback"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need 1");

    assert_eq!(need1_response.status(), 201);

    let need2_response = client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "WR",
            "priority": 2,
            "notes": "Need playmaker on offense"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team need 2");

    assert_eq!(need2_response.status(), 201);

    // Verify team needs in database
    let db_needs_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM team_needs WHERE team_id = $1",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count team needs");

    assert_eq!(db_needs_count.count, Some(2));

    // Step 3: Scout players matching those positions
    let qb1_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Top",
            "last_name": "Quarterback",
            "position": "QB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create QB1");

    let qb1: serde_json::Value = qb1_response.json().await.expect("Failed to parse JSON");
    let qb1_id = qb1["id"].as_str().expect("Missing QB1 id");

    let qb2_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Second",
            "last_name": "Quarterback",
            "position": "QB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create QB2");

    let qb2: serde_json::Value = qb2_response.json().await.expect("Failed to parse JSON");
    let qb2_id = qb2["id"].as_str().expect("Missing QB2 id");

    let wr1_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Elite",
            "last_name": "Receiver",
            "position": "WR",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create WR1");

    let wr1: serde_json::Value = wr1_response.json().await.expect("Failed to parse JSON");
    let wr1_id = wr1["id"].as_str().expect("Missing WR1 id");

    // Create a player for a position that's NOT a team need
    let rb1_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Good",
            "last_name": "Running Back",
            "position": "RB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create RB1");

    let rb1: serde_json::Value = rb1_response.json().await.expect("Failed to parse JSON");
    let rb1_id = rb1["id"].as_str().expect("Missing RB1 id");

    // Step 4: Add scouting reports for all players
    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": qb1_id,
            "team_id": team_id,
            "grade": 9.2,
            "notes": "Perfect fit for our offense",
            "fit_grade": "A"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create QB1 scouting report");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": qb2_id,
            "team_id": team_id,
            "grade": 8.5,
            "notes": "Good backup option if QB1 is gone",
            "fit_grade": "A"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create QB2 scouting report");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": wr1_id,
            "team_id": team_id,
            "grade": 8.8,
            "notes": "Dynamic playmaker",
            "fit_grade": "A"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create WR1 scouting report");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": rb1_id,
            "team_id": team_id,
            "grade": 8.0,
            "notes": "Best player available but not a need",
            "fit_grade": "B"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create RB1 scouting report");

    // Step 5: Query team needs alongside scouting reports
    let needs_response = client
        .get(&format!("{}/api/v1/teams/{}/needs", base_url, team_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team needs");

    assert_eq!(needs_response.status(), 200);
    let needs: Vec<serde_json::Value> = needs_response.json().await.expect("Failed to parse JSON");
    assert_eq!(needs.len(), 2);
    assert_eq!(needs[0]["position"], "QB"); // Priority 1
    assert_eq!(needs[1]["position"], "WR"); // Priority 2

    let reports_response = client
        .get(&format!("{}/api/v1/teams/{}/scouting-reports", base_url, team_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get scouting reports");

    assert_eq!(reports_response.status(), 200);
    let reports: Vec<serde_json::Value> = reports_response.json().await.expect("Failed to parse JSON");
    assert_eq!(reports.len(), 4);

    // Verify database consistency
    let db_reports_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM scouting_reports WHERE team_id = $1",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count scouting reports");

    assert_eq!(db_reports_count.count, Some(4));

    // Step 6: Query scouting reports by position to match needs
    // In a real app, this would be done via JOIN or application logic
    // For this test, we verify the data structure supports this workflow
    let qb_reports = sqlx::query!(
        r#"
        SELECT sr.id, sr.grade, p.position
        FROM scouting_reports sr
        JOIN players p ON sr.player_id = p.id
        WHERE sr.team_id = $1 AND p.position = 'QB'
        ORDER BY sr.grade DESC
        "#,
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query QB scouting reports");

    assert_eq!(qb_reports.len(), 2);
    assert_eq!(qb_reports[0].grade, 9.2); // QB1 highest grade
    assert_eq!(qb_reports[1].grade, 8.5); // QB2 second

    let wr_reports = sqlx::query!(
        r#"
        SELECT sr.id, sr.grade, p.position
        FROM scouting_reports sr
        JOIN players p ON sr.player_id = p.id
        WHERE sr.team_id = $1 AND p.position = 'WR'
        ORDER BY sr.grade DESC
        "#,
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query WR scouting reports");

    assert_eq!(wr_reports.len(), 1);
    assert_eq!(wr_reports[0].grade, 8.8);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_multiple_teams_scouting_same_player() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a highly-rated player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Travis",
            "last_name": "Hunter",
            "position": "CB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create three teams with different scouting opinions
    let team1_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Cleveland Browns",
            "abbreviation": "CLE",
            "city": "Cleveland",
            "conference": "AFC",
            "division": "AFC North"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team 1");

    let team1: serde_json::Value = team1_response.json().await.expect("Failed to parse JSON");
    let team1_id = team1["id"].as_str().expect("Missing team1 id");

    let team2_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Miami Dolphins",
            "abbreviation": "MIA",
            "city": "Miami",
            "conference": "AFC",
            "division": "AFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team 2");

    let team2: serde_json::Value = team2_response.json().await.expect("Failed to parse JSON");
    let team2_id = team2["id"].as_str().expect("Missing team2 id");

    let team3_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Detroit Lions",
            "abbreviation": "DET",
            "city": "Detroit",
            "conference": "NFC",
            "division": "NFC North"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team 3");

    let team3: serde_json::Value = team3_response.json().await.expect("Failed to parse JSON");
    let team3_id = team3["id"].as_str().expect("Missing team3 id");

    // Each team scouts with different grades and fit assessments
    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team1_id,
            "grade": 9.5,
            "notes": "Generational talent, perfect fit",
            "fit_grade": "A",
            "injury_concern": false,
            "character_concern": false
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team1 scouting report");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team2_id,
            "grade": 8.8,
            "notes": "Excellent player but we have other needs",
            "fit_grade": "B",
            "injury_concern": false,
            "character_concern": false
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team2 scouting report");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": player_id,
            "team_id": team3_id,
            "grade": 9.2,
            "notes": "Top 5 pick, worth trading up",
            "fit_grade": "A",
            "injury_concern": false,
            "character_concern": false
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team3 scouting report");

    // Query player's scouting reports across all teams
    let reports_response = client
        .get(&format!("{}/api/v1/players/{}/scouting-reports", base_url, player_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get player scouting reports");

    assert_eq!(reports_response.status(), 200);
    let reports: Vec<serde_json::Value> = reports_response.json().await.expect("Failed to parse JSON");
    assert_eq!(reports.len(), 3);

    // Verify different grades from different teams
    let grades: Vec<f64> = reports.iter()
        .map(|r| r["grade"].as_f64().unwrap())
        .collect();

    assert!(grades.contains(&9.5));
    assert!(grades.contains(&8.8));
    assert!(grades.contains(&9.2));

    // Verify in database
    let db_reports = sqlx::query!(
        r#"
        SELECT team_id, grade, fit_grade
        FROM scouting_reports
        WHERE player_id = $1
        ORDER BY grade DESC
        "#,
        uuid::Uuid::parse_str(player_id).unwrap()
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query scouting reports");

    assert_eq!(db_reports.len(), 3);
    assert_eq!(db_reports[0].grade, 9.5); // Cleveland highest
    assert_eq!(db_reports[1].grade, 9.2); // Detroit second
    assert_eq!(db_reports[2].grade, 8.8); // Miami third

    // Each team can query their own report
    let team1_reports = client
        .get(&format!("{}/api/v1/teams/{}/scouting-reports", base_url, team1_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team1 reports");

    assert_eq!(team1_reports.status(), 200);
    let team1_data: Vec<serde_json::Value> = team1_reports.json().await.expect("Failed to parse JSON");
    assert_eq!(team1_data.len(), 1);
    assert_eq!(team1_data[0]["grade"], 9.5);
    assert_eq!(team1_data[0]["fit_grade"], "A");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_team_scouting_by_position_matching() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Indianapolis Colts",
            "abbreviation": "IND",
            "city": "Indianapolis",
            "conference": "AFC",
            "division": "AFC South"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Create team needs for specific positions
    client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "DE",
            "priority": 1
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create DE need");

    client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "OT",
            "priority": 2
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create OT need");

    // Create players at various positions
    let edge1_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Top",
            "last_name": "Edge",
            "position": "DE",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create DE1");

    let edge1: serde_json::Value = edge1_response.json().await.expect("Failed to parse JSON");
    let edge1_id = edge1["id"].as_str().expect("Missing DE1 id");

    let edge2_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Second",
            "last_name": "Edge",
            "position": "DE",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create DE2");

    let edge2: serde_json::Value = edge2_response.json().await.expect("Failed to parse JSON");
    let edge2_id = edge2["id"].as_str().expect("Missing DE2 id");

    let ot_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Elite",
            "last_name": "Tackle",
            "position": "OT",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create OT");

    let ot: serde_json::Value = ot_response.json().await.expect("Failed to parse JSON");
    let ot_id = ot["id"].as_str().expect("Missing OT id");

    let cb_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Good",
            "last_name": "Corner",
            "position": "CB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create CB");

    let cb: serde_json::Value = cb_response.json().await.expect("Failed to parse JSON");
    let cb_id = cb["id"].as_str().expect("Missing CB id");

    // Scout all players
    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": edge1_id,
            "team_id": team_id,
            "grade": 9.0,
            "fit_grade": "A"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to scout DE1");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": edge2_id,
            "team_id": team_id,
            "grade": 8.3,
            "fit_grade": "A"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to scout DE2");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": ot_id,
            "team_id": team_id,
            "grade": 8.7,
            "fit_grade": "A"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to scout OT");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": cb_id,
            "team_id": team_id,
            "grade": 8.9,
            "fit_grade": "B"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to scout CB");

    // Query to match scouting reports with team needs by position
    let need_positions = sqlx::query!(
        "SELECT position FROM team_needs WHERE team_id = $1 ORDER BY priority",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query team needs");

    assert_eq!(need_positions.len(), 2);
    assert_eq!(need_positions[0].position, "DE");
    assert_eq!(need_positions[1].position, "OT");

    // Query scouting reports matching first need (DE)
    let edge_scouts = sqlx::query!(
        r#"
        SELECT sr.grade, sr.fit_grade, p.first_name, p.last_name
        FROM scouting_reports sr
        JOIN players p ON sr.player_id = p.id
        WHERE sr.team_id = $1 AND p.position = 'DE'
        ORDER BY sr.grade DESC
        "#,
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query DE scouts");

    assert_eq!(edge_scouts.len(), 2);
    assert_eq!(edge_scouts[0].grade, 9.0);
    assert_eq!(edge_scouts[0].first_name, "Top");
    assert_eq!(edge_scouts[1].grade, 8.3);
    assert_eq!(edge_scouts[1].first_name, "Second");

    // Query scouting reports matching second need (OT)
    let ot_scouts = sqlx::query!(
        r#"
        SELECT sr.grade, sr.fit_grade, p.first_name, p.last_name
        FROM scouting_reports sr
        JOIN players p ON sr.player_id = p.id
        WHERE sr.team_id = $1 AND p.position = 'OT'
        ORDER BY sr.grade DESC
        "#,
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query OT scouts");

    assert_eq!(ot_scouts.len(), 1);
    assert_eq!(ot_scouts[0].grade, 8.7);
    assert_eq!(ot_scouts[0].first_name, "Elite");

    // Verify CB was scouted but doesn't match needs
    let cb_scouts = sqlx::query!(
        r#"
        SELECT sr.grade
        FROM scouting_reports sr
        JOIN players p ON sr.player_id = p.id
        WHERE sr.team_id = $1 AND p.position = 'CB'
        "#,
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query CB scouts");

    assert_eq!(cb_scouts.len(), 1);
    assert_eq!(cb_scouts[0].grade, 8.9);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_team_draft_board_generation() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create team
    let team_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Atlanta Falcons",
            "abbreviation": "ATL",
            "city": "Atlanta",
            "conference": "NFC",
            "division": "NFC South"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team");

    let team: serde_json::Value = team_response.json().await.expect("Failed to parse JSON");
    let team_id = team["id"].as_str().expect("Missing team id");

    // Add team needs with priorities
    client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "QB",
            "priority": 1
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create QB need");

    client
        .post(&format!("{}/api/v1/team-needs", base_url))
        .json(&json!({
            "team_id": team_id,
            "position": "DE",
            "priority": 2
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create DE need");

    // Create players and scout them
    let qb_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Elite",
            "last_name": "QB",
            "position": "QB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create QB");

    let qb: serde_json::Value = qb_response.json().await.expect("Failed to parse JSON");
    let qb_id = qb["id"].as_str().expect("Missing QB id");

    let edge_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Elite",
            "last_name": "Edge",
            "position": "DE",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create DE");

    let edge: serde_json::Value = edge_response.json().await.expect("Failed to parse JSON");
    let edge_id = edge["id"].as_str().expect("Missing DE id");

    let wr_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Best",
            "last_name": "Available",
            "position": "WR",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create WR");

    let wr: serde_json::Value = wr_response.json().await.expect("Failed to parse JSON");
    let wr_id = wr["id"].as_str().expect("Missing WR id");

    // Scout all players
    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": qb_id,
            "team_id": team_id,
            "grade": 9.0,
            "fit_grade": "A"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to scout QB");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": edge_id,
            "team_id": team_id,
            "grade": 8.5,
            "fit_grade": "A"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to scout DE");

    client
        .post(&format!("{}/api/v1/scouting-reports", base_url))
        .json(&json!({
            "player_id": wr_id,
            "team_id": team_id,
            "grade": 9.2,
            "fit_grade": "B"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to scout WR");

    // Generate draft board: combine needs priority with scouting grades
    // This query simulates a draft board algorithm
    let draft_board = sqlx::query!(
        r#"
        SELECT
            p.id,
            p.first_name,
            p.last_name,
            p.position,
            sr.grade,
            sr.fit_grade,
            tn.priority as "priority?",
            CASE
                WHEN tn.priority IS NOT NULL THEN sr.grade + (4.0 - tn.priority)
                ELSE sr.grade - 1.0
            END as board_score
        FROM scouting_reports sr
        JOIN players p ON sr.player_id = p.id
        LEFT JOIN team_needs tn ON sr.team_id = tn.team_id AND p.position = tn.position
        WHERE sr.team_id = $1
        ORDER BY board_score DESC, sr.grade DESC
        "#,
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to generate draft board");

    assert_eq!(draft_board.len(), 3);

    // Verify board order:
    // 1. QB (grade 9.0 + priority boost 3.0 = 12.0)
    // 2. DE (grade 8.5 + priority boost 2.0 = 10.5)
    // 3. WR (grade 9.2 - 1.0 for no need = 8.2)
    assert_eq!(draft_board[0].position, "QB");
    assert_eq!(draft_board[0].grade, 9.0);
    assert_eq!(draft_board[0].priority, Some(1)); // Priority 1 team need

    assert_eq!(draft_board[1].position, "DE");
    assert_eq!(draft_board[1].grade, 8.5);
    assert_eq!(draft_board[1].priority, Some(2)); // Priority 2 team need

    assert_eq!(draft_board[2].position, "WR");
    assert_eq!(draft_board[2].grade, 9.2);
    assert_eq!(draft_board[2].priority, None); // WR is not a team need

    // Verify API endpoints support this workflow
    let needs_response = client
        .get(&format!("{}/api/v1/teams/{}/needs", base_url, team_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get needs");

    assert_eq!(needs_response.status(), 200);

    let reports_response = client
        .get(&format!("{}/api/v1/teams/{}/scouting-reports", base_url, team_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get reports");

    assert_eq!(reports_response.status(), 200);

    common::cleanup_database(&pool).await;
}
