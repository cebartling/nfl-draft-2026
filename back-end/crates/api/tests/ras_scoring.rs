mod common;

use serde_json::json;

/// Helper to create a player and return their ID
async fn create_player(client: &reqwest::Client, base_url: &str) -> uuid::Uuid {
    let resp = client
        .post(format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Travis",
            "last_name": "Hunter",
            "position": "CB",
            "draft_year": 2026,
            "height_inches": 73,
            "weight_pounds": 185
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let player: serde_json::Value = resp.json().await.unwrap();
    uuid::Uuid::parse_str(player["id"].as_str().unwrap()).unwrap()
}

/// Helper to create combine results for a player
async fn create_combine_results(
    client: &reqwest::Client,
    base_url: &str,
    player_id: uuid::Uuid,
) {
    let resp = client
        .post(format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "source": "combine",
            "forty_yard_dash": 4.38,
            "bench_press": 15,
            "vertical_jump": 39.0,
            "broad_jump": 128,
            "three_cone_drill": 6.72,
            "twenty_yard_shuttle": 4.05,
            "arm_length": 32.0,
            "hand_size": 9.25,
            "wingspan": 76.5,
            "ten_yard_split": 1.50,
            "twenty_yard_split": 2.52
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
}

/// Helper to seed percentile data
async fn seed_percentiles(client: &reqwest::Client, base_url: &str) {
    // All breakpoints stored in ascending order (min < p10 < ... < p90 < max)
    // For timed events: lower values are better, inversion happens in scoring
    let body = json!({
        "percentiles": [
            {
                "position": "CB",
                "measurement": "forty_yard_dash",
                "sample_size": 300,
                "min_value": 4.24, "p10": 4.33, "p20": 4.37, "p30": 4.40,
                "p40": 4.42, "p50": 4.45, "p60": 4.48, "p70": 4.50,
                "p80": 4.54, "p90": 4.58, "max_value": 4.80
            },
            {
                "position": "CB",
                "measurement": "bench_press",
                "sample_size": 250,
                "min_value": 5.0, "p10": 9.0, "p20": 11.0, "p30": 12.0,
                "p40": 13.0, "p50": 14.0, "p60": 15.0, "p70": 16.0,
                "p80": 18.0, "p90": 20.0, "max_value": 28.0
            },
            {
                "position": "CB",
                "measurement": "vertical_jump",
                "sample_size": 300,
                "min_value": 28.0, "p10": 31.0, "p20": 33.0, "p30": 34.0,
                "p40": 35.0, "p50": 37.0, "p60": 38.0, "p70": 39.0,
                "p80": 40.0, "p90": 42.0, "max_value": 46.0
            },
            {
                "position": "CB",
                "measurement": "broad_jump",
                "sample_size": 300,
                "min_value": 108.0, "p10": 114.0, "p20": 117.0, "p30": 119.0,
                "p40": 121.0, "p50": 122.0, "p60": 124.0, "p70": 125.0,
                "p80": 127.0, "p90": 130.0, "max_value": 138.0
            },
            {
                "position": "CB",
                "measurement": "three_cone_drill",
                "sample_size": 200,
                "min_value": 6.40, "p10": 6.55, "p20": 6.63, "p30": 6.70,
                "p40": 6.75, "p50": 6.80, "p60": 6.87, "p70": 6.92,
                "p80": 6.98, "p90": 7.05, "max_value": 7.40
            },
            {
                "position": "CB",
                "measurement": "twenty_yard_shuttle",
                "sample_size": 200,
                "min_value": 3.86, "p10": 3.96, "p20": 4.00, "p30": 4.04,
                "p40": 4.07, "p50": 4.10, "p60": 4.14, "p70": 4.17,
                "p80": 4.20, "p90": 4.25, "max_value": 4.50
            },
            {
                "position": "CB",
                "measurement": "arm_length",
                "sample_size": 350,
                "min_value": 29.0, "p10": 30.0, "p20": 30.5, "p30": 31.0,
                "p40": 31.5, "p50": 31.5, "p60": 32.0, "p70": 32.5,
                "p80": 33.0, "p90": 33.5, "max_value": 35.0
            },
            {
                "position": "CB",
                "measurement": "hand_size",
                "sample_size": 350,
                "min_value": 8.0, "p10": 8.75, "p20": 8.88, "p30": 9.0,
                "p40": 9.13, "p50": 9.25, "p60": 9.38, "p70": 9.50,
                "p80": 9.63, "p90": 9.75, "max_value": 10.5
            },
            {
                "position": "CB",
                "measurement": "wingspan",
                "sample_size": 350,
                "min_value": 72.0, "p10": 74.0, "p20": 74.5, "p30": 75.0,
                "p40": 75.5, "p50": 76.0, "p60": 76.5, "p70": 77.0,
                "p80": 77.5, "p90": 78.5, "max_value": 82.0
            },
            {
                "position": "CB",
                "measurement": "ten_yard_split",
                "sample_size": 250,
                "min_value": 1.42, "p10": 1.47, "p20": 1.49, "p30": 1.50,
                "p40": 1.51, "p50": 1.52, "p60": 1.54, "p70": 1.55,
                "p80": 1.56, "p90": 1.58, "max_value": 1.70
            },
            {
                "position": "CB",
                "measurement": "twenty_yard_split",
                "sample_size": 250,
                "min_value": 2.40, "p10": 2.47, "p20": 2.50, "p30": 2.52,
                "p40": 2.54, "p50": 2.56, "p60": 2.58, "p70": 2.60,
                "p80": 2.63, "p90": 2.66, "max_value": 2.80
            },
            {
                "position": "CB",
                "measurement": "height",
                "sample_size": 350,
                "min_value": 67.0, "p10": 69.0, "p20": 70.0, "p30": 70.5,
                "p40": 71.0, "p50": 71.5, "p60": 72.0, "p70": 72.5,
                "p80": 73.0, "p90": 74.0, "max_value": 77.0
            },
            {
                "position": "CB",
                "measurement": "weight",
                "sample_size": 350,
                "min_value": 165.0, "p10": 175.0, "p20": 180.0, "p30": 183.0,
                "p40": 185.0, "p50": 188.0, "p60": 191.0, "p70": 194.0,
                "p80": 198.0, "p90": 202.0, "max_value": 215.0
            }
        ]
    });

    let resp = client
        .post(format!("{}/api/v1/admin/seed-percentiles", base_url))
        .header("X-Seed-Api-Key", "test-key")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_get_ras_score_for_player() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    seed_percentiles(&client, &base_url).await;
    let player_id = create_player(&client, &base_url).await;
    create_combine_results(&client, &base_url, player_id).await;

    let resp = client
        .get(format!("{}/api/v1/players/{}/ras", base_url, player_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let ras: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ras["player_id"], player_id.to_string());

    // Should have an overall score (we provided all measurements)
    assert!(ras["overall_score"].is_number(), "Expected overall_score to be a number");
    let overall = ras["overall_score"].as_f64().unwrap();
    assert!(overall >= 0.0 && overall <= 10.0, "RAS should be 0-10, got {}", overall);

    // Should have category scores
    assert!(ras["size_score"].is_number());
    assert!(ras["speed_score"].is_number());
    assert!(ras["explosion_score"].is_number());
    assert!(ras["agility_score"].is_number());

    // Should have individual scores
    let individual = ras["individual_scores"].as_array().unwrap();
    assert!(individual.len() >= 6, "Should have at least 6 individual scores");
}

#[tokio::test]
async fn test_ras_score_requires_minimum_measurements() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    seed_percentiles(&client, &base_url).await;
    let player_id = create_player(&client, &base_url).await;

    // Create combine results with very few measurements
    let resp = client
        .post(format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.38
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let resp = client
        .get(format!("{}/api/v1/players/{}/ras", base_url, player_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let ras: serde_json::Value = resp.json().await.unwrap();
    // Should have null overall score due to insufficient measurements
    assert!(ras["overall_score"].is_null(), "Expected null overall score with few measurements");
    assert!(ras["explanation"].is_string(), "Should have explanation");
}

#[tokio::test]
async fn test_ras_score_includes_category_breakdown() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    seed_percentiles(&client, &base_url).await;
    let player_id = create_player(&client, &base_url).await;
    create_combine_results(&client, &base_url, player_id).await;

    let resp = client
        .get(format!("{}/api/v1/players/{}/ras", base_url, player_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let ras: serde_json::Value = resp.json().await.unwrap();

    // Check all category scores exist
    for category in &["size_score", "speed_score", "strength_score", "explosion_score", "agility_score"] {
        assert!(
            ras[category].is_number(),
            "Expected {} to be a number, got {:?}",
            category,
            ras[category]
        );
        let score = ras[category].as_f64().unwrap();
        assert!(score >= 0.0 && score <= 10.0, "{} should be 0-10, got {}", category, score);
    }
}

#[tokio::test]
async fn test_ras_elite_athlete() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    seed_percentiles(&client, &base_url).await;

    // Create a player with elite measurables
    let resp = client
        .post(format!("{}/api/v1/players", base_url))
        .json(&json!({
            "first_name": "Elite",
            "last_name": "Prospect",
            "position": "CB",
            "draft_year": 2026,
            "height_inches": 74,
            "weight_pounds": 202
        }))
        .send()
        .await
        .unwrap();
    let player: serde_json::Value = resp.json().await.unwrap();
    let player_id = uuid::Uuid::parse_str(player["id"].as_str().unwrap()).unwrap();

    // Elite combine numbers (all near or beyond p90)
    client
        .post(format!("{}/api/v1/combine-results", base_url))
        .json(&json!({
            "player_id": player_id,
            "year": 2026,
            "forty_yard_dash": 4.30,
            "bench_press": 22,
            "vertical_jump": 43.0,
            "broad_jump": 132,
            "three_cone_drill": 6.50,
            "twenty_yard_shuttle": 3.93,
            "arm_length": 34.0,
            "hand_size": 9.75,
            "wingspan": 79.0,
            "ten_yard_split": 1.45,
            "twenty_yard_split": 2.44
        }))
        .send()
        .await
        .unwrap();

    let resp = client
        .get(format!("{}/api/v1/players/{}/ras", base_url, player_id))
        .send()
        .await
        .unwrap();
    let ras: serde_json::Value = resp.json().await.unwrap();

    let overall = ras["overall_score"].as_f64().unwrap();
    assert!(
        overall >= 7.0,
        "Elite athlete should score 7.0+, got {}",
        overall
    );
}

#[tokio::test]
async fn test_ras_player_not_found() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let fake_id = uuid::Uuid::new_v4();
    let resp = client
        .get(format!("{}/api/v1/players/{}/ras", base_url, fake_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
