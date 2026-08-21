//! NIST beacon contract tests against the double.
//!
//! Set NIST_DOUBLE_URL env var to point at the double (default: http://localhost:8090).

use eel_adapter_nist_beacon_rs::{fetch_latest, PulseCache};

fn double_url() -> String {
    std::env::var("NIST_DOUBLE_URL").unwrap_or_else(|_| "http://localhost:8090".into())
}

#[tokio::test]
async fn fetch_latest_from_double() {
    let base = format!("{}/beacon/2.0", double_url());
    // Skip if double not running (CI without docker).
    if reqwest::Client::new()
        .get(format!("{}/health", double_url()))
        .send()
        .await
        .is_err()
    {
        return;
    }
    let pulse = fetch_latest(&base).await.expect("fetch_latest must succeed");
    assert_eq!(pulse.chain_index, 1);
    assert!(!pulse.output_value.is_empty());
}

#[test]
fn cache_holds_latest() {
    let cache = PulseCache::new(10);
    assert!(cache.latest().is_none());
}
