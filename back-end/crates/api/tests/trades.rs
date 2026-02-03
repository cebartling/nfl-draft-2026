//! Trade acceptance tests - End-to-end HTTP and database validation

mod common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_fair_trade_proposal() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Setup: Create teams
    let (team1_id, team2_id) = create_two_teams(&base_url, &client).await;

    // Setup: Create draft and session
    let (draft_id, session_id) = create_draft_and_session(&base_url, &client, &pool).await;

    // Setup: Initialize picks
    initialize_draft_picks(&base_url, &client, &draft_id, &pool).await;

    // Get pick IDs from database
    let picks = sqlx::query!(
        "SELECT id, overall_pick, team_id FROM draft_picks ORDER BY overall_pick LIMIT 2"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch picks");

    let pick1_id = picks[0].id;
    let pick2_id = picks[1].id;

    // Update pick ownership
    sqlx::query!(
        "UPDATE draft_picks SET team_id = $1 WHERE id = $2",
        team1_id,
        pick1_id
    )
    .execute(&pool)
    .await
    .expect("Failed to update pick1 ownership");

    sqlx::query!(
        "UPDATE draft_picks SET team_id = $1 WHERE id = $2",
        team2_id,
        pick2_id
    )
    .execute(&pool)
    .await
    .expect("Failed to update pick2 ownership");

    // Propose fair trade (pick 1 = 3000 points, pick 2 = 2600 points - within 15%)
    let response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team2_id.to_string(),
            "from_team_picks": [pick1_id.to_string()],
            "to_team_picks": [pick2_id.to_string()]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose trade");

    assert_eq!(response.status(), 201);

    let trade: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let trade_id = trade["trade"]["id"].as_str().expect("Missing trade id");

    // Verify trade HTTP response
    assert_eq!(trade["trade"]["status"], "Proposed");
    assert_eq!(trade["trade"]["from_team_value"], 3000);
    assert_eq!(trade["trade"]["to_team_value"], 2600);
    assert_eq!(trade["trade"]["value_difference"], 400);

    // Verify trade persisted in database
    let db_trade = sqlx::query!(
        "SELECT status, from_team_value, to_team_value, value_difference FROM pick_trades WHERE id = $1",
        uuid::Uuid::parse_str(trade_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Trade not found in database");

    assert_eq!(db_trade.status, "Proposed");
    assert_eq!(db_trade.from_team_value, 3000);
    assert_eq!(db_trade.to_team_value, 2600);
    assert_eq!(db_trade.value_difference, 400);

    // Verify trade details in database
    let db_details_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM pick_trade_details WHERE trade_id = $1",
        uuid::Uuid::parse_str(trade_id).expect("Invalid UUID")
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count trade details");

    assert_eq!(db_details_count.count.unwrap(), 2);
}

#[tokio::test]
async fn test_unfair_trade_rejected() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Setup
    let (team1_id, team2_id) = create_two_teams(&base_url, &client).await;
    let (draft_id, session_id) = create_draft_and_session(&base_url, &client, &pool).await;
    initialize_draft_picks(&base_url, &client, &draft_id, &pool).await;

    // Get pick 1 and pick 6 (unfair trade: 3000 vs 350 points - way outside 15%)
    let pick1 = sqlx::query!("SELECT id FROM draft_picks WHERE overall_pick = 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch pick 1");

    let pick6 = sqlx::query!("SELECT id FROM draft_picks WHERE overall_pick = 6")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch pick 6");

    // Update ownership
    sqlx::query!(
        "UPDATE draft_picks SET team_id = $1 WHERE id = $2",
        team1_id,
        pick1.id
    )
    .execute(&pool)
    .await
    .expect("Failed to update pick ownership");

    sqlx::query!(
        "UPDATE draft_picks SET team_id = $1 WHERE id = $2",
        team2_id,
        pick6.id
    )
    .execute(&pool)
    .await
    .expect("Failed to update pick ownership");

    // Attempt unfair trade
    let response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team2_id.to_string(),
            "from_team_picks": [pick1.id.to_string()],
            "to_team_picks": [pick6.id.to_string()]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose trade");

    assert_eq!(response.status(), 400);

    // Verify no trade was created in database
    let trade_count = sqlx::query!("SELECT COUNT(*) as count FROM pick_trades")
        .fetch_one(&pool)
        .await
        .expect("Failed to count trades");

    assert_eq!(trade_count.count.unwrap(), 0);
}

#[tokio::test]
async fn test_accept_trade_transfers_ownership() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Setup
    let (team1_id, team2_id) = create_two_teams(&base_url, &client).await;
    let (draft_id, session_id) = create_draft_and_session(&base_url, &client, &pool).await;
    initialize_draft_picks(&base_url, &client, &draft_id, &pool).await;

    // Get picks
    let picks = sqlx::query!("SELECT id FROM draft_picks ORDER BY overall_pick LIMIT 2")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch picks");

    let pick1_id = picks[0].id;
    let pick2_id = picks[1].id;

    // Setup ownership: team1 has pick 1, team2 has pick 2
    sqlx::query!(
        "UPDATE draft_picks SET team_id = $1 WHERE id = $2",
        team1_id,
        pick1_id
    )
    .execute(&pool)
    .await
    .expect("Failed to update ownership");

    sqlx::query!(
        "UPDATE draft_picks SET team_id = $1 WHERE id = $2",
        team2_id,
        pick2_id
    )
    .execute(&pool)
    .await
    .expect("Failed to update ownership");

    // Verify initial ownership
    let pick1_owner = sqlx::query!("SELECT team_id FROM draft_picks WHERE id = $1", pick1_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch pick");
    assert_eq!(pick1_owner.team_id, team1_id);

    // Propose trade (pick 1 for pick 2 - fair: 3000 vs 2600 = 13.3% difference, within 15%)
    let trade_response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team2_id.to_string(),
            "from_team_picks": [pick1_id.to_string()],
            "to_team_picks": [pick2_id.to_string()]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose trade");

    assert_eq!(trade_response.status(), 201);

    let trade: serde_json::Value = trade_response.json().await.expect("Failed to parse JSON");
    let trade_id = trade["trade"]["id"].as_str().expect("Missing trade id");

    // Accept trade as team2
    let accept_response = client
        .post(&format!("{}/api/v1/trades/{}/accept", base_url, trade_id))
        .json(&json!({
            "team_id": team2_id.to_string()
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to accept trade");

    let status = accept_response.status();
    if status != 200 {
        let error_body = accept_response.text().await.unwrap_or_else(|_| "Could not read body".to_string());
        panic!("Accept trade failed with status {}: {}", status, error_body);
    }

    let accepted_trade: serde_json::Value =
        accept_response.json().await.expect("Failed to parse JSON");
    assert_eq!(accepted_trade["status"], "Accepted");

    // Verify pick ownership was transferred in database
    let pick1_new_owner = sqlx::query!("SELECT team_id FROM draft_picks WHERE id = $1", pick1_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch pick");
    assert_eq!(pick1_new_owner.team_id, team2_id); // team2 now owns pick 1

    let pick2_new_owner = sqlx::query!("SELECT team_id FROM draft_picks WHERE id = $1", pick2_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch pick");
    assert_eq!(pick2_new_owner.team_id, team1_id); // team1 now owns pick 2

    // Verify trade status updated in database
    let db_trade = sqlx::query!("SELECT status FROM pick_trades WHERE id = $1", uuid::Uuid::parse_str(trade_id).unwrap())
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch trade");
    assert_eq!(db_trade.status, "Accepted");
}

#[tokio::test]
async fn test_reject_trade() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Setup
    let (team1_id, team2_id) = create_two_teams(&base_url, &client).await;
    let (draft_id, session_id) = create_draft_and_session(&base_url, &client, &pool).await;
    initialize_draft_picks(&base_url, &client, &draft_id, &pool).await;

    // Get picks and setup ownership
    let picks = sqlx::query!("SELECT id FROM draft_picks ORDER BY overall_pick LIMIT 2")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch picks");

    let pick1_id = picks[0].id;
    let pick2_id = picks[1].id;

    sqlx::query!(
        "UPDATE draft_picks SET team_id = $1 WHERE id = $2",
        team1_id,
        pick1_id
    )
    .execute(&pool)
    .await
    .expect("Failed to update ownership");

    sqlx::query!(
        "UPDATE draft_picks SET team_id = $1 WHERE id = $2",
        team2_id,
        pick2_id
    )
    .execute(&pool)
    .await
    .expect("Failed to update ownership");

    // Propose trade
    let trade_response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team2_id.to_string(),
            "from_team_picks": [pick1_id.to_string()],
            "to_team_picks": [pick2_id.to_string()]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose trade");

    let trade: serde_json::Value = trade_response.json().await.expect("Failed to parse JSON");
    let trade_id = trade["trade"]["id"].as_str().expect("Missing trade id");

    // Verify initial ownership hasn't changed
    let pick1_owner_before = sqlx::query!("SELECT team_id FROM draft_picks WHERE id = $1", pick1_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch pick");
    assert_eq!(pick1_owner_before.team_id, team1_id);

    // Reject trade as team2
    let reject_response = client
        .post(&format!("{}/api/v1/trades/{}/reject", base_url, trade_id))
        .json(&json!({
            "team_id": team2_id.to_string()
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to reject trade");

    assert_eq!(reject_response.status(), 200);

    let rejected_trade: serde_json::Value =
        reject_response.json().await.expect("Failed to parse JSON");
    assert_eq!(rejected_trade["status"], "Rejected");

    // Verify ownership unchanged in database
    let pick1_owner_after = sqlx::query!("SELECT team_id FROM draft_picks WHERE id = $1", pick1_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch pick");
    assert_eq!(pick1_owner_after.team_id, team1_id); // Still owned by team1

    let pick2_owner_after = sqlx::query!("SELECT team_id FROM draft_picks WHERE id = $1", pick2_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch pick");
    assert_eq!(pick2_owner_after.team_id, team2_id); // Still owned by team2

    // Verify trade status in database
    let db_trade = sqlx::query!("SELECT status FROM pick_trades WHERE id = $1", uuid::Uuid::parse_str(trade_id).unwrap())
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch trade");
    assert_eq!(db_trade.status, "Rejected");
}

#[tokio::test]
async fn test_pick_in_active_trade_cannot_be_traded_again() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Setup
    let (team1_id, team2_id) = create_two_teams(&base_url, &client).await;

    // Create team 3 for second trade
    let team3_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Team C",
            "abbreviation": "TMC",
            "city": "City C",
            "conference": "AFC",
            "division": "AFC West"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team 3");
    let team3: serde_json::Value = team3_response.json().await.expect("Failed to parse JSON");
    let team3_id = uuid::Uuid::parse_str(team3["id"].as_str().unwrap()).unwrap();

    // Create draft with 3 picks per round to match 3 teams
    let draft_response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 2,
            "picks_per_round": 3
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create draft");
    let draft: serde_json::Value = draft_response.json().await.expect("Failed to parse JSON");
    let draft_id = uuid::Uuid::parse_str(draft["id"].as_str().unwrap()).unwrap();

    // Create session
    let session_id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO draft_sessions (id, draft_id, status, chart_type, created_at, updated_at) VALUES ($1, $2, 'NotStarted', 'JimmyJohnson', NOW(), NOW())",
        session_id,
        draft_id
    )
    .execute(&pool)
    .await
    .expect("Failed to create session");

    // Initialize picks
    let init_response = client
        .post(&format!("{}/api/v1/drafts/{}/initialize", base_url, draft_id.to_string()))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to initialize picks");
    assert_eq!(init_response.status(), 201);

    // Get picks
    let picks = sqlx::query!("SELECT id FROM draft_picks ORDER BY overall_pick LIMIT 3")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch picks");

    let pick1_id = picks[0].id;
    let pick2_id = picks[1].id;
    let pick3_id = picks[2].id;

    // Setup ownership
    sqlx::query!("UPDATE draft_picks SET team_id = $1 WHERE id = $2", team1_id, pick1_id)
        .execute(&pool)
        .await
        .expect("Failed to update ownership");
    sqlx::query!("UPDATE draft_picks SET team_id = $1 WHERE id = $2", team2_id, pick2_id)
        .execute(&pool)
        .await
        .expect("Failed to update ownership");
    sqlx::query!("UPDATE draft_picks SET team_id = $1 WHERE id = $2", team3_id, pick3_id)
        .execute(&pool)
        .await
        .expect("Failed to update ownership");

    // Propose first trade (pick1 for pick2)
    let trade1_response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team2_id.to_string(),
            "from_team_picks": [pick1_id.to_string()],
            "to_team_picks": [pick2_id.to_string()]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose first trade");

    assert_eq!(trade1_response.status(), 201);

    // Try to propose second trade with same pick (pick1 for pick3) - should fail
    let trade2_response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team3_id.to_string(),
            "from_team_picks": [pick1_id.to_string()],
            "to_team_picks": [pick3_id.to_string()]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose second trade");

    assert_eq!(trade2_response.status(), 400);

    // Verify only one trade in database
    let trade_count = sqlx::query!("SELECT COUNT(*) as count FROM pick_trades")
        .fetch_one(&pool)
        .await
        .expect("Failed to count trades");
    assert_eq!(trade_count.count.unwrap(), 1);
}

#[tokio::test]
async fn test_get_pending_trades_for_team() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Setup
    let (team1_id, team2_id) = create_two_teams(&base_url, &client).await;
    let (draft_id, session_id) = create_draft_and_session(&base_url, &client, &pool).await;
    initialize_draft_picks(&base_url, &client, &draft_id, &pool).await;

    // Get picks (need 6 picks total)
    let picks = sqlx::query!("SELECT id FROM draft_picks ORDER BY overall_pick LIMIT 6")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch picks");

    // Setup: team1 owns picks 1, 3, 5 and team2 owns picks 2, 4, 6
    for i in 0..6 {
        let team_id = if i % 2 == 0 { team1_id } else { team2_id };
        sqlx::query!("UPDATE draft_picks SET team_id = $1 WHERE id = $2", team_id, picks[i].id)
            .execute(&pool)
            .await
            .expect("Failed to update ownership");
    }

    // Team1 proposes two fair trades to Team2
    // Trade 1: pick 1 (3000) for pick 2 (2600) - 13.3% difference, within 15%
    let trade1_response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team2_id.to_string(),
            "from_team_picks": [picks[0].id.to_string()],  // pick 1
            "to_team_picks": [picks[1].id.to_string()]     // pick 2
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose trade 1");
    assert_eq!(trade1_response.status(), 201);

    // Trade 2: pick 3 (2200) for pick 4 (1800) - 18.2% difference, but still outside 15%
    // Let's use pick 3 for pick 2... wait, pick 2 is already in trade 1
    // Instead: pick 5 (1700) for pick 6 (1600) - 5.9% difference, within 15%
    let trade2_response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team2_id.to_string(),
            "from_team_picks": [picks[4].id.to_string()],  // pick 5
            "to_team_picks": [picks[5].id.to_string()]     // pick 6
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose trade 2");
    assert_eq!(trade2_response.status(), 201);

    // Get pending trades for team2
    let response = client
        .get(&format!(
            "{}/api/v1/teams/{}/pending-trades",
            base_url,
            team2_id.to_string()
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get pending trades");

    assert_eq!(response.status(), 200);

    let pending_trades: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let trades = pending_trades.as_array().expect("Expected array");
    assert_eq!(trades.len(), 2);

    // Verify both trades are for team2
    for trade in trades {
        assert_eq!(
            trade["trade"]["to_team_id"].as_str().unwrap(),
            team2_id.to_string()
        );
        assert_eq!(trade["trade"]["status"], "Proposed");
    }

    // Verify pending trades count in database
    let db_pending = sqlx::query!(
        "SELECT COUNT(*) as count FROM pick_trades WHERE to_team_id = $1 AND status = 'Proposed'",
        team2_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count pending trades");
    assert_eq!(db_pending.count.unwrap(), 2);
}

#[tokio::test]
async fn test_get_trade_details() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Setup
    let (team1_id, team2_id) = create_two_teams(&base_url, &client).await;
    let (draft_id, session_id) = create_draft_and_session(&base_url, &client, &pool).await;
    initialize_draft_picks(&base_url, &client, &draft_id, &pool).await;

    let picks = sqlx::query!("SELECT id FROM draft_picks ORDER BY overall_pick LIMIT 2")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch picks");

    let pick1_id = picks[0].id;
    let pick2_id = picks[1].id;

    sqlx::query!("UPDATE draft_picks SET team_id = $1 WHERE id = $2", team1_id, pick1_id)
        .execute(&pool)
        .await
        .expect("Failed to update ownership");
    sqlx::query!("UPDATE draft_picks SET team_id = $1 WHERE id = $2", team2_id, pick2_id)
        .execute(&pool)
        .await
        .expect("Failed to update ownership");

    // Propose trade
    let trade_response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team2_id.to_string(),
            "from_team_picks": [pick1_id.to_string()],
            "to_team_picks": [pick2_id.to_string()]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose trade");

    let trade: serde_json::Value = trade_response.json().await.expect("Failed to parse JSON");
    let trade_id = trade["trade"]["id"].as_str().expect("Missing trade id");

    // Get trade details
    let details_response = client
        .get(&format!("{}/api/v1/trades/{}", base_url, trade_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get trade details");

    assert_eq!(details_response.status(), 200);

    let details: serde_json::Value = details_response.json().await.expect("Failed to parse JSON");
    assert_eq!(details["trade"]["id"], trade_id);
    assert_eq!(details["trade"]["status"], "Proposed");
    assert_eq!(details["from_team_picks"].as_array().unwrap().len(), 1);
    assert_eq!(details["to_team_picks"].as_array().unwrap().len(), 1);
    assert_eq!(
        details["from_team_picks"][0].as_str().unwrap(),
        pick1_id.to_string()
    );
    assert_eq!(
        details["to_team_picks"][0].as_str().unwrap(),
        pick2_id.to_string()
    );
}

#[tokio::test]
async fn test_cannot_accept_trade_as_wrong_team() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // Setup
    let (team1_id, team2_id) = create_two_teams(&base_url, &client).await;
    let (draft_id, session_id) = create_draft_and_session(&base_url, &client, &pool).await;
    initialize_draft_picks(&base_url, &client, &draft_id, &pool).await;

    let picks = sqlx::query!("SELECT id FROM draft_picks ORDER BY overall_pick LIMIT 2")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch picks");

    sqlx::query!("UPDATE draft_picks SET team_id = $1 WHERE id = $2", team1_id, picks[0].id)
        .execute(&pool)
        .await
        .expect("Failed to update ownership");
    sqlx::query!("UPDATE draft_picks SET team_id = $1 WHERE id = $2", team2_id, picks[1].id)
        .execute(&pool)
        .await
        .expect("Failed to update ownership");

    // Propose trade from team1 to team2
    let trade_response = client
        .post(&format!("{}/api/v1/trades", base_url))
        .json(&json!({
            "session_id": session_id.to_string(),
            "from_team_id": team1_id.to_string(),
            "to_team_id": team2_id.to_string(),
            "from_team_picks": [picks[0].id.to_string()],
            "to_team_picks": [picks[1].id.to_string()]
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to propose trade");

    let trade: serde_json::Value = trade_response.json().await.expect("Failed to parse JSON");
    let trade_id = trade["trade"]["id"].as_str().expect("Missing trade id");

    // Try to accept as team1 (the proposing team) - should fail
    let accept_response = client
        .post(&format!("{}/api/v1/trades/{}/accept", base_url, trade_id))
        .json(&json!({
            "team_id": team1_id.to_string()
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to accept trade");

    assert_eq!(accept_response.status(), 400);

    // Verify trade still in Proposed status
    let db_trade = sqlx::query!(
        "SELECT status FROM pick_trades WHERE id = $1",
        uuid::Uuid::parse_str(trade_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch trade");
    assert_eq!(db_trade.status, "Proposed");
}

// Helper functions

async fn create_two_teams(base_url: &str, client: &reqwest::Client) -> (uuid::Uuid, uuid::Uuid) {
    let team1_response = client
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
        .expect("Failed to create team 1");

    let team1: serde_json::Value = team1_response.json().await.expect("Failed to parse JSON");
    let team1_id = uuid::Uuid::parse_str(team1["id"].as_str().unwrap()).unwrap();

    let team2_response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "Team B",
            "abbreviation": "TMB",
            "city": "City B",
            "conference": "NFC",
            "division": "NFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create team 2");

    let team2: serde_json::Value = team2_response.json().await.expect("Failed to parse JSON");
    let team2_id = uuid::Uuid::parse_str(team2["id"].as_str().unwrap()).unwrap();

    (team1_id, team2_id)
}

async fn create_draft_and_session(
    base_url: &str,
    client: &reqwest::Client,
    pool: &sqlx::PgPool,
) -> (uuid::Uuid, uuid::Uuid) {
    // Create draft (smaller to match the 2 teams we create in tests)
    let draft_response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 3,
            "picks_per_round": 2
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create draft");

    let draft: serde_json::Value = draft_response.json().await.expect("Failed to parse JSON");
    let draft_id = uuid::Uuid::parse_str(draft["id"].as_str().unwrap()).unwrap();

    // Create session directly in database (simpler than making API call)
    let session_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO draft_sessions (id, draft_id, status, chart_type, created_at, updated_at)
        VALUES ($1, $2, 'NotStarted', 'JimmyJohnson', NOW(), NOW())
        "#,
        session_id,
        draft_id
    )
    .execute(pool)
    .await
    .expect("Failed to create session");

    (draft_id, session_id)
}

async fn initialize_draft_picks(
    base_url: &str,
    client: &reqwest::Client,
    draft_id: &uuid::Uuid,
    pool: &sqlx::PgPool,
) {
    // Initialize picks via API
    let init_response = client
        .post(&format!(
            "{}/api/v1/drafts/{}/initialize",
            base_url,
            draft_id.to_string()
        ))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to initialize picks");

    let status = init_response.status();
    if status != 201 {
        let error_body = init_response.text().await.unwrap_or_else(|_| "Could not read body".to_string());
        panic!("Initialize failed with status {}: {}", status, error_body);
    }

    // Verify picks were created
    let pick_count = sqlx::query!("SELECT COUNT(*) as count FROM draft_picks WHERE draft_id = $1", draft_id)
        .fetch_one(pool)
        .await
        .expect("Failed to count picks");

    assert_eq!(pick_count.count.unwrap(), 6); // 3 rounds * 2 picks
}
