//! Contract tests for the channel-gateway webhooks (USSD/IVR/WhatsApp).
//!
//! Pins status codes + response shapes. Several handlers currently return
//! static gateway responses (documented inline); these tests lock the
//! contract so backend wiring (M1) cannot silently change the shape.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use channel_gateway::http::router;
use serde_json::{json, Value};
use tower::ServiceExt;

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

fn ussd_body() -> Value {
    json!({
        "session_id": "sess-001",
        "msisdn_hash": "abc123",
        "text": "1",
        "service_code": "*384#",
        "network_code": "63902"
    })
}

#[tokio::test]
async fn healthz_returns_ok() {
    let (status, body) = get(router(), "/healthz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn ussd_session_closes_with_end() {
    let (status, body) = post_json(router(), "/ussd/sessions", ussd_body()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["action"], "end");
    assert_eq!(body["session_id"], "sess-001");
}

#[tokio::test]
async fn ussd_session_requires_body() {
    // Missing required fields -> axum Json rejection (422).
    let (status, _) = post_json(router(), "/ussd/sessions", json!({})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn ussd_session_log_is_empty_list() {
    let (status, body) = get(router(), "/ussd/sessions/sess-001").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["session_id"], "sess-001");
    assert_eq!(body["log"], json!([]));
}

#[tokio::test]
async fn ivr_voice_call_ends() {
    let (status, body) = post_json(
        router(),
        "/ivr/voice",
        json!({ "call_id": "call-1", "caller_id_hash": "h", "dtmf": "1" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["call_id"], "call-1");
}

#[tokio::test]
async fn whatsapp_verify_challenge() {
    let (status, _) = get(router(), "/webhook/whatsapp").await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = get(router(), "/whatsapp/webhook").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn whatsapp_inbound_is_acknowledged() {
    let (status, body) = post_json(
        router(),
        "/webhook/whatsapp",
        json!({ "from": "254700000000", "text": "hi" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["acknowledged"], true);
}
