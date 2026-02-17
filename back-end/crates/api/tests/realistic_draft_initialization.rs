//! Acceptance tests for realistic draft initialization with trade data
//!
//! These tests verify that when a realistic draft is initialized via the API,
//! the picks come from the draft order JSON data (with trade metadata, compensatory
//! picks, etc.) rather than from a simple standings-based team ordering.

mod common;

use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

/// All 32 NFL team abbreviations + names for seeding
const NFL_TEAMS: &[(&str, &str, &str, &str, &str)] = &[
    ("Arizona Cardinals", "ARI", "Glendale", "NFC", "NFC West"),
    ("Atlanta Falcons", "ATL", "Atlanta", "NFC", "NFC South"),
    ("Baltimore Ravens", "BAL", "Baltimore", "AFC", "AFC North"),
    ("Buffalo Bills", "BUF", "Buffalo", "AFC", "AFC East"),
    ("Carolina Panthers", "CAR", "Charlotte", "NFC", "NFC South"),
    ("Chicago Bears", "CHI", "Chicago", "NFC", "NFC North"),
    ("Cincinnati Bengals", "CIN", "Cincinnati", "AFC", "AFC North"),
    ("Cleveland Browns", "CLE", "Cleveland", "AFC", "AFC North"),
    ("Dallas Cowboys", "DAL", "Dallas", "NFC", "NFC East"),
    ("Denver Broncos", "DEN", "Denver", "AFC", "AFC West"),
    ("Detroit Lions", "DET", "Detroit", "NFC", "NFC North"),
    ("Green Bay Packers", "GB", "Green Bay", "NFC", "NFC North"),
    ("Houston Texans", "HOU", "Houston", "AFC", "AFC South"),
    ("Indianapolis Colts", "IND", "Indianapolis", "AFC", "AFC South"),
    ("Jacksonville Jaguars", "JAX", "Jacksonville", "AFC", "AFC South"),
    ("Kansas City Chiefs", "KC", "Kansas City", "AFC", "AFC West"),
    ("Los Angeles Chargers", "LAC", "Los Angeles", "AFC", "AFC West"),
    ("Los Angeles Rams", "LAR", "Los Angeles", "NFC", "NFC West"),
    ("Las Vegas Raiders", "LV", "Las Vegas", "AFC", "AFC West"),
    ("Miami Dolphins", "MIA", "Miami", "AFC", "AFC East"),
    ("Minnesota Vikings", "MIN", "Minneapolis", "NFC", "NFC North"),
    ("New England Patriots", "NE", "Foxborough", "AFC", "AFC East"),
    ("New Orleans Saints", "NO", "New Orleans", "NFC", "NFC South"),
    ("New York Giants", "NYG", "East Rutherford", "NFC", "NFC East"),
    ("New York Jets", "NYJ", "East Rutherford", "AFC", "AFC East"),
    ("Philadelphia Eagles", "PHI", "Philadelphia", "NFC", "NFC East"),
    ("Pittsburgh Steelers", "PIT", "Pittsburgh", "AFC", "AFC North"),
    ("Seattle Seahawks", "SEA", "Seattle", "NFC", "NFC West"),
    ("San Francisco 49ers", "SF", "Santa Clara", "NFC", "NFC West"),
    ("Tampa Bay Buccaneers", "TB", "Tampa", "NFC", "NFC South"),
    ("Tennessee Titans", "TEN", "Nashville", "AFC", "AFC South"),
    ("Washington Commanders", "WAS", "Landover", "NFC", "NFC East"),
];

async fn seed_all_nfl_teams(client: &reqwest::Client, base_url: &str) {
    for &(name, abbr, city, conference, division) in NFL_TEAMS {
        let resp = client
            .post(&format!("{}/api/v1/teams", base_url))
            .json(&json!({
                "name": name,
                "abbreviation": abbr,
                "city": city,
                "conference": conference,
                "division": division,
            }))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .unwrap_or_else(|_| panic!("Failed to create team {}", abbr));
        assert_eq!(
            resp.status(),
            201,
            "Failed to create team {} ({})",
            name,
            abbr
        );
    }
}

