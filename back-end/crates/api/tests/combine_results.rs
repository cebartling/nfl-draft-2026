//! Combine Results CRUD acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_create_and_get_combine_results() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Cleanup
    common::cleanup_database(&pool).await;

    // Create a player first
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

    // Create combine results
    let create_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.52,
            "bench_press": 20,
            "vertical_jump": 35.5,
            "broad_jump": 120,
            "three_cone_drill": 7.1,
            "twenty_yard_shuttle": 4.3
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    assert_eq!(create_response.status(), 201);

    let created_results: serde_json::Value =
        create_response.json().await.expect("Failed to parse JSON");
    let results_id = created_results["id"].as_str().expect("Missing results id");

    // Validate results were persisted in database
    let db_results = sqlx::query!(
        r#"
        SELECT id, player_id, year, source, forty_yard_dash, bench_press, vertical_jump,
               broad_jump, three_cone_drill, twenty_yard_shuttle
        FROM combine_results
        WHERE id = $1
        "#,
        uuid::Uuid::parse_str(results_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Combine results not found in database");

    assert_eq!(
        db_results.player_id,
        uuid::Uuid::parse_str(player_id).unwrap()
    );
    assert_eq!(db_results.year, 2026);
    assert_eq!(db_results.source, "combine");
    assert_eq!(db_results.forty_yard_dash, Some(4.52));
    assert_eq!(db_results.bench_press, Some(20));
    assert_eq!(db_results.vertical_jump, Some(35.5));
    assert_eq!(db_results.broad_jump, Some(120));
    assert_eq!(db_results.three_cone_drill, Some(7.1));
    assert_eq!(db_results.twenty_yard_shuttle, Some(4.3));

    // Get combine results via API
    let get_response = client
        .get(&format!(
            "{}/api/v1/combine-results/{}",
            base_url, results_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get combine results");

    assert_eq!(get_response.status(), 200);

    let results: serde_json::Value = get_response.json().await.expect("Failed to parse JSON");
    assert_eq!(results["player_id"], player_id);
    assert_eq!(results["year"], 2026);
    assert_eq!(results["source"], "combine");
    assert_eq!(results["forty_yard_dash"], 4.52);
    assert_eq!(results["bench_press"], 20);

    // Verify API response matches database
    assert_eq!(results["year"].as_i64().unwrap(), db_results.year as i64);
    assert_eq!(
        results["forty_yard_dash"].as_f64().unwrap(),
        db_results.forty_yard_dash.unwrap()
    );

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_combine_results_with_source() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Pro",
            "last_name": "Day",
            "position": "WR",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create with explicit pro_day source
    let create_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "source": "pro_day",
            "forty_yard_dash": 4.48
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    assert_eq!(create_response.status(), 201);

    let created: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    assert_eq!(created["source"], "pro_day");

    // Verify source persisted in DB
    let db_results = sqlx::query!(
        "SELECT source FROM combine_results WHERE id = $1",
        uuid::Uuid::parse_str(created["id"].as_str().unwrap()).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Not found in database");

    assert_eq!(db_results.source, "pro_day");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_combine_and_pro_day_same_player_year() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Both",
            "last_name": "Sources",
            "position": "RB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create combine results
    let combine_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.52
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    assert_eq!(combine_response.status(), 201);

    // Create pro_day results for same player/year — should succeed
    let pro_day_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "source": "pro_day",
            "forty_yard_dash": 4.48
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create pro day results");

    assert_eq!(pro_day_response.status(), 201);

    // Verify both entries exist in database
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM combine_results WHERE player_id = $1 AND year = 2026",
        uuid::Uuid::parse_str(player_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count");

    assert_eq!(db_count.count, Some(2));

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_create_combine_results_with_new_measurables() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Full",
            "last_name": "Measurables",
            "position": "QB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create with new measurables
    let create_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.52,
            "arm_length": 33.5,
            "hand_size": 9.75,
            "wingspan": 78.5,
            "ten_yard_split": 1.55,
            "twenty_yard_split": 2.65
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    assert_eq!(create_response.status(), 201);

    let created: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");

    // Verify all new fields in API response
    assert_eq!(created["arm_length"], 33.5);
    assert_eq!(created["hand_size"], 9.75);
    assert_eq!(created["wingspan"], 78.5);
    assert_eq!(created["ten_yard_split"], 1.55);
    assert_eq!(created["twenty_yard_split"], 2.65);

    // Verify all persisted in database
    let db_results = sqlx::query!(
        r#"
        SELECT arm_length, hand_size, wingspan, ten_yard_split, twenty_yard_split
        FROM combine_results WHERE id = $1
        "#,
        uuid::Uuid::parse_str(created["id"].as_str().unwrap()).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Not found in database");

    assert_eq!(db_results.arm_length, Some(bigdecimal_to_f64(33.5)));
    assert_eq!(db_results.hand_size, Some(bigdecimal_to_f64(9.75)));
    assert_eq!(db_results.wingspan, Some(bigdecimal_to_f64(78.5)));
    assert_eq!(db_results.ten_yard_split, Some(bigdecimal_to_f64(1.55)));
    assert_eq!(
        db_results.twenty_yard_split,
        Some(bigdecimal_to_f64(2.65))
    );

    common::cleanup_database(&pool).await;
}

/// Helper: SQLx returns DECIMAL columns as bigdecimal::BigDecimal in query!.
/// We compare using f64 since that's what our domain model uses.
fn bigdecimal_to_f64(val: f64) -> f64 {
    val
}

#[tokio::test]
async fn test_source_defaults_to_combine() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Default",
            "last_name": "Source",
            "position": "TE",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create without source field
    let create_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.70
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    assert_eq!(create_response.status(), 201);

    let created: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    assert_eq!(created["source"], "combine");

    // Verify in database
    let db_results = sqlx::query!(
        "SELECT source FROM combine_results WHERE id = $1",
        uuid::Uuid::parse_str(created["id"].as_str().unwrap()).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Not found in database");

    assert_eq!(db_results.source, "combine");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_get_player_combine_results() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create a player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Jane",
            "last_name": "Smith",
            "position": "WR",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create two combine results for different years
    client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2025,
            "forty_yard_dash": 4.60
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.52
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    // Get all combine results for player
    let list_response = client
        .get(&format!(
            "{}/api/v1/players/{}/combine-results",
            base_url, player_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get player combine results");

    assert_eq!(list_response.status(), 200);

    let results_list: Vec<serde_json::Value> =
        list_response.json().await.expect("Failed to parse JSON");
    assert_eq!(results_list.len(), 2);

    // Verify database count matches
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM combine_results WHERE player_id = $1",
        uuid::Uuid::parse_str(player_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count combine results");

    assert_eq!(db_count.count, Some(2));

    // Verify ordering (most recent year first)
    assert_eq!(results_list[0]["year"], 2026);
    assert_eq!(results_list[1]["year"], 2025);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_update_combine_results() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create player and combine results
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Update",
            "last_name": "Test",
            "position": "RB",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    let create_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.60
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    let created: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let results_id = created["id"].as_str().expect("Missing results id");

    // Update combine results
    let update_response = client
        .put(&format!(
            "{}/api/v1/combine-results/{}",
            base_url, results_id
        ))
        .json(&json!({
            "forty_yard_dash": 4.52,
            "bench_press": 25,
            "vertical_jump": 38.0
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to update combine results");

    assert_eq!(update_response.status(), 200);

    let updated: serde_json::Value = update_response.json().await.expect("Failed to parse JSON");
    assert_eq!(updated["forty_yard_dash"], 4.52);
    assert_eq!(updated["bench_press"], 25);
    assert_eq!(updated["vertical_jump"], 38.0);

    // Verify in database
    let db_results = sqlx::query!(
        "SELECT forty_yard_dash, bench_press, vertical_jump FROM combine_results WHERE id = $1",
        uuid::Uuid::parse_str(results_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Combine results not found in database");

    assert_eq!(db_results.forty_yard_dash, Some(4.52));
    assert_eq!(db_results.bench_press, Some(25));
    assert_eq!(db_results.vertical_jump, Some(38.0));

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_delete_combine_results() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create player and combine results
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Delete",
            "last_name": "Test",
            "position": "TE",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    let create_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.70
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    let created: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let results_id = created["id"].as_str().expect("Missing results id");

    // Delete combine results
    let delete_response = client
        .delete(&format!(
            "{}/api/v1/combine-results/{}",
            base_url, results_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to delete combine results");

    assert_eq!(delete_response.status(), 204);

    // Verify deletion in database
    let db_result = sqlx::query!(
        "SELECT id FROM combine_results WHERE id = $1",
        uuid::Uuid::parse_str(results_id).unwrap()
    )
    .fetch_optional(&pool)
    .await
    .expect("Database query failed");

    assert!(db_result.is_none());

    // Verify 404 on subsequent GET
    let get_response = client
        .get(&format!(
            "{}/api/v1/combine-results/{}",
            base_url, results_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get combine results");

    assert_eq!(get_response.status(), 404);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_duplicate_player_year_error() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    common::cleanup_database(&pool).await;

    // Create player
    let player_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Duplicate",
            "last_name": "Test",
            "position": "DE",
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    let player: serde_json::Value = player_response.json().await.expect("Failed to parse JSON");
    let player_id = player["id"].as_str().expect("Missing player id");

    // Create first combine results
    let first_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.60
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create combine results");

    assert_eq!(first_response.status(), 201);

    // Attempt to create duplicate (same player, same year, same source)
    let duplicate_response = client
        .post(&format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.55
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send duplicate request");

    assert_eq!(duplicate_response.status(), 409);

    // Verify only one entry in database
    let db_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM combine_results WHERE player_id = $1 AND year = 2026",
        uuid::Uuid::parse_str(player_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count combine results");

    assert_eq!(db_count.count, Some(1));

    common::cleanup_database(&pool).await;
}
