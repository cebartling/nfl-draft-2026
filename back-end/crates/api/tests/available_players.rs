//! Acceptance tests for GET /api/v1/drafts/:id/available-players

mod common;

use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_available_players_returns_all_when_no_picks_made() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 1::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create players
    let player1_id = Uuid::new_v4();
    let player2_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Alpha', 'Player', 'QB', 2026, true)",
        player1_id
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Beta', 'Player', 'WR', 2026, true)",
        player2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/available-players",
            app_url, draft_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let players: Vec<Value> = response.json().await.unwrap();
    assert_eq!(players.len(), 2);

    // All players should have rankings array (possibly empty)
    for p in &players {
        assert!(p["rankings"].is_array());
    }

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_available_players_excludes_drafted_players() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create team and draft
    let team_id = Uuid::new_v4();
    let draft_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO teams (id, name, city, abbreviation, conference, division) VALUES ($1, 'Test Team', 'Test', 'TST', 'AFC', 'AFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 1::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create two players
    let drafted_player_id = Uuid::new_v4();
    let available_player_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Drafted', 'Player', 'QB', 2026, true)",
        drafted_player_id
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Available', 'Player', 'RB', 2026, true)",
        available_player_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create a pick with the drafted player assigned
    let pick_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id, player_id) VALUES ($1, $2, 1, 1, 1, $3, $4)",
        pick_id,
        draft_id,
        team_id,
        drafted_player_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/available-players",
            app_url, draft_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let players: Vec<Value> = response.json().await.unwrap();
    assert_eq!(players.len(), 1);
    assert_eq!(players[0]["id"], available_player_id.to_string());
    assert_eq!(players[0]["first_name"], "Available");

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_available_players_includes_rankings() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create draft
    let draft_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 1::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create player
    let player_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Ranked', 'Player', 'QB', 2026, true)",
        player_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create ranking source (DB stores name, url, description; abbreviation is derived in domain)
    let source_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO ranking_sources (id, name) VALUES ($1, 'Test Source')",
        source_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create prospect ranking
    sqlx::query!(
        "INSERT INTO prospect_rankings (id, player_id, ranking_source_id, rank, scraped_at) VALUES ($1, $2, $3, 1, '2026-01-15')",
        Uuid::new_v4(),
        player_id,
        source_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/available-players",
            app_url, draft_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let players: Vec<Value> = response.json().await.unwrap();
    assert_eq!(players.len(), 1);

    let rankings = players[0]["rankings"].as_array().unwrap();
    assert_eq!(rankings.len(), 1);
    assert_eq!(rankings[0]["source_name"], "Test Source");
    assert_eq!(rankings[0]["rank"], 1);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_available_players_includes_scouting_data_for_team() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create team and draft
    let team_id = Uuid::new_v4();
    let draft_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO teams (id, name, city, abbreviation, conference, division) VALUES ($1, 'Scout Team', 'Test', 'SCT', 'NFC', 'NFC East')",
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 1::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create player
    let player_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO players (id, first_name, last_name, position, draft_year, draft_eligible) VALUES ($1, 'Scouted', 'Player', 'QB', 2026, true)",
        player_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create scouting report
    sqlx::query!(
        "INSERT INTO scouting_reports (id, player_id, team_id, grade, fit_grade, injury_concern, character_concern) VALUES ($1, $2, $3, 8.5, 'A', false, false)",
        Uuid::new_v4(),
        player_id,
        team_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/available-players?team_id={}",
            app_url, draft_id, team_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let players: Vec<Value> = response.json().await.unwrap();
    assert_eq!(players.len(), 1);
    assert_eq!(players[0]["scouting_grade"], 8.5);
    assert_eq!(players[0]["fit_grade"], "A");
    assert_eq!(players[0]["injury_concern"], false);
    assert_eq!(players[0]["character_concern"], false);

    common::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_available_players_returns_404_for_nonexistent_draft() {
    let (app_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let fake_draft_id = Uuid::new_v4();

    let response = client
        .get(&format!(
            "{}/api/v1/drafts/{}/available-players",
            app_url, fake_draft_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
