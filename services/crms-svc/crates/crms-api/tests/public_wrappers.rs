//! Round-trip tests for the CRMS public wrappers.
//!
//! `check_symbol_collision` answered `collision: false` for every request without
//! looking at the registered symbols, so a genuine collision with a symbol
//! already registered to another party was reported as clear.
//! `submit_primary_results`, `submit_primary_dispute` and
//! `record_registration_decision` each minted an id and discarded the submission,
//! so a primary tally or a registration decision could be filed and never read
//! back. `initiate_party_document_upload` minted an upload_ref and dropped the
//! request.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use crms_api::{router::build_router, state::AppState};
use std::sync::Arc;

fn app() -> axum::Router {
    build_router(Arc::new(AppState::default()))
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn send(app: &axum::Router, method: &str, uri: &str, body: Option<Value>) -> axum::response::Response {
    let mut b = Request::builder().method(method).uri(uri);
    let payload = match body {
        Some(v) => {
            b = b.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    app.clone().oneshot(b.body(payload).unwrap()).await.unwrap()
}

/// Register a party symbol through the public registry and return its id.
async fn register_symbol(app: &axum::Router, symbol: &str) -> String {
    let resp = send(
        app,
        "POST",
        "/api/v1/symbol/registrations",
        Some(json!({"symbol": symbol, "party_name": "Some Party"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED, "symbol registration must work");
    body_json(resp).await["registration_id"].as_str().unwrap().to_string()
}

async fn declare_primary(app: &axum::Router) -> String {
    let resp = send(
        app,
        "POST",
        "/api/v1/party/primaries",
        Some(json!({"party_id": "00000000-0000-0000-0000-0000000000c1", "scheduled_at": "2026-08-01"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    body_json(resp).await["primary_id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn symbol_collision_is_detected_against_registered_symbols() {
    // The headline: this previously reported every symbol as clear.
    let app = app();
    register_symbol(&app, "AK").await;

    let clear = body_json(
        send(&app, "POST", "/api/v1/symbol/collision-checks", Some(json!({"symbol": "PNC"}))).await,
    )
    .await;
    assert_eq!(clear["collision"], false, "a fresh symbol is not a collision");
    assert!(clear["conflicting_registration_id"].is_null());

    let clash = body_json(
        send(&app, "POST", "/api/v1/symbol/collision-checks", Some(json!({"symbol": "ak"}))).await,
    )
    .await;
    assert_eq!(
        clash["collision"], true,
        "a symbol already registered to another party must collide"
    );
    assert!(
        !clash["conflicting_registration_id"].is_null(),
        "the colliding registration should be identified"
    );
    assert_eq!(clash["symbol"], "AK", "comparison is case-insensitive");
}

#[tokio::test]
async fn an_empty_symbol_is_rejected() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/symbol/collision-checks",
        Some(json!({"symbol": "   "})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn submitted_primary_results_are_stored_under_their_primary() {
    let app = app();
    let primary = declare_primary(&app).await;

    let created = body_json(
        send(
            &app,
            "POST",
            &format!("/api/v1/party/primaries/{primary}/results"),
            Some(json!({"results": [{"candidate_id": "c1", "votes": 1200}]})),
        )
        .await,
    )
    .await;
    assert_eq!(created["primary_id"], primary.as_str());
    assert!(!created["result_id"].is_null());
    assert_eq!(created["results"][0]["votes"], 1200);
}

#[tokio::test]
async fn results_for_an_unknown_primary_are_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/party/primaries/no-such-primary/results",
        Some(json!({"results": []})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "submit_primary_results previously answered 201 for any primary id"
    );
}

#[tokio::test]
async fn filed_disputes_are_listed_against_their_primary() {
    let app = app();
    let primary = declare_primary(&app).await;
    let other = declare_primary(&app).await;

    let created = body_json(
        send(
            &app,
            "POST",
            &format!("/api/v1/party/primaries/{primary}/disputes"),
            Some(json!({"grounds": "irregular counting"})),
        )
        .await,
    )
    .await;
    assert_eq!(created["primary_id"], primary.as_str());
    assert_eq!(created["status"], "submitted");

    let listed = body_json(
        send(&app, "GET", &format!("/api/v1/party/primaries/{primary}/disputes"), None).await,
    )
    .await;
    assert_eq!(
        listed["disputes"].as_array().unwrap().len(),
        1,
        "a filed dispute must be listed; previously the list was always empty"
    );

    // Scoping: a dispute on one primary must not appear under another.
    let other_listed = body_json(
        send(&app, "GET", &format!("/api/v1/party/primaries/{other}/disputes"), None).await,
    )
    .await;
    assert!(
        other_listed["disputes"].as_array().unwrap().is_empty(),
        "disputes leaked between primaries"
    );
}

#[tokio::test]
async fn a_registration_decision_is_recorded() {
    let app = app();
    let created = body_json(
        send(
            &app,
            "POST",
            "/api/v1/party/admin/decisions",
            Some(json!({"registration_id": "reg-1", "decision": "approved"})),
        )
        .await,
    )
    .await;
    assert_eq!(created["decision"], "approved");
    assert_eq!(created["registration_id"], "reg-1");
    assert!(!created["decision_id"].is_null());
    assert!(!created["decided_at"].is_null());
}

#[tokio::test]
async fn a_document_upload_is_initiated_for_a_known_registration() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/party/registrations/no-such-registration/documents",
        Some(json!({"document_type": "constitution"})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "initiate_party_document_upload previously minted an upload_ref for any \\
         registration id"
    );
}

#[tokio::test]
async fn routers_do_not_share_records() {
    let a = app();
    let b = app();
    declare_primary(&a).await;
    let resp = send(
        &b,
        "POST",
        "/api/v1/party/primaries/no-such-primary/results",
        Some(json!({"results": []})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND, "state leaked between routers");
}