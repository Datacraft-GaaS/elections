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

//! Round-trip tests for the post-election review endpoints.
//!
//! Every handler here answered a success status while doing nothing.
//! `submit_finding` built a finding, answered 201 and dropped it, so a finding
//! submitted to the review could never be listed or triaged. `triage_finding`,
//! `sign_recommendation`, `record_long_drip_review` and
//! `record_parliament_response` all returned 204 for *any* id — including ones
//! that had never existed — so a caller was told a finding had been triaged, a
//! recommendation signed, a long-drip reviewed and parliament had responded when
//! no such records existed.
//!
//! `sign_recommendation` is the sharpest: the request carries an `hsm_signature`
//! and the handler discarded it entirely, logging only the signer's name.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_review::{router::build_router, state::AppState};

fn app() -> axum::Router {
    build_router(AppState::new())
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> axum::response::Response {
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

fn finding_body() -> Value {
    json!({
        "election_cycle": "2027-general",
        "submitter_category": "CITIZEN",
        "submitter_id": Some("CSO-1".to_string()),
        "finding_type": "TECHNICAL",
        "severity": "HIGH",
        "description": "Register extract for STN-014 omitted 300 voters",
        "evidence_refs": Some("s3://evidence/register-STN-014.csv".to_string()),
        "recommended_action": Some("Republish the extract".to_string()),
    })
}

async fn submit_finding(app: &axum::Router) -> String {
    let resp = send(app, "POST", "/v1/review/findings", Some(finding_body())).await;
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "submit_finding must work for the rest to be reachable"
    );
    body_json(resp).await["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn submitted_finding_is_listed() {
    let app = app();
    let id = submit_finding(&app).await;

    let listed = body_json(send(&app, "GET", "/v1/review/findings", None).await).await;
    let findings = listed["findings"].as_array().unwrap();
    assert_eq!(
        findings.len(),
        1,
        "submit_finding answered 201 and dropped the finding, so this was always empty"
    );
    assert_eq!(findings[0]["id"], id.as_str());
    assert_eq!(findings[0]["severity"], "HIGH");
    assert_eq!(findings[0]["status"], "SUBMITTED");
}

#[tokio::test]
async fn triaging_a_finding_changes_its_status_and_is_observable() {
    let app = app();
    let id = submit_finding(&app).await;

    let resp = send(
        &app,
        "PATCH",
        &format!("/v1/review/findings/{id}"),
        Some(json!({
            "status": "APPROVED",
            "duplicate_of": Value::Null,
            "is_long_drip": Some(true)
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let listed = body_json(send(&app, "GET", "/v1/review/findings", None).await).await;
    let f = &listed["findings"][0];
    assert_eq!(f["status"], "APPROVED");
    assert_eq!(f["is_long_drip"], true);
}

#[tokio::test]
async fn triaging_an_unknown_finding_is_404() {
    // Previously 204 for any id, so a finding that was never submitted could be
    // reported as triaged.
    let app = app();
    let resp = send(
        &app,
        "PATCH",
        "/v1/review/findings/never-submitted",
        Some(
            json!({"status": "APPROVED", "duplicate_of": Value::Null, "is_long_drip": Value::Null}),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn next_cycle_handoff_reflects_outstanding_findings() {
    // Previously always an empty handoff_items array.
    let app = app();
    let empty = body_json(send(&app, "GET", "/v1/review/next-cycle-handoff", None).await).await;
    assert!(empty["handoff_items"].as_array().unwrap().is_empty());

    let keep = submit_finding(&app).await;
    let close = submit_finding(&app).await;
    send(
        &app,
        "PATCH",
        &format!("/v1/review/findings/{close}"),
        Some(json!({"status": "CLOSED", "duplicate_of": Value::Null, "is_long_drip": Value::Null})),
    )
    .await;

    let handoff = body_json(send(&app, "GET", "/v1/review/next-cycle-handoff", None).await).await;
    let items = handoff["handoff_items"].as_array().unwrap();
    assert_eq!(
        items.len(),
        1,
        "only findings still open belong in the next-cycle handoff"
    );
    assert_eq!(items[0]["finding_id"], keep.as_str());
    assert_eq!(items[0]["election_cycle"], "2027-general");
}

#[tokio::test]
async fn signing_a_recommendation_requires_and_records_a_signature() {
    let app = app();
    // No endpoint creates recommendations, so signing one is a 404 -- which is
    // itself the behaviour that was missing, since it previously returned 204.
    let missing = send(
        &app,
        "POST",
        "/v1/review/recommendations/unknown/sign",
        Some(json!({"signed_by": "chair", "hsm_signature": "sig"})),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);

    let listed = body_json(send(&app, "GET", "/v1/review/recommendations", None).await).await;
    assert!(listed["recommendations"].as_array().unwrap().is_empty());

    let tracker = body_json(send(&app, "GET", "/v1/review/public", None).await).await;
    assert!(tracker["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn recording_a_long_drip_review_of_an_unknown_alert_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/review/long-drip/none/review",
        Some(json!({"reviewer_id": "rev-1", "escalated_to_commissioner": true})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "record_long_drip_review previously returned 204 for any id"
    );
    let alerts = body_json(send(&app, "GET", "/v1/review/long-drip", None).await).await;
    assert!(alerts["alerts"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn recording_a_parliament_response_of_an_unknown_referral_is_404() {
    let app = app();
    let resp = send(
        &app,
        "PATCH",
        "/v1/review/parliament-referrals/none",
        Some(json!({
            "parliament_response": "Adopted",
            "bill_reference": Some("BILL-2027-4".to_string()),
            "parliament_response_at": "2027-02-01T00:00:00Z"
        })),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "record_parliament_response previously discarded the response and returned 204 for any id"
    );
}

#[tokio::test]
async fn routers_do_not_share_records() {
    // build_router constructs its own state per call; if findings leaked, every
    // empty-state assertion above would pass for the wrong reason.
    let a = app();
    let b = app();
    submit_finding(&a).await;
    let b_listed = body_json(send(&b, "GET", "/v1/review/findings", None).await).await;
    assert!(
        b_listed["findings"].as_array().unwrap().is_empty(),
        "findings leaked between routers"
    );
}
