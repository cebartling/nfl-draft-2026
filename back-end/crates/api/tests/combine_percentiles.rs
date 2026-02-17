mod common;

use serde_json::json;

#[tokio::test]
async fn test_seed_and_get_percentiles() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    // Seed some percentile data
    let body = json!({
        "percentiles": [
            {
                "position": "QB",
                "measurement": "forty_yard_dash",
                "sample_size": 200,
                "min_value": 4.4,
                "p10": 4.55,
                "p20": 4.6,
                "p30": 4.65,
                "p40": 4.7,
                "p50": 4.75,
                "p60": 4.8,
                "p70": 4.85,
                "p80": 4.9,
                "p90": 5.0,
                "max_value": 5.3
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

    let seed_result: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(seed_result["upserted_count"], 1);
    assert_eq!(seed_result["error_count"], 0);

    // Get all percentiles
    let resp = client
        .get(format!("{}/api/v1/combine-percentiles", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let percentiles: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(percentiles.len(), 1);
    assert_eq!(percentiles[0]["position"], "QB");
    assert_eq!(percentiles[0]["measurement"], "forty_yard_dash");
    assert_eq!(percentiles[0]["sample_size"], 200);
    assert_eq!(percentiles[0]["p50"], 4.75);
}

#[tokio::test]
async fn test_get_percentiles_by_position() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    // Seed data for QB and WR
    let body = json!({
        "percentiles": [
            {
                "position": "QB",
                "measurement": "forty_yard_dash",
                "sample_size": 200,
                "min_value": 4.4,
                "p10": 4.55, "p20": 4.6, "p30": 4.65, "p40": 4.7,
                "p50": 4.75, "p60": 4.8, "p70": 4.85, "p80": 4.9,
                "p90": 5.0,
                "max_value": 5.3
            },
            {
                "position": "WR",
                "measurement": "forty_yard_dash",
                "sample_size": 300,
                "min_value": 4.2,
                "p10": 4.3, "p20": 4.35, "p30": 4.4, "p40": 4.42,
                "p50": 4.45, "p60": 4.5, "p70": 4.55, "p80": 4.6,
                "p90": 4.7,
                "max_value": 5.0
            },
            {
                "position": "QB",
                "measurement": "bench_press",
                "sample_size": 180,
                "min_value": 8.0,
                "p10": 12.0, "p20": 14.0, "p30": 16.0, "p40": 17.0,
                "p50": 19.0, "p60": 20.0, "p70": 22.0, "p80": 24.0,
                "p90": 26.0,
                "max_value": 35.0
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

    // Filter by QB
    let resp = client
        .get(format!(
            "{}/api/v1/combine-percentiles?position=QB",
            base_url
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let qb_percentiles: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(qb_percentiles.len(), 2);
    for p in &qb_percentiles {
        assert_eq!(p["position"], "QB");
    }

    // Filter by WR
    let resp = client
        .get(format!(
            "{}/api/v1/combine-percentiles?position=WR",
            base_url
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let wr_percentiles: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(wr_percentiles.len(), 1);
    assert_eq!(wr_percentiles[0]["position"], "WR");
}

#[tokio::test]
async fn test_get_all_percentiles() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    // Seed QB + WR data
    let body = json!({
        "percentiles": [
            {
                "position": "QB",
                "measurement": "forty_yard_dash",
                "sample_size": 200,
                "min_value": 4.4,
                "p10": 4.55, "p20": 4.6, "p30": 4.65, "p40": 4.7,
                "p50": 4.75, "p60": 4.8, "p70": 4.85, "p80": 4.9,
                "p90": 5.0,
                "max_value": 5.3
            },
            {
                "position": "WR",
                "measurement": "forty_yard_dash",
                "sample_size": 300,
                "min_value": 4.2,
                "p10": 4.3, "p20": 4.35, "p30": 4.4, "p40": 4.42,
                "p50": 4.45, "p60": 4.5, "p70": 4.55, "p80": 4.6,
                "p90": 4.7,
                "max_value": 5.0
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

    // Get all without filter
    let resp = client
        .get(format!("{}/api/v1/combine-percentiles", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let all: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_percentile_data_structure() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    let body = json!({
        "percentiles": [
            {
                "position": "RB",
                "measurement": "vertical_jump",
                "sample_size": 250,
                "min_value": 25.0,
                "p10": 28.0, "p20": 30.0, "p30": 31.5, "p40": 33.0,
                "p50": 34.5, "p60": 36.0, "p70": 37.5, "p80": 39.0,
                "p90": 41.0,
                "max_value": 46.0,
                "years_start": 2005,
                "years_end": 2024
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

    let resp = client
        .get(format!(
            "{}/api/v1/combine-percentiles?position=RB",
            base_url
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let percentiles: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(percentiles.len(), 1);

    let p = &percentiles[0];
    assert_eq!(p["position"], "RB");
    assert_eq!(p["measurement"], "vertical_jump");
    assert_eq!(p["sample_size"], 250);
    assert_eq!(p["min_value"], 25.0);
    assert_eq!(p["p10"], 28.0);
    assert_eq!(p["p20"], 30.0);
    assert_eq!(p["p30"], 31.5);
    assert_eq!(p["p40"], 33.0);
    assert_eq!(p["p50"], 34.5);
    assert_eq!(p["p60"], 36.0);
    assert_eq!(p["p70"], 37.5);
    assert_eq!(p["p80"], 39.0);
    assert_eq!(p["p90"], 41.0);
    assert_eq!(p["max_value"], 46.0);
    assert_eq!(p["years_start"], 2005);
    assert_eq!(p["years_end"], 2024);
    assert!(p["id"].is_string());
}

#[tokio::test]
async fn test_seed_percentiles_requires_auth() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    let body = json!({
        "percentiles": []
    });

    // No key
    let resp = client
        .post(format!("{}/api/v1/admin/seed-percentiles", base_url))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Wrong key
    let resp = client
        .post(format!("{}/api/v1/admin/seed-percentiles", base_url))
        .header("X-Seed-Api-Key", "wrong-key")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_seed_percentiles_upsert_overwrites() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    // Seed initial data
    let body = json!({
        "percentiles": [
            {
                "position": "QB",
                "measurement": "forty_yard_dash",
                "sample_size": 100,
                "min_value": 4.4,
                "p10": 4.55, "p20": 4.6, "p30": 4.65, "p40": 4.7,
                "p50": 4.75, "p60": 4.8, "p70": 4.85, "p80": 4.9,
                "p90": 5.0,
                "max_value": 5.3
            }
        ]
    });

    client
        .post(format!("{}/api/v1/admin/seed-percentiles", base_url))
        .header("X-Seed-Api-Key", "test-key")
        .json(&body)
        .send()
        .await
        .unwrap();

    // Upsert with new values
    let body2 = json!({
        "percentiles": [
            {
                "position": "QB",
                "measurement": "forty_yard_dash",
                "sample_size": 300,
                "min_value": 4.3,
                "p10": 4.45, "p20": 4.5, "p30": 4.55, "p40": 4.6,
                "p50": 4.65, "p60": 4.7, "p70": 4.75, "p80": 4.8,
                "p90": 4.9,
                "max_value": 5.2
            }
        ]
    });

    let resp = client
        .post(format!("{}/api/v1/admin/seed-percentiles", base_url))
        .header("X-Seed-Api-Key", "test-key")
        .json(&body2)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Verify the data was updated
    let resp = client
        .get(format!(
            "{}/api/v1/combine-percentiles?position=QB",
            base_url
        ))
        .send()
        .await
        .unwrap();
    let percentiles: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(percentiles.len(), 1);
    assert_eq!(percentiles[0]["sample_size"], 300);
    assert_eq!(percentiles[0]["p50"], 4.65);
}

#[tokio::test]
async fn test_seed_percentiles_invalid_data() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    let body = json!({
        "percentiles": [
            {
                "position": "INVALID",
                "measurement": "forty_yard_dash",
                "sample_size": 100,
                "min_value": 4.4,
                "p10": 4.55, "p20": 4.6, "p30": 4.65, "p40": 4.7,
                "p50": 4.75, "p60": 4.8, "p70": 4.85, "p80": 4.9,
                "p90": 5.0,
                "max_value": 5.3
            },
            {
                "position": "QB",
                "measurement": "invalid_measurement",
                "sample_size": 100,
                "min_value": 4.4,
                "p10": 4.55, "p20": 4.6, "p30": 4.65, "p40": 4.7,
                "p50": 4.75, "p60": 4.8, "p70": 4.85, "p80": 4.9,
                "p90": 5.0,
                "max_value": 5.3
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

    let result: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(result["upserted_count"], 0);
    assert_eq!(result["error_count"], 2);
}

#[tokio::test]
async fn test_delete_all_percentiles() {
    let (base_url, _pool) = common::spawn_app_with_seed_key("test-key").await;
    let client = common::create_client();

    // Seed data first
    let body = json!({
        "percentiles": [
            {
                "position": "QB",
                "measurement": "forty_yard_dash",
                "sample_size": 100,
                "min_value": 4.4,
                "p10": 4.55, "p20": 4.6, "p30": 4.65, "p40": 4.7,
                "p50": 4.75, "p60": 4.8, "p70": 4.85, "p80": 4.9,
                "p90": 5.0,
                "max_value": 5.3
            }
        ]
    });

    client
        .post(format!("{}/api/v1/admin/seed-percentiles", base_url))
        .header("X-Seed-Api-Key", "test-key")
        .json(&body)
        .send()
        .await
        .unwrap();

    // Delete all
    let resp = client
        .delete(format!("{}/api/v1/admin/percentiles", base_url))
        .header("X-Seed-Api-Key", "test-key")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Verify empty
    let resp = client
        .get(format!("{}/api/v1/combine-percentiles", base_url))
        .send()
        .await
        .unwrap();
    let all: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(all.is_empty());
}
