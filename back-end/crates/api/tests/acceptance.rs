use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::sync::oneshot;

/// Spawns the API server on an ephemeral port and returns the base URL
async fn spawn_app() -> String {
    // Setup database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create pool");

    // Cleanup database
    cleanup_database(&pool).await;

    let state = api::state::AppState::new(pool);
    let app = api::routes::create_router(state);

    // Bind to ephemeral port (port 0)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to ephemeral port");

    let addr = listener.local_addr().expect("Failed to get local address");
    let base_url = format!("http://{}", addr);

    // Create channel to notify when server is ready
    let (tx, rx) = oneshot::channel();

    // Spawn server in background task
    tokio::spawn(async move {
        // Notify that server is about to start
        tx.send(()).unwrap();
        
        axum::serve(listener, app)
            .await
            .expect("Server failed to start");
    });

    // Wait for server to be ready
    rx.await.expect("Server failed to start");

    // Give server a moment to fully initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    base_url
}

async fn cleanup_database(pool: &sqlx::PgPool) {
    sqlx::query!("DELETE FROM draft_picks")
        .execute(pool)
        .await
        .expect("Failed to cleanup picks");
    sqlx::query!("DELETE FROM drafts")
        .execute(pool)
        .await
        .expect("Failed to cleanup drafts");
    sqlx::query!("DELETE FROM players")
        .execute(pool)
        .await
        .expect("Failed to cleanup players");
    sqlx::query!("DELETE FROM teams")
        .execute(pool)
        .await
        .expect("Failed to cleanup teams");
}

/// Creates a configured reqwest client with sensible defaults
fn create_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client")
}

