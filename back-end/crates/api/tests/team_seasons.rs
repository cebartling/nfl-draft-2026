//! Team seasons and draft order acceptance tests

mod common;

use std::time::Duration;
use uuid::Uuid;

/// Helper: create a team via SQL and return its id
async fn create_team(pool: &sqlx::PgPool, name: &str, abbr: &str, city: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, $2, $3, $4, 'AFC', 'AFC East')",
        id,
        name,
        abbr,
        city,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Helper: insert a team season
async fn insert_team_season(
    pool: &sqlx::PgPool,
    team_id: Uuid,
    year: i32,
    wins: i32,
    losses: i32,
    ties: i32,
    draft_position: Option<i32>,
) {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO team_seasons (id, team_id, season_year, wins, losses, ties, draft_position)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        id,
        team_id,
        year,
        wins,
        losses,
        ties,
        draft_position,
    )
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn test_list_team_seasons_by_year() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let team1_id = create_team(&pool, "Seasons Team A", "STA", "Alpha").await;
    let team2_id = create_team(&pool, "Seasons Team B", "STB", "Beta").await;

    insert_team_season(&pool, team1_id, 2025, 10, 7, 0, Some(15)).await;
    insert_team_season(&pool, team2_id, 2025, 4, 13, 0, Some(3)).await;

    let response = client
        .get(&format!("{}/api/v1/team-seasons?year=2025", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list team seasons");

    assert_eq!(response.status(), 200);

    let seasons: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(seasons.len(), 2);

    // Verify fields are present
    for season in &seasons {
        assert!(season["wins"].is_number());
        assert!(season["losses"].is_number());
        assert!(season["ties"].is_number());
        assert!(season["draft_position"].is_number());
        assert!(season["team_id"].is_string());
    }
}

#[tokio::test]
async fn test_list_team_seasons_empty_year() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!("{}/api/v1/team-seasons?year=1999", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list team seasons");

    assert_eq!(response.status(), 200);

    let seasons: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(seasons.is_empty());
}

#[tokio::test]
async fn test_get_team_season() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let team_id = create_team(&pool, "Season Get Team", "SGT", "GetCity").await;
    insert_team_season(&pool, team_id, 2025, 12, 5, 0, Some(20)).await;

    let response = client
        .get(&format!(
            "{}/api/v1/teams/{}/seasons/2025",
            base_url, team_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get team season");

    assert_eq!(response.status(), 200);

    let season: serde_json::Value = response.json().await.unwrap();
    assert_eq!(season["team_id"], team_id.to_string());
    assert_eq!(season["season_year"], 2025);
    assert_eq!(season["wins"], 12);
    assert_eq!(season["losses"], 5);
    assert_eq!(season["ties"], 0);
    assert_eq!(season["draft_position"], 20);
}

#[tokio::test]
async fn test_get_team_season_not_found() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let random_id = Uuid::new_v4();

    let response = client
        .get(&format!(
            "{}/api/v1/teams/{}/seasons/2025",
            base_url, random_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_draft_order_by_year() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let team1_id = create_team(&pool, "Order Team 1", "OT1", "City1").await;
    let team2_id = create_team(&pool, "Order Team 2", "OT2", "City2").await;
    let team3_id = create_team(&pool, "Order Team 3", "OT3", "City3").await;

    // Draft order for 2026 uses 2025 standings
    insert_team_season(&pool, team1_id, 2025, 3, 14, 0, Some(1)).await;
    insert_team_season(&pool, team2_id, 2025, 5, 12, 0, Some(2)).await;
    insert_team_season(&pool, team3_id, 2025, 6, 11, 0, Some(3)).await;

    let response = client
        .get(&format!("{}/api/v1/draft-order?year=2026", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get draft order");

    assert_eq!(response.status(), 200);

    let order: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(order.len(), 3);

    assert_eq!(order[0]["draft_position"], 1);
    assert_eq!(order[0]["team_id"], team1_id.to_string());
    assert_eq!(order[1]["draft_position"], 2);
    assert_eq!(order[1]["team_id"], team2_id.to_string());
    assert_eq!(order[2]["draft_position"], 3);
    assert_eq!(order[2]["team_id"], team3_id.to_string());
}

#[tokio::test]
async fn test_draft_order_empty_year() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!("{}/api/v1/draft-order?year=2050", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get draft order");

    assert_eq!(response.status(), 200);

    let order: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(order.is_empty());
}
