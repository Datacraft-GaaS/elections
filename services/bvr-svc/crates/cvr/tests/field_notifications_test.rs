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

//! Integration-level tests for the field-notifications route module.
//!
//! Auth, validation, and list tests are hermetic: no network or database is
//! required. The submit and reviewer-decision success paths write through the
//! transactional outbox and therefore need PostgreSQL; they are gated behind
//! the `db-tests` feature (run against the docker-compose stack with
//! `cargo test -p cvr --features db-tests -- --include-ignored`).
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::util::ServiceExt; // for `oneshot`

fn app() -> axum::Router {
    // Lazy pool: connections are only attempted when a handler touches the
    // database, so the hermetic tests below never need a live PostgreSQL.
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://localhost/cvr")
        .expect("lazy pool construction");
    cvr::app(pool)
}

fn base_capture_payload() -> Value {
    json!({
        "nationalId": "12345678",
        "modality": "DEATH_CERTIFICATE",
        "officerId": "018f9e5c-0000-7000-8000-000000000001",
        "kitId": "KIT-NRB-001",
        "capturedAt": "2026-08-22T08:00:00Z",
        "geoLat": -1.286389,
        "geoLon": 36.817223,
        "geoAccuracyM": 5.0,
        "clientCertSha256": "a".repeat(64),
        "ocrResult": {"deceasedName": "John Doe", "dateOfDeath": "2026-08-20"},
        "ocrConfidence": 0.93,
        "officerHsmSignature": "deadbeef0102030405060708",
        "offlineQueuedAt": null
    })
}

fn with_field_officer_headers(req: axum::http::request::Builder) -> axum::http::request::Builder {
    req.header("x-spiffe-svid", "spiffe://eel.internal/agent/field/KE-007")
        .header("x-eel-role", "COMMISSION_FIELD_OFFICER")
        .header("idempotency-key", "test-idem-001")
}

fn with_adjudicator_headers(req: axum::http::request::Builder) -> axum::http::request::Builder {
    req.header("x-spiffe-svid", "spiffe://eel.internal/adjudicator/KE-001")
        .header("x-eel-role", "COMMISSION_ADJUDICATOR")
        .header("idempotency-key", "test-adj-001")
}

// ---------------------------------------------------------------------------
// 1. SVID role rejection - wrong role returns 400
// ---------------------------------------------------------------------------