#[tokio::test]
async fn test_health_check() {
    let base_url = spawn_app().await;
    let client = create_client();

    let response = client
        .get(&format!("{}/health", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_create_and_get_team() {
    let base_url = spawn_app().await;
    let client = create_client();

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

    // Get team
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
}

#[tokio::test]
async fn test_create_and_get_player() {
    let base_url = spawn_app().await;
    let client = create_client();

    // Create player
    let create_response = client
        .post(&format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "John",
            "last_name": "Doe",
            "position": "QB",
            "college": "Texas",
            "height_inches": 75,
            "weight_pounds": 220,
            "draft_year": 2026
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create player");

    assert_eq!(create_response.status(), 201);

    let created_player: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    let player_id = created_player["id"].as_str().expect("Missing player id");

    // Get player
    let get_response = client
        .get(&format!("{}/api/v1/players/{}", base_url, player_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get player");

    assert_eq!(get_response.status(), 200);

    let player: serde_json::Value = get_response.json().await.expect("Failed to parse JSON");
    assert_eq!(player["first_name"], "John");
    assert_eq!(player["last_name"], "Doe");
    assert_eq!(player["position"], "QB");
}

#[tokio::test]
async fn test_draft_flow() {
    let base_url = spawn_app().await;
    let client = create_client();

    // Create two teams
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
    assert_eq!(team1_response.status(), 201);

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
    assert_eq!(team2_response.status(), 201);

    // Create a player
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

    // Create draft
    let draft_response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 1,
            "picks_per_round": 2
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create draft");
    assert_eq!(draft_response.status(), 201);

    let draft: serde_json::Value = draft_response.json().await.expect("Failed to parse JSON");
    let draft_id = draft["id"].as_str().expect("Missing draft id");
    assert_eq!(draft["year"], 2026);
    assert_eq!(draft["status"], "NotStarted");
    assert_eq!(draft["total_picks"], 2);

    // Initialize draft picks
    let init_response = client
        .post(&format!("{}/api/v1/drafts/{}/initialize", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to initialize picks");
    assert_eq!(init_response.status(), 201);

    let picks: Vec<serde_json::Value> = init_response.json().await.expect("Failed to parse JSON");
    assert_eq!(picks.len(), 2);
    assert_eq!(picks[0]["round"], 1);
    assert_eq!(picks[0]["pick_number"], 1);
    assert_eq!(picks[0]["overall_pick"], 1);

    // Get next pick
    let next_pick_response = client
        .get(&format!("{}/api/v1/drafts/{}/picks/next", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get next pick");
    assert_eq!(next_pick_response.status(), 200);

    let next_pick: serde_json::Value = next_pick_response.json().await.expect("Failed to parse JSON");
    let pick_id = next_pick["id"].as_str().expect("Missing pick id");

    // Start draft
    let start_response = client
        .post(&format!("{}/api/v1/drafts/{}/start", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to start draft");
    assert_eq!(start_response.status(), 200);

    let started_draft: serde_json::Value = start_response.json().await.expect("Failed to parse JSON");
    assert_eq!(started_draft["status"], "InProgress");

    // Make pick
    let make_pick_response = client
        .post(&format!("{}/api/v1/picks/{}/make", base_url, pick_id))
        .json(&json!({
            "player_id": player_id
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to make pick");
    assert_eq!(make_pick_response.status(), 200);

    let made_pick: serde_json::Value = make_pick_response.json().await.expect("Failed to parse JSON");
    assert_eq!(made_pick["player_id"], player_id);
    assert!(made_pick["picked_at"].is_string());

    // Pause draft
    let pause_response = client
        .post(&format!("{}/api/v1/drafts/{}/pause", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to pause draft");
    assert_eq!(pause_response.status(), 200);

    let paused_draft: serde_json::Value = pause_response.json().await.expect("Failed to parse JSON");
    assert_eq!(paused_draft["status"], "Paused");

    // Resume draft (start again)
    let resume_response = client
        .post(&format!("{}/api/v1/drafts/{}/start", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to resume draft");
    assert_eq!(resume_response.status(), 200);

    // Complete draft
    let complete_response = client
        .post(&format!("{}/api/v1/drafts/{}/complete", base_url, draft_id))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to complete draft");
    assert_eq!(complete_response.status(), 200);

    let completed_draft: serde_json::Value = complete_response.json().await.expect("Failed to parse JSON");
    assert_eq!(completed_draft["status"], "Completed");
}

#[tokio::test]
async fn test_list_endpoints() {
    let base_url = spawn_app().await;
    let client = create_client();

    // Create some test data
    client
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
        .expect("Failed to create team");

    client
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

    client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 1
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create draft");

    // List teams
    let teams_response = client
        .get(&format!("{}/api/v1/teams", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list teams");
    assert_eq!(teams_response.status(), 200);
    let teams: Vec<serde_json::Value> = teams_response.json().await.expect("Failed to parse JSON");
    assert!(!teams.is_empty());

    // List players
    let players_response = client
        .get(&format!("{}/api/v1/players", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list players");
    assert_eq!(players_response.status(), 200);
    let players: Vec<serde_json::Value> = players_response.json().await.expect("Failed to parse JSON");
    assert!(!players.is_empty());

    // List drafts
    let drafts_response = client
        .get(&format!("{}/api/v1/drafts", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to list drafts");
    assert_eq!(drafts_response.status(), 200);
    let drafts: Vec<serde_json::Value> = drafts_response.json().await.expect("Failed to parse JSON");
    assert!(!drafts.is_empty());
}

#[tokio::test]
async fn test_error_handling() {
    let base_url = spawn_app().await;
    let client = create_client();

    // Try to get non-existent team
    let response = client
        .get(&format!("{}/api/v1/teams/00000000-0000-0000-0000-000000000000", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), 404);

    // Try to create team with invalid data
    let response = client
        .post(&format!("{}/api/v1/teams", base_url))
        .json(&json!({
            "name": "",
            "abbreviation": "DAL",
            "city": "Dallas",
            "conference": "NFC",
            "division": "NFC East"
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), 400);

    // Try to create duplicate draft year
    client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create first draft");

    let response = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 32
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), 409);
}
