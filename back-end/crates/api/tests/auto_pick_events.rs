use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

mod common;

/// Helper: insert a team with the given ID. Uses raw sqlx::query (not macro) to avoid
/// type-inference issues when called in loops.
async fn insert_team(
    pool: &sqlx::PgPool,
    id: Uuid,
    name: &str,
    city: &str,
    abbreviation: &str,
    conference: &str,
    division: &str,
) {
    sqlx::query(
        "INSERT INTO teams (id, name, city, abbreviation, conference, division) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(name)
    .bind(city)
    .bind(abbreviation)
    .bind(conference)
    .bind(division)
    .execute(pool)
    .await
    .unwrap();
}

/// Helper: insert a player with the given ID.
async fn insert_player(pool: &sqlx::PgPool, id: Uuid, first: &str, last: &str, position: &str) {
    sqlx::query(
        "INSERT INTO players (id, first_name, last_name, position, draft_year) VALUES ($1, $2, $3, $4, 2026)",
    )
    .bind(id)
    .bind(first)
    .bind(last)
    .bind(position)
    .execute(pool)
    .await
    .unwrap();
}

/// Helper: insert a scouting report for a player/team combo.
async fn insert_scouting_report(pool: &sqlx::PgPool, player_id: Uuid, team_id: Uuid, grade: f64) {
    sqlx::query(
        "INSERT INTO scouting_reports (id, player_id, team_id, scout_name, grade, report, strengths, weaknesses) VALUES ($1, $2, $3, 'Scout', $4, 'Good prospect', ARRAY['Talent'], ARRAY['Development'])",
    )
    .bind(Uuid::new_v4())
    .bind(player_id)
    .bind(team_id)
    .bind(grade)
    .execute(pool)
    .await
    .unwrap();
}

/// Helper: insert a draft pick.
async fn insert_draft_pick(
    pool: &sqlx::PgPool,
    id: Uuid,
    draft_id: Uuid,
    round: i32,
    pick_number: i32,
    overall_pick: i32,
    team_id: Uuid,
) {
    sqlx::query(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(draft_id)
    .bind(round)
    .bind(pick_number)
    .bind(overall_pick)
    .bind(team_id)
    .execute(pool)
    .await
    .unwrap();
}

/// Test: auto-pick-run stores PickMade events in the draft_events table
///
/// This test verifies the event sourcing audit trail is maintained during auto-pick.
/// Currently, the auto-pick-run handler broadcasts PickMade via WebSocket but does NOT
/// persist DraftEvent::pick_made() to the database. This test will fail until that gap
/// is fixed.
#[tokio::test]
async fn test_auto_pick_run_stores_pick_made_events() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Setup: draft, 2 teams, 2 players, 2 picks, session (no controlled teams)
    let draft_id = Uuid::new_v4();
    let team_1_id = Uuid::new_v4();
    let team_2_id = Uuid::new_v4();
    let player_1_id = Uuid::new_v4();
    let player_2_id = Uuid::new_v4();
    let pick_1_id = Uuid::new_v4();
    let pick_2_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 2::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    insert_team(&pool, team_1_id, "Team Alpha", "Alpha City", "ALP", "AFC", "AFC East").await;
    insert_team(&pool, team_2_id, "Team Beta", "Beta City", "BET", "NFC", "NFC East").await;

    insert_player(&pool, player_1_id, "Alpha", "Player", "QB").await;
    insert_player(&pool, player_2_id, "Beta", "Player", "WR").await;

    insert_scouting_report(&pool, player_1_id, team_1_id, 85.0).await;
    insert_scouting_report(&pool, player_2_id, team_2_id, 80.0).await;

    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3), ($4, $2, 1, 2, 2, $5)",
        pick_1_id,
        draft_id,
        team_1_id,
        pick_2_id,
        team_2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled, controlled_team_ids) VALUES ($1, $2, 'InProgress', 1, 300, true, $3)",
        session_id,
        draft_id,
        &Vec::<Uuid>::new() as &[Uuid]
    )
    .execute(&pool)
    .await
    .unwrap();

    // Action: run auto-pick
    let response = client
        .post(&format!(
            "{}/api/v1/sessions/{}/auto-pick-run",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let result: Value = response.json().await.unwrap();
    let picks_made = result["picks_made"].as_array().unwrap();
    assert_eq!(picks_made.len(), 2, "Should have made 2 picks");

    // Each pick should have a player_id
    for pick in picks_made {
        assert!(
            pick["player_id"].is_string(),
            "Each pick should have a player_id assigned"
        );
    }

    // Assert DB: draft_events table has 2 PickMade entries for this session
    let event_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_events WHERE session_id = $1 AND event_type = 'PickMade'",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        event_count.count.unwrap(),
        2,
        "Should have 2 PickMade events stored in draft_events"
    );

    // Assert DB: each event's event_data contains expected fields
    let events = sqlx::query!(
        "SELECT event_data FROM draft_events WHERE session_id = $1 AND event_type = 'PickMade' ORDER BY created_at ASC",
        session_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for event in &events {
        let data: Value = event.event_data.clone();
        assert!(data["pick_id"].is_string(), "event_data should contain pick_id");
        assert!(data["team_id"].is_string(), "event_data should contain team_id");
        assert!(
            data["player_id"].is_string(),
            "event_data should contain player_id"
        );
        assert!(data["round"].is_number(), "event_data should contain round");
        assert!(
            data["pick_number"].is_number(),
            "event_data should contain pick_number"
        );
    }

    common::cleanup_database(&pool).await;
}

/// Test: auto-pick-run response includes detailed pick information
///
/// Verifies that each pick in picks_made has round, pick_number, overall_pick,
/// picked_at timestamp, and that session.current_pick_number advances past all picks.
#[tokio::test]
async fn test_auto_pick_run_response_includes_pick_details() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = Uuid::new_v4();
    let team_1_id = Uuid::new_v4();
    let team_2_id = Uuid::new_v4();
    let player_1_id = Uuid::new_v4();
    let player_2_id = Uuid::new_v4();
    let pick_1_id = Uuid::new_v4();
    let pick_2_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 2::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    insert_team(&pool, team_1_id, "East Eagles", "East", "EEA", "AFC", "AFC East").await;
    insert_team(&pool, team_2_id, "West Wolves", "West", "WWO", "NFC", "NFC West").await;

    insert_player(&pool, player_1_id, "Jake", "Quarterback", "QB").await;
    insert_player(&pool, player_2_id, "Sam", "Receiver", "WR").await;

    insert_scouting_report(&pool, player_1_id, team_1_id, 90.0).await;
    insert_scouting_report(&pool, player_2_id, team_2_id, 88.0).await;

    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3), ($4, $2, 1, 2, 2, $5)",
        pick_1_id,
        draft_id,
        team_1_id,
        pick_2_id,
        team_2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled, controlled_team_ids) VALUES ($1, $2, 'InProgress', 1, 300, true, $3)",
        session_id,
        draft_id,
        &Vec::<Uuid>::new() as &[Uuid]
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .post(&format!(
            "{}/api/v1/sessions/{}/auto-pick-run",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let result: Value = response.json().await.unwrap();
    let picks_made = result["picks_made"].as_array().unwrap();
    assert_eq!(picks_made.len(), 2);

    // Verify each pick has detailed fields
    for pick in picks_made {
        assert!(pick["player_id"].is_string(), "player_id should be non-null");
        assert!(pick["round"].is_number(), "round should be present");
        assert!(
            pick["pick_number"].is_number(),
            "pick_number should be present"
        );
        assert!(
            pick["overall_pick"].is_number(),
            "overall_pick should be present"
        );
        assert!(
            pick["picked_at"].is_string(),
            "picked_at timestamp should be present"
        );
    }

    // Verify round/pick_number values
    assert_eq!(picks_made[0]["round"], 1);
    assert_eq!(picks_made[0]["pick_number"], 1);
    assert_eq!(picks_made[0]["overall_pick"], 1);
    assert_eq!(picks_made[1]["round"], 1);
    assert_eq!(picks_made[1]["pick_number"], 2);
    assert_eq!(picks_made[1]["overall_pick"], 2);

    // Session current_pick_number should be total picks + 1
    assert_eq!(
        result["session"]["current_pick_number"], 3,
        "current_pick_number should be total_picks + 1 (past the last pick)"
    );

    common::cleanup_database(&pool).await;
}

/// Test: auto-pick-run assigns unique players to each pick (no duplicates)
///
/// With 4 teams/players/picks and no controlled teams, auto-pick should assign
/// each player to exactly one pick.
#[tokio::test]
async fn test_auto_pick_run_all_picks_unique_players() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    // Use explicit variables (not Vec indexing) for SQLx macro compatibility
    let team_1 = Uuid::new_v4();
    let team_2 = Uuid::new_v4();
    let team_3 = Uuid::new_v4();
    let team_4 = Uuid::new_v4();
    let player_1 = Uuid::new_v4();
    let player_2 = Uuid::new_v4();
    let player_3 = Uuid::new_v4();
    let player_4 = Uuid::new_v4();
    let pick_1 = Uuid::new_v4();
    let pick_2 = Uuid::new_v4();
    let pick_3 = Uuid::new_v4();
    let pick_4 = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 4::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create 4 teams
    insert_team(&pool, team_1, "North Bears", "North", "NBR", "AFC", "AFC North").await;
    insert_team(&pool, team_2, "South Lions", "South", "SLI", "AFC", "AFC South").await;
    insert_team(&pool, team_3, "East Hawks", "East", "EHK", "NFC", "NFC East").await;
    insert_team(&pool, team_4, "West Rams", "West", "WRM", "NFC", "NFC West").await;

    // Create 4 players with scouting reports
    insert_player(&pool, player_1, "Zach", "Passer", "QB").await;
    insert_scouting_report(&pool, player_1, team_1, 90.0).await;

    insert_player(&pool, player_2, "Devon", "Runner", "RB").await;
    insert_scouting_report(&pool, player_2, team_2, 88.0).await;

    insert_player(&pool, player_3, "Miles", "Catcher", "WR").await;
    insert_scouting_report(&pool, player_3, team_3, 86.0).await;

    insert_player(&pool, player_4, "Jake", "Blocker", "OT").await;
    insert_scouting_report(&pool, player_4, team_4, 84.0).await;

    // Create 4 picks
    insert_draft_pick(&pool, pick_1, draft_id, 1, 1, 1, team_1).await;
    insert_draft_pick(&pool, pick_2, draft_id, 1, 2, 2, team_2).await;
    insert_draft_pick(&pool, pick_3, draft_id, 1, 3, 3, team_3).await;
    insert_draft_pick(&pool, pick_4, draft_id, 1, 4, 4, team_4).await;

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled, controlled_team_ids) VALUES ($1, $2, 'InProgress', 1, 300, true, $3)",
        session_id,
        draft_id,
        &Vec::<Uuid>::new() as &[Uuid]
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .post(&format!(
            "{}/api/v1/sessions/{}/auto-pick-run",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let result: Value = response.json().await.unwrap();
    let picks_made = result["picks_made"].as_array().unwrap();
    assert_eq!(picks_made.len(), 4, "All 4 picks should be made");

    // Collect all player_ids from the response
    let mut response_player_ids: Vec<String> = picks_made
        .iter()
        .map(|p| p["player_id"].as_str().unwrap().to_string())
        .collect();
    response_player_ids.sort();
    response_player_ids.dedup();
    assert_eq!(
        response_player_ids.len(),
        4,
        "All 4 picks should have unique player_ids"
    );

    // Verify DB: each draft_pick row has a different player_id
    let db_picks = sqlx::query!(
        "SELECT player_id FROM draft_picks WHERE draft_id = $1 AND player_id IS NOT NULL ORDER BY overall_pick",
        draft_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(db_picks.len(), 4, "All 4 picks should have player_id in DB");

    let mut db_player_ids: Vec<Uuid> = db_picks
        .iter()
        .map(|p| p.player_id.unwrap())
        .collect();
    db_player_ids.sort();
    db_player_ids.dedup();
    assert_eq!(
        db_player_ids.len(),
        4,
        "All 4 DB picks should have unique player_ids"
    );

    common::cleanup_database(&pool).await;
}

/// Test: auto-pick-run completes draft and stores a SessionCompleted event
///
/// With a small draft (1 round, 2 teams, 2 picks) and no controlled teams,
/// auto-pick should complete the entire draft and record completion in both
/// the session and draft tables, plus store a SessionCompleted event.
#[tokio::test]
async fn test_auto_pick_run_completes_draft_and_stores_completion_event() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = Uuid::new_v4();
    let team_1_id = Uuid::new_v4();
    let team_2_id = Uuid::new_v4();
    let player_1_id = Uuid::new_v4();
    let player_2_id = Uuid::new_v4();
    let pick_1_id = Uuid::new_v4();
    let pick_2_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 2::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    insert_team(&pool, team_1_id, "Finish First", "City A", "FFA", "AFC", "AFC East").await;
    insert_team(&pool, team_2_id, "Done Deal", "City B", "DDB", "NFC", "NFC East").await;

    insert_player(&pool, player_1_id, "First", "Pick", "QB").await;
    insert_player(&pool, player_2_id, "Second", "Pick", "RB").await;

    insert_scouting_report(&pool, player_1_id, team_1_id, 92.0).await;
    insert_scouting_report(&pool, player_2_id, team_2_id, 88.0).await;

    sqlx::query!(
        "INSERT INTO draft_picks (id, draft_id, round, pick_number, overall_pick, team_id) VALUES ($1, $2, 1, 1, 1, $3), ($4, $2, 1, 2, 2, $5)",
        pick_1_id,
        draft_id,
        team_1_id,
        pick_2_id,
        team_2_id
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled, controlled_team_ids) VALUES ($1, $2, 'InProgress', 1, 300, true, $3)",
        session_id,
        draft_id,
        &Vec::<Uuid>::new() as &[Uuid]
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .post(&format!(
            "{}/api/v1/sessions/{}/auto-pick-run",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let result: Value = response.json().await.unwrap();

    // Assert HTTP: session status is "Completed"
    assert_eq!(
        result["session"]["status"], "Completed",
        "Session status should be Completed after all picks made"
    );

    // Assert DB: drafts.status = "Completed"
    let db_draft = sqlx::query!("SELECT status FROM drafts WHERE id = $1", draft_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_draft.status, "Completed");

    // Assert DB: draft_sessions.status = "Completed" and completed_at is not null
    let db_session = sqlx::query!(
        "SELECT status, completed_at FROM draft_sessions WHERE id = $1",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_session.status, "Completed");
    assert!(
        db_session.completed_at.is_some(),
        "completed_at should be set when session completes"
    );

    // Assert DB: draft_events has a SessionCompleted event
    let completion_event_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_events WHERE session_id = $1 AND event_type = 'SessionCompleted'",
        session_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        completion_event_count.count.unwrap(),
        1,
        "Should have exactly 1 SessionCompleted event"
    );

    common::cleanup_database(&pool).await;
}

/// Test: auto-pick-run picks have sequential timestamps
///
/// Verifies that picked_at timestamps on draft_picks rows are in chronological
/// order (pick 1 <= pick 2 <= pick 3), ensuring picks are recorded sequentially.
#[tokio::test]
async fn test_auto_pick_run_picks_have_sequential_timestamps() {
    let (app_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    let draft_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let team_1 = Uuid::new_v4();
    let team_2 = Uuid::new_v4();
    let team_3 = Uuid::new_v4();
    let player_1 = Uuid::new_v4();
    let player_2 = Uuid::new_v4();
    let player_3 = Uuid::new_v4();
    let pick_1 = Uuid::new_v4();
    let pick_2 = Uuid::new_v4();
    let pick_3 = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO drafts (id, year, status, rounds, picks_per_round) VALUES ($1, 2026, 'InProgress', 1, 3::INTEGER)",
        draft_id
    )
    .execute(&pool)
    .await
    .unwrap();

    insert_team(&pool, team_1, "Seq Team A", "Seq A", "SQA", "AFC", "AFC East").await;
    insert_team(&pool, team_2, "Seq Team B", "Seq B", "SQB", "AFC", "AFC North").await;
    insert_team(&pool, team_3, "Seq Team C", "Seq C", "SQC", "NFC", "NFC East").await;

    insert_player(&pool, player_1, "Seq", "Player1", "QB").await;
    insert_scouting_report(&pool, player_1, team_1, 90.0).await;

    insert_player(&pool, player_2, "Seq", "Player2", "RB").await;
    insert_scouting_report(&pool, player_2, team_2, 87.0).await;

    insert_player(&pool, player_3, "Seq", "Player3", "WR").await;
    insert_scouting_report(&pool, player_3, team_3, 84.0).await;

    insert_draft_pick(&pool, pick_1, draft_id, 1, 1, 1, team_1).await;
    insert_draft_pick(&pool, pick_2, draft_id, 1, 2, 2, team_2).await;
    insert_draft_pick(&pool, pick_3, draft_id, 1, 3, 3, team_3).await;

    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, current_pick_number, time_per_pick_seconds, auto_pick_enabled, controlled_team_ids) VALUES ($1, $2, 'InProgress', 1, 300, true, $3)",
        session_id,
        draft_id,
        &Vec::<Uuid>::new() as &[Uuid]
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = client
        .post(&format!(
            "{}/api/v1/sessions/{}/auto-pick-run",
            app_url, session_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify DB: picked_at timestamps are in chronological order
    let db_picks = sqlx::query!(
        "SELECT overall_pick, picked_at FROM draft_picks WHERE draft_id = $1 AND picked_at IS NOT NULL ORDER BY overall_pick ASC",
        draft_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(db_picks.len(), 3, "All 3 picks should have picked_at set");

    // Verify timestamps are sequential (each pick's timestamp >= previous pick's)
    for i in 1..db_picks.len() {
        let prev_time = db_picks[i - 1].picked_at.unwrap();
        let curr_time = db_picks[i].picked_at.unwrap();
        assert!(
            curr_time >= prev_time,
            "Pick {} timestamp ({}) should be >= pick {} timestamp ({})",
            db_picks[i].overall_pick,
            curr_time,
            db_picks[i - 1].overall_pick,
            prev_time
        );
    }

    common::cleanup_database(&pool).await;
}
