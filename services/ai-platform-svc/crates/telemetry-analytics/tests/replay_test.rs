//! A replay is not accepted when it will not be run.
//!
//! `replay` logged the detector id and the window and answered 202, under the
//! comment "In production this triggers the backtest harness asynchronously. For now
//! we acknowledge the request and return accepted." So a caller was left holding a
//! 202 for a result that did not exist.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_ai_platform_telemetry_analytics::api::{router, ApiState};
use eel_ai_platform_telemetry_analytics::rule_registry::RuleRegistry;
use std::sync::Arc;

/// A registry over a real rules file, so the listing assertion has something to
/// compare against.
fn app() -> (axum::Router, Arc<RuleRegistry>) {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../config/telemetry-rules.yaml"
    );
    let registry = Arc::new(RuleRegistry::load(path).expect("load rules fixture"));
    (router(ApiState { registry: registry.clone() }), registry)
}

async fn post(router: axum::Router, body: Value) -> (StatusCode, Value) {
    let resp = router
        .oneshot(
            Request::builder()
                .uri("/v1/analytics/replay")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

fn valid_window() -> Value {
    json!({
        "detector_id": "det-001",
        "window_start": "2027-08-21T00:00:00Z",
        "window_end": "2027-08-21T06:00:00Z"
    })
}

#[tokio::test]
async fn a_replay_is_refused_rather_than_accepted_and_dropped() {
    let (router, _state) = app();
    let (status, body) = post(router, valid_window()).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert!(body["error"].as_str().unwrap().contains("backtest harness"));
    assert!(body["error"].as_str().unwrap().contains("202 Accepted"));
}

#[tokio::test]
async fn an_unreplayable_window_is_a_client_error_not_a_blanket_refusal() {
    // An unparseable window cannot be replayed, and saying so is more useful than
    // the blanket refusal.
    let (router, _state) = app();
    let mut body = valid_window();
    body["window_start"] = json!("yesterday");
    let (status, _) = post(router, body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (router, _state) = app();
    let mut body = valid_window();
    body["detector_id"] = json!("   ");
    let (status, _) = post(router, body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn the_rule_listing_is_a_real_registry_read() {
    // `list_rules` was flagged because its state is bound to `s`, so the
    // `state.`/`store.` markers missed it. It reads the registry.
    let (router, state) = app();
    let resp = router
        .oneshot(
            Request::builder()
                .uri("/v1/analytics/rules")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let listed: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        listed.as_array().map(|a| a.len()),
        Some(state.all().len()),
        "the listing must be the registry, not a constant"
    );
    assert!(!listed.as_array().unwrap().is_empty(), "the fixture has rules");
}