#[tokio::test]
async fn svid_role_rejection_wrong_role() {
    let app = app();
    let body = serde_json::to_vec(&base_capture_payload()).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/cvr/field-notifications")
        .header("content-type", "application/json")
        .header("x-spiffe-svid", "spiffe://eel.internal/agent/field/KE-007")
        .header("x-eel-role", "COMMISSION_OBSERVER") // wrong role
        .header("idempotency-key", "idem-role-test")
        .body(Body::from(body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// 2. SVID role rejection - missing SVID header returns 400
// ---------------------------------------------------------------------------

#[tokio::test]
async fn svid_role_rejection_missing_svid() {
    let app = app();
    let body = serde_json::to_vec(&base_capture_payload()).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/cvr/field-notifications")
        .header("content-type", "application/json")
        .header("x-eel-role", "COMMISSION_FIELD_OFFICER")
        .header("idempotency-key", "idem-no-svid")
        .body(Body::from(body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// 3. SHA-256 mismatch rejection - client_cert_sha256 wrong length returns 400
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sha256_mismatch_rejection_short_hash() {
    let app = app();
    let mut payload = base_capture_payload();
    payload["clientCertSha256"] = json!("tooshort");
    let body = serde_json::to_vec(&payload).unwrap();
    let req = with_field_officer_headers(
        Request::builder()
            .method("POST")
            .uri("/v1/cvr/field-notifications")
            .header("content-type", "application/json"),
    )
    .body(Body::from(body))
    .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// 4. Idempotent retry - missing Idempotency-Key returns 400
// ---------------------------------------------------------------------------

#[tokio::test]
async fn missing_idempotency_key_rejected() {
    let app = app();
    let body = serde_json::to_vec(&base_capture_payload()).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/cvr/field-notifications")
        .header("content-type", "application/json")
        .header("x-spiffe-svid", "spiffe://eel.internal/agent/field/KE-007")
        .header("x-eel-role", "COMMISSION_FIELD_OFFICER")
        // no idempotency-key header
        .body(Body::from(body))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// 5. Geo bounds validation - coordinates outside Kenya rejected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn geo_bounds_outside_kenya_rejected() {
    let app = app();
    let mut payload = base_capture_payload();
    payload["geoLat"] = json!(51.5); // London latitude
    payload["geoLon"] = json!(-0.12);
    payload["clientCertSha256"] = json!("a".repeat(64));
    let body = serde_json::to_vec(&payload).unwrap();
    let req = with_field_officer_headers(
        Request::builder()
            .method("POST")
            .uri("/v1/cvr/field-notifications")
            .header("content-type", "application/json"),
    )
    .body(Body::from(body))
    .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// 6. Geo bounds validation - valid Nairobi coordinates accepted
// ---------------------------------------------------------------------------

#[tokio::test]
#[cfg_attr(
    not(feature = "db-tests"),
    ignore = "requires PostgreSQL; run via docker-compose stack with --features db-tests"
)]
async fn geo_bounds_nairobi_accepted() {
    let app = app();
    let mut payload = base_capture_payload();
    payload["clientCertSha256"] = json!("a".repeat(64));
    payload["officerHsmSignature"] = json!("cafebabe01020304");
    let body = serde_json::to_vec(&payload).unwrap();
    let req = with_field_officer_headers(
        Request::builder()
            .method("POST")
            .uri("/v1/cvr/field-notifications")
            .header("content-type", "application/json"),
    )
    .body(Body::from(body))
    .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
}

// ---------------------------------------------------------------------------
// 7. Successful submit returns capture_id + pending verdict
// ---------------------------------------------------------------------------

#[tokio::test]
#[cfg_attr(
    not(feature = "db-tests"),
    ignore = "requires PostgreSQL; run via docker-compose stack with --features db-tests"
)]
async fn submit_returns_capture_id_and_pending() {
    let app = app();
    let mut payload = base_capture_payload();
    payload["clientCertSha256"] = json!("b".repeat(64));
    payload["officerHsmSignature"] = json!("sig0102030405060708");
    let body = serde_json::to_vec(&payload).unwrap();
    let req = with_field_officer_headers(
        Request::builder()
            .method("POST")
            .uri("/v1/cvr/field-notifications")
            .header("content-type", "application/json"),
    )
    .body(Body::from(body))
    .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(resp.into_body(), 65536).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("captureId").is_some(), "captureId missing");
    assert!(json.get("eventId").is_some(), "eventId missing");
    assert_eq!(json["reviewVerdict"], "pending");
}

// ---------------------------------------------------------------------------
// 8. Reviewer decision - adjudicator role required
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reviewer_decision_requires_adjudicator_role() {
    let app = app();
    let capture_id = uuid::Uuid::now_v7();
    let body = serde_json::to_vec(&json!({"verdict": "accepted", "reviewerNotes": null})).unwrap();
    // Use field-officer role (wrong for this endpoint)
    let req = with_field_officer_headers(
        Request::builder()
            .method("PATCH")
            .uri(format!(
                "/v1/cvr/field-notifications/{}/reviewer-decision",
                capture_id
            ))
            .header("content-type", "application/json"),
    )
    .body(Body::from(body))
    .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// 9. Verdict transition: accepted -> register_event_emitted = true
// ---------------------------------------------------------------------------

#[tokio::test]
#[cfg_attr(
    not(feature = "db-tests"),
    ignore = "requires PostgreSQL; run via docker-compose stack with --features db-tests"
)]
async fn reviewer_accepted_emits_register_event() {
    let app = app();
    let capture_id = uuid::Uuid::now_v7();
    let body = serde_json::to_vec(
        &json!({"verdict": "accepted", "reviewerNotes": "confirmed by register"}),
    )
    .unwrap();
    let req = with_adjudicator_headers(
        Request::builder()
            .method("PATCH")
            .uri(format!(
                "/v1/cvr/field-notifications/{}/reviewer-decision",
                capture_id
            ))
            .header("content-type", "application/json"),
    )
    .body(Body::from(body))
    .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), 65536).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["reviewVerdict"], "accepted");
    assert_eq!(json["registerEventEmitted"], true);
}

// ---------------------------------------------------------------------------
// 10. Verdict transition: rejected -> register_event_emitted = false
// ---------------------------------------------------------------------------

#[tokio::test]
#[cfg_attr(
    not(feature = "db-tests"),
    ignore = "requires PostgreSQL; run via docker-compose stack with --features db-tests"
)]
async fn reviewer_rejected_does_not_emit_register_event() {
    let app = app();
    let capture_id = uuid::Uuid::now_v7();
    let body =
        serde_json::to_vec(&json!({"verdict": "rejected", "reviewerNotes": "cert illegible"}))
            .unwrap();
    let req = with_adjudicator_headers(
        Request::builder()
            .method("PATCH")
            .uri(format!(
                "/v1/cvr/field-notifications/{}/reviewer-decision",
                capture_id
            ))
            .header("content-type", "application/json"),
    )
    .body(Body::from(body))
    .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), 65536).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["reviewVerdict"], "rejected");
    assert_eq!(json["registerEventEmitted"], false);
}

// ---------------------------------------------------------------------------
// 11. List endpoint returns empty captures for officer with no history
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_captures_returns_empty_for_unknown_officer() {
    let app = app();
    let req = with_field_officer_headers(
        Request::builder()
            .method("GET")
            .uri("/v1/cvr/field-notifications?officer_id=018f9e5c-0000-7000-8000-000000000099&status=pending"),
    )
    .body(Body::empty())
    .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), 65536).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["total"], 0);
    assert!(json["captures"].as_array().unwrap().is_empty());
}
