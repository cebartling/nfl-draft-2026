//! Make pick error case acceptance tests

mod common;

use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn test_make_pick_nonexistent_pick_returns_404() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let random_pick_id = Uuid::new_v4();
    let random_player_id = Uuid::new_v4();

    let response = client
        .post(&format!(
            "{}/api/v1/picks/{}/make",
            base_url, random_pick_id
        ))
        .json(&json!({ "player_id": random_player_id }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_make_pick_nonexistent_player_returns_404() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create a team
    let team_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Pick Error Team', 'PET', 'Error City', 'AFC', 'AFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create draft and pick
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 1::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let pick_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3)",
        pick_id,
        draft_id,
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Try to make pick with non-existent player
    let random_player_id = Uuid::new_v4();
    let response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_id))
        .json(&json!({ "player_id": random_player_id }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);

    // Verify pick is still unmade in DB
    let db_pick = sqlx::query!("SELECT player_id FROM draft_picks WHERE id = $1", pick_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(db_pick.player_id.is_none());
}

#[tokio::test]
async fn test_make_pick_already_made_returns_400() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create team
    let team_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Already Made Team', 'AMT', 'Made City', 'AFC', 'AFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create draft
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 1::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create pick
    let pick_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3)",
        pick_id,
        draft_id,
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create two players
    let player1_id = Uuid::new_v4();
    let player2_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Player', 'One', 'QB', 2026, true)",
        player1_id
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Player', 'Two', 'RB', 2026, true)",
        player2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Make pick with first player → 200
    let first_response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_id))
        .json(&json!({ "player_id": player1_id }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to make first pick");
    assert_eq!(first_response.status(), 200);

    // Try to make same pick with second player → 400
    let second_response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_id))
        .json(&json!({ "player_id": player2_id }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send second make pick");
    assert_eq!(second_response.status(), 400);
}

#[tokio::test]
async fn test_make_pick_wrong_draft_year_returns_400() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create team
    let team_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Wrong Year Team', 'WYT', 'Wrong City', 'NFC', 'NFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create 2027 draft
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2027, 'InProgress', 1, 1::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let pick_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3)",
        pick_id,
        draft_id,
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create 2026 player (wrong year for 2027 draft)
    let player_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Wrong', 'Year', 'QB', 2026, true)",
        player_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_id))
        .json(&json!({ "player_id": player_id }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_make_pick_ineligible_player_returns_400() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create team
    let team_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO teams (id, name, abbreviation, city, conference, division) VALUES ($1, 'Ineligible Team', 'IET', 'Ineligible City', 'AFC', 'AFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create draft
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 1::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let pick_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3)",
        pick_id,
        draft_id,
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create ineligible player (draft_eligible = false)
    let player_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Not', 'Eligible', 'QB', 2026, false)",
        player_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_id))
        .json(&json!({ "player_id": player_id }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
}