#[tokio::test]
async fn test_realistic_draft_initialize_uses_trade_data() {
    let (base_url, pool) = common::spawn_app().await;
    let client = common::create_client();

    // 1. Seed all 32 NFL teams
    seed_all_nfl_teams(&client, &base_url).await;

    // Verify teams were created
    let team_count = sqlx::query!("SELECT COUNT(*) as count FROM teams")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(team_count.count.unwrap(), 32);

    // 2. Create a realistic draft (picks_per_round = null)
    let draft_resp = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "name": "Realistic Draft 2026",
            "year": 2026,
            "rounds": 7
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to create realistic draft");
    assert_eq!(draft_resp.status(), 201);

    let draft: serde_json::Value = draft_resp.json().await.unwrap();
    let draft_id = draft["id"].as_str().unwrap();
    assert!(draft["is_realistic"].as_bool().unwrap());
    assert!(draft["picks_per_round"].is_null());

    // 3. Initialize picks via API
    let init_resp = client
        .post(&format!(
            "{}/api/v1/drafts/{}/initialize",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to initialize draft picks");
    assert_eq!(init_resp.status(), 201);

    let picks: Vec<serde_json::Value> = init_resp.json().await.unwrap();

    // 4. Assert total pick count matches JSON data (257, not 7*32=224)
    assert_eq!(
        picks.len(),
        257,
        "Realistic draft should have 257 picks from JSON data, not {}",
        picks.len()
    );

    // 5. Find pick #20 and verify it's DAL (traded from GB)
    let pick_20 = picks
        .iter()
        .find(|p| p["overall_pick"].as_i64() == Some(20))
        .expect("Pick #20 not found");

    // Look up DAL team_id
    let dal_team = sqlx::query!("SELECT id FROM teams WHERE abbreviation = 'DAL'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let gb_team = sqlx::query!("SELECT id FROM teams WHERE abbreviation = 'GB'")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(
        pick_20["team_id"].as_str().unwrap(),
        dal_team.id.to_string(),
        "Pick #20 should belong to DAL"
    );
    assert_eq!(
        pick_20["is_traded"].as_bool().unwrap(),
        true,
        "Pick #20 should be marked as traded"
    );
    assert_eq!(
        pick_20["original_team_id"].as_str().unwrap(),
        gb_team.id.to_string(),
        "Pick #20 should have original_team_id = GB"
    );

    // 6. Assert compensatory picks exist
    let comp_picks: Vec<&serde_json::Value> = picks
        .iter()
        .filter(|p| p["is_compensatory"].as_bool() == Some(true))
        .collect();
    assert!(
        comp_picks.len() > 0,
        "Realistic draft should have compensatory picks"
    );

    // Compensatory picks should be in rounds 3-7
    for comp in &comp_picks {
        let round = comp["round"].as_i64().unwrap();
        assert!(
            (3..=7).contains(&round),
            "Compensatory pick in invalid round: {}",
            round
        );
    }

    // 7. Verify via GET endpoint too
    let get_resp = client
        .get(&format!(
            "{}/api/v1/drafts/{}/picks",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to get picks");
    assert_eq!(get_resp.status(), 200);

    let get_picks: Vec<serde_json::Value> = get_resp.json().await.unwrap();
    assert_eq!(get_picks.len(), 257);

    // 8. Verify in database directly
    let db_pick_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM draft_picks WHERE draft_id = $1",
        Uuid::parse_str(draft_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_pick_count.count.unwrap(), 257);

    // Verify traded pick in database
    let db_pick_20 = sqlx::query!(
        "SELECT team_id, original_team_id, is_compensatory, notes FROM draft_picks WHERE draft_id = $1 AND overall_pick = 20",
        Uuid::parse_str(draft_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(db_pick_20.team_id, dal_team.id);
    assert_eq!(db_pick_20.original_team_id, Some(gb_team.id));
    assert!(!db_pick_20.is_compensatory);
    assert_eq!(db_pick_20.notes, Some("From GB".to_string()));
}

#[tokio::test]
async fn test_custom_draft_initialize_unchanged() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    // Create two teams
    for (name, abbr, city) in [("Team A", "TMA", "City A"), ("Team B", "TMB", "City B")] {
        let resp = client
            .post(&format!("{}/api/v1/teams", base_url))
            .json(&json!({
                "name": name,
                "abbreviation": abbr,
                "city": city,
                "conference": "AFC",
                "division": "AFC East"
            }))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 201);
    }

    // Create a custom (non-realistic) draft with picks_per_round = 2
    let draft_resp = client
        .post(&format!("{}/api/v1/drafts", base_url))
        .json(&json!({
            "name": "Custom Draft",
            "year": 2026,
            "rounds": 7,
            "picks_per_round": 2
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .unwrap();
    assert_eq!(draft_resp.status(), 201);

    let draft: serde_json::Value = draft_resp.json().await.unwrap();
    let draft_id = draft["id"].as_str().unwrap();
    assert!(!draft["is_realistic"].as_bool().unwrap());

    // Initialize picks - should use the old standings-based logic
    let init_resp = client
        .post(&format!(
            "{}/api/v1/drafts/{}/initialize",
            base_url, draft_id
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .unwrap();
    assert_eq!(init_resp.status(), 201);

    let picks: Vec<serde_json::Value> = init_resp.json().await.unwrap();
    // 2 teams * 7 rounds = 14 picks (standard custom draft behavior)
    assert_eq!(picks.len(), 14);

    // No traded or compensatory picks in custom drafts
    for pick in &picks {
        assert_eq!(pick["is_traded"].as_bool().unwrap(), false);
        assert_eq!(pick["is_compensatory"].as_bool().unwrap(), false);
    }
}
