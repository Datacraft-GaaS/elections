// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Contract tests for the channel-gateway webhooks (USSD/IVR/WhatsApp).
//!
//! Pins status codes and response shapes.
//!
//! **These previously pinned the fabricated behaviour.** `ussd_session_closes_with_end`
//! asserted `action == "end"` for every input, because the handler answered `END`
//! to everything. `ussd_session_log_is_empty_list` asserted `log == []`, which is
//! what the handler returned for any session id. `whatsapp_verify_challenge`
//! asserted `200 OK` from the verification endpoints, which returned
//! `{"challenge": "verified"}` without reading a signature. `whatsapp_inbound_is_acknowledged`
//! asserted that an **unsigned** body was acknowledged.
//!
//! The contract they pin is now the real one. An unsigned or unverifiable webhook
//! is refused, which is the whole point of a verification endpoint.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use channel_gateway::http::{router, sign};
use channel_gateway::store::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

const SECRET: &str = "test-app-secret";

fn app() -> axum::Router {
    router(Arc::new(AppState::new()))
}

fn secured() -> axum::Router {
    router(Arc::new(AppState::new().with_whatsapp_secret(SECRET)))
}

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    post_raw(app, uri, &body.to_string(), None).await
}

async fn post_raw(
    app: axum::Router,
    uri: &str,
    body: &str,
    signature: Option<&str>,
) -> (StatusCode, Value) {
    let mut b = Request::builder()
        .uri(uri)
        .method("POST")
        .header("content-type", "application/json");
    if let Some(sig) = signature {
        b = b.header("x-hub-signature-256", sig);
    }
    let req = b.body(Body::from(body.to_string())).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    // Rejection bodies (4xx from extractors) are plain text, not JSON.
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn get(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .uri(uri)
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn ussd_body() -> Value {
    json!({
        "session_id": "sess-001",
        "msisdn_hash": "abc123",
        "text": "",
        "service_code": "*567#",
        "network_code": "63902"
    })
}

#[tokio::test]
async fn healthz_returns_ok() {
    let (status, body) = get(app(), "/healthz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn ussd_session_opens_with_a_menu() {
    // Previously answered "END" for every input, disconnecting the caller on the
    // first menu.
    let (status, body) = post_json(app(), "/ussd/sessions", ussd_body()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["action"], "continue");
    assert!(
        body["response_text"]
            .as_str()
            .unwrap()
            .contains("Check my registration"),
        "the caller should see a menu, got {:?}",
        body["response_text"]
    );
}

#[tokio::test]
async fn ussd_session_ends_when_the_caller_cancels() {
    // "END" is the correct response to a cancel, and only to a cancel.
    let mut body = ussd_body();
    body["text"] = json!("0");
    let (status, resp) = post_json(app(), "/ussd/sessions", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["action"], "end");
}

#[tokio::test]
async fn ussd_session_requires_body() {
    // Missing required fields -> axum Json rejection (422).
    let (status, _) = post_json(app(), "/ussd/sessions", json!({})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn ussd_session_rejects_an_unoperated_service_code() {
    // The old contract test used *384#, which this gateway does not operate. A
    // gateway that answers every code accepts traffic for services it does not run.
    let mut body = ussd_body();
    body["service_code"] = json!("*384#");
    let (status, resp) = post_json(app(), "/ussd/sessions", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(resp["detail"].as_str().unwrap().contains("*384#"));
}

#[tokio::test]
async fn ussd_session_log_holds_the_whole_exchange() {
    // Previously `log == []` for any session id, so a caller's exchange was
    // unreadable.
    let app = app();
    let (mut status, _) = post_json(app.clone(), "/ussd/sessions", ussd_body()).await;
    assert_eq!(status, StatusCode::OK);
    let mut next = ussd_body();
    next["text"] = json!("2");
    let (_, _) = post_json(app.clone(), "/ussd/sessions", next).await;
    status = StatusCode::OK;
    assert_eq!(status, StatusCode::OK);

    let (status, log) = get(app, "/ussd/sessions/sess-001").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        log["log"].as_array().unwrap().len(),
        2,
        "both steps of the exchange should be recorded"
    );
    assert_eq!(log["service_code"], "*567#");
    assert_eq!(log["state"], "awaiting_constituency");
}

#[tokio::test]
async fn an_unknown_ussd_session_is_404() {
    let (status, _) = get(app(), "/ussd/sessions/sess-does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn ivr_voice_call_advances_the_menu() {
    let (status, body) = post_json(
        app(),
        "/ivr/voice",
        json!({ "call_id": "call-1", "caller_id_hash": "h", "dtmf": "1" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["call_id"], "call-1");
    assert_eq!(body["state"], "awaiting_station");
    assert!(body["response_text"]
        .as_str()
        .unwrap()
        .contains("station code"));
}

#[tokio::test]
async fn whatsapp_verify_refuses_when_no_secret_is_configured() {
    // The old contract asserted 200 OK here, from a handler that returned
    // `{"challenge": "verified"}` for any request at all.
    for uri in ["/webhook/whatsapp", "/whatsapp/webhook"] {
        let (status, body) = get(app(), uri).await;
        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "{uri} must not report a check it could not perform"
        );
        assert_eq!(body["challenge"], "unverifiable");
    }
}

#[tokio::test]
async fn whatsapp_verify_rejects_a_missing_signature() {
    for uri in ["/webhook/whatsapp", "/whatsapp/webhook"] {
        let (status, _) = get(secured(), uri).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{uri}");
    }
}

#[tokio::test]
async fn whatsapp_verify_accepts_a_valid_signature() {
    let sig = sign(SECRET, b"");
    for uri in ["/webhook/whatsapp", "/whatsapp/webhook"] {
        let req = Request::builder()
            .uri(uri)
            .method("GET")
            .header("x-hub-signature-256", sig.clone())
            .body(Body::empty())
            .unwrap();
        let resp = secured().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{uri}");
    }
}

/// A WhatsApp Cloud API payload.
fn whatsapp_payload() -> String {
    json!({
        "object": "whatsapp_business_account",
        "entry": [{
            "id": "WABA_ID",
            "changes": [{
                "value": {
                    "messages": [{
                        "id": "wamid.ABC",
                        "from": "254700000000",
                        "text": { "body": "where is my station" }
                    }]
                },
                "field": "messages"
            }]
        }]
    })
    .to_string()
}

#[tokio::test]
async fn whatsapp_inbound_is_refused_without_a_signature() {
    // The old contract asserted that an **unsigned** body was acknowledged.
    let (status, _) = post_json(
        secured(),
        "/webhook/whatsapp",
        json!({ "from": "254700000000" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn whatsapp_inbound_is_refused_when_no_secret_is_configured() {
    let (status, _) = post_json(
        app(),
        "/webhook/whatsapp",
        json!({ "from": "254700000000" }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "recording an unauthenticated message would accept unauthenticated traffic"
    );
}

#[tokio::test]
async fn whatsapp_inbound_is_acknowledged_when_signed() {
    let payload = whatsapp_payload();
    let sig = sign(SECRET, payload.as_bytes());
    let (status, body) = post_raw(secured(), "/webhook/whatsapp", &payload, Some(&sig)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["acknowledged"], true);
    assert_eq!(body["message_id"], "wamid.ABC");
    // The sender's number is hashed rather than stored.
    assert!(body["from_hash"].as_str().unwrap().starts_with("sha256:"));
    assert_ne!(body["from_hash"], "254700000000");
}

#[tokio::test]
async fn a_signature_over_a_different_body_is_rejected() {
    let sig = sign(SECRET, br#"{"different":true}"#);
    let (status, _) = post_raw(
        secured(),
        "/webhook/whatsapp",
        &whatsapp_payload(),
        Some(&sig),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn both_webhook_paths_behave_identically() {
    // Two route paths served the same handler with the same fabrication. They
    // must not diverge now either.
    let payload = whatsapp_payload();
    let sig = sign(SECRET, payload.as_bytes());
    for uri in ["/webhook/whatsapp", "/whatsapp/webhook"] {
        let (status, _) = post_raw(secured(), uri, &payload, Some(&sig)).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        let (bad, _) = post_raw(secured(), uri, &payload, Some("00")).await;
        assert_eq!(bad, StatusCode::UNAUTHORIZED, "{uri}");
    }
}
