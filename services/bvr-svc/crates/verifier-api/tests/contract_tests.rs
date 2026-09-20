//! Contract tests for verifier-api (registration lookup, station detail,
//! result summary, USSD/WhatsApp verification).
//!
//! Pins status codes + response shapes. Lookup/ussd/whatsapp handlers
//! currently return static shapes pending register integration (M1); these
//! tests lock the contract so that wiring cannot silently change it.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use verifier_api::http::router;

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .uri(uri)
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    // Rejection bodies (4xx from extractors) are plain text, not JSON.
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn get(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .uri(uri)
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn registration_lookup_shape() {
    let (status, body) = post_json(
        router(),
        "/api/v1/verifier/lookups",
        json!({ "national_id_hash": "abc" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.get("registered").is_some());
}

#[tokio::test]
async fn station_detail_echoes_code() {
    let (status, body) = get(router(), "/api/v1/verifier/station/KE-NBI-001").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["station_code"], "KE-NBI-001");
}

#[tokio::test]
async fn result_summary_shape() {
    let (status, body) = get(router(), "/api/v1/verifier/results/summary").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.get("results").is_some());
}

#[tokio::test]
async fn ussd_session_ends() {
    let (status, body) = post_json(
        router(),
        "/api/v1/verifier/ussd/session",
        json!({ "session_id": "s1", "text": "1" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["action"], "end");
}

#[tokio::test]
async fn whatsapp_webhook_acknowledges() {
    let (status, body) = post_json(
        router(),
        "/api/v1/verifier/whatsapp/webhook",
        json!({ "from": "254700000000" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["acknowledged"], true);
}
