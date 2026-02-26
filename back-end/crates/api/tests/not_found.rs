//! 404 Not Found acceptance tests for individual resource GET endpoints

mod common;

use std::time::Duration;

const FAKE_UUID: &str = "00000000-0000-0000-0000-000000000000";

#[tokio::test]
async fn test_get_nonexistent_draft_returns_404() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!("{}/api/v1/drafts/{}", base_url, FAKE_UUID))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_player_returns_404() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!("{}/api/v1/players/{}", base_url, FAKE_UUID))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_combine_results_returns_404() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!(
            "{}/api/v1/combine-results/{}",
            base_url, FAKE_UUID
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_scouting_report_returns_404() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!(
            "{}/api/v1/scouting-reports/{}",
            base_url, FAKE_UUID
        ))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_team_need_returns_404() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!("{}/api/v1/team-needs/{}", base_url, FAKE_UUID))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_trade_returns_404() {
    let (base_url, _pool) = common::spawn_app().await;
    let client = common::create_client();

    let response = client
        .get(&format!("{}/api/v1/trades/{}", base_url, FAKE_UUID))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}
