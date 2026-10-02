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

//! Round-trip tests for the `/v1/forensics/cycles` lifecycle.
//!
//! Every one of these endpoints previously answered from `vec![]`, returned
//! `202 Accepted` with no body, or minted a response it never stored:
//!
//! - `list_cycles` and `list_estimates` always returned an empty list, so a
//!   created cycle was invisible.
//! - `trigger_run` accepted any id -- including one that never existed --
//!   discarded the path id, and stored nothing.
//! - `publish_replication` published against a draft cycle, attesting to a
//!   result that had not been produced.
//! - `get_public_report` answered `200` with an empty `PublicReport` for every
//!   id, so a request for a report that was never published looked like a real
//!   but blank report.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_forensics::{router::build_router, state::AppState};
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
    app.clone()
        .oneshot(b.body(payload).unwrap())
        .await
        .unwrap()
}

async fn create_cycle(app: &axum::Router, method: &str) -> String {
    let resp = send(
        app,
        "POST",
        "/v1/forensics/cycles",
        Some(json!({"election_id": "elec-2027-general", "method": method})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    body_json(resp).await["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn a_created_cycle_is_listed() {
    // Previously list_cycles returned `{"cycles": []}` unconditionally, so a
    // created cycle could never be found.
    let app = app();
    let id = create_cycle(&app, "RD").await;

    let listed = body_json(send(&app, "GET", "/v1/forensics/cycles", None).await).await;
    let cycles = listed["cycles"].as_array().unwrap();
    assert_eq!(cycles.len(), 1, "a created cycle must be listed");
    assert_eq!(cycles[0]["id"], id.as_str());
    assert_eq!(cycles[0]["election_id"], "elec-2027-general");
    assert_eq!(cycles[0]["method"], "RD");
    assert_eq!(cycles[0]["status"], "draft");
}

#[tokio::test]
async fn a_cycle_requires_an_election_id() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/forensics/cycles",
        Some(json!({"election_id": "   "})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn starting_a_run_moves_the_stored_cycle_to_running() {
    // Previously trigger_run answered 202 with an empty body and left the cycle
    // untouched, so "running" was never observable anywhere.
    let app = app();
    let id = create_cycle(&app, "IV").await;

    let run = body_json(
        send(&app, "PUT", &format!("/v1/forensics/cycles/{id}/run"), None).await,
    )
    .await;
    assert_eq!(run["id"], id.as_str());
    assert_eq!(run["status"], "running");
    assert!(!run["started_at"].is_null(), "started_at must be stamped");

    // The change is against the stored record, not just the response.
    let listed = body_json(send(&app, "GET", "/v1/forensics/cycles", None).await).await;
    let stored = listed["cycles"].as_array().unwrap()[0].clone();
    assert_eq!(stored["status"], "running");
    assert!(!stored["started_at"].is_null());
}

#[tokio::test]
async fn the_run_records_why_the_estimator_cannot_proceed() {
    // The RD/IV/DiD estimators are in services/governance-svc/py/forensics-ml,
    // a separate Python distribution with no bridge in this workspace. The
    // honest outcome is a recorded reason, not an invented estimate.
    let app = app();
    let id = create_cycle(&app, "RD").await;
    let run = body_json(
        send(&app, "PUT", &format!("/v1/forensics/cycles/{id}/run"), None).await,
    )
    .await;
    let reason = run["blocked_reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("forensics-ml"),
        "the blocked reason should name the estimator that is absent, got: {reason:?}"
    );
    assert_eq!(run["completed_at"], Value::Null, "nothing has completed");
}

#[tokio::test]
async fn running_an_unknown_cycle_is_404() {
    // Previously any id returned 202, so a run could be "triggered" against a
    // cycle that did not exist.
    let app = app();
    let resp = send(&app, "PUT", "/v1/forensics/cycles/no-such-cycle/run", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_cycle_cannot_be_run_twice() {
    let app = app();
    let id = create_cycle(&app, "RD").await;
    assert_eq!(
        send(&app, "PUT", &format!("/v1/forensics/cycles/{id}/run"), None)
            .await
            .status(),
        StatusCode::ACCEPTED
    );
    let second = send(&app, "PUT", &format!("/v1/forensics/cycles/{id}/run"), None).await;
    assert_eq!(
        second.status(),
        StatusCode::CONFLICT,
        "a running cycle must not be re-run"
    );
    let detail = body_json(second).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("running"), "the reason should say the status: {detail}");
}

#[tokio::test]
async fn replication_requires_a_complete_cycle() {
    // This is the substantive one. Publishing a replication package attests that
    // an analysis was produced and can be reproduced; against a draft cycle the
    // old code happily returned 201.
    let app = app();
    let id = create_cycle(&app, "RD").await;
    let body = json!({
        "code_url": "https://example.invalid/repl",
        "data_hash": "sha256:abc",
        "environment_spec": "forensics-ml==0.1.0"
    });

    let resp = send(
        &app,
        "POST",
        &format!("/v1/forensics/cycles/{id}/replication"),
        Some(body.clone()),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "a draft cycle must not be able to publish a replication package"
    );
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("draft"), "the reason should name the status: {detail}");
}

#[tokio::test]
async fn replication_for_an_unknown_cycle_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/forensics/cycles/no-such-cycle/replication",
        Some(json!({
            "code_url": "https://example.invalid/repl",
            "data_hash": "sha256:abc",
            "environment_spec": "forensics-ml==0.1.0"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_unpublished_report_is_404_not_a_blank_one() {
    // Previously get_public_report returned 200 with an empty PublicReport for
    // any id, which reads as a real report with no content.
    let app = app();
    let resp = send(&app, "GET", "/v1/forensics/reports/report-1", None).await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "an unpublished report must not read as a blank one"
    );
}

#[tokio::test]
async fn estimates_for_an_unknown_cycle_are_404() {
    // Previously any cycle id returned 200 with an empty list, which is
    // indistinguishable from "this cycle has produced no estimates".
    let app = app();
    let resp = send(&app, "GET", "/v1/forensics/cycles/no-such-cycle/estimates", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn estimates_for_a_known_cycle_start_empty() {
    let app = app();
    let id = create_cycle(&app, "RD").await;
    let listed = body_json(
        send(&app, "GET", &format!("/v1/forensics/cycles/{id}/estimates"), None).await,
    )
    .await;
    assert!(
        listed["estimates"].as_array().unwrap().is_empty(),
        "no estimate has been produced"
    );
}

#[tokio::test]
async fn an_odpp_referral_uses_the_engine_convention() {
    let app = app();
    let id = create_cycle(&app, "RD").await;
    let packet = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/forensics/cycles/{id}/referrals"),
            Some(json!({
                "referred_to": "ODPP",
                "referral_basis": "statistical anomaly in turnout",
                "content_url": "https://example.invalid/packet"
            })),
        )
        .await,
    )
    .await;
    assert_eq!(packet["cycle_id"], id.as_str());
    assert_eq!(packet["referred_to"], "ODPP");
    assert_eq!(packet["referral_basis"], "statistical anomaly in turnout");
    assert_eq!(
        packet["content_url"], "https://example.invalid/packet",
        "the supplied content reference must win over the engine's fallback"
    );
    assert_eq!(packet["acknowledged_at"], Value::Null);
}

#[tokio::test]
async fn a_knbs_referral_keeps_its_target() {
    let app = app();
    let id = create_cycle(&app, "DIFF_IN_DIFF").await;
    let packet = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/forensics/cycles/{id}/referrals"),
            Some(json!({
                "referred_to": "KNBS",
                "referral_basis": "count discrepancy",
                "content_url": "https://example.invalid/knbs"
            })),
        )
        .await,
    )
    .await;
    assert_eq!(packet["referred_to"], "KNBS");
    assert!(!packet["id"].is_null());
}

#[tokio::test]
async fn a_referral_for_an_unknown_cycle_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/forensics/cycles/no-such-cycle/referrals",
        Some(json!({
            "referred_to": "ODPP",
            "referral_basis": "x",
            "content_url": "https://example.invalid/p"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cycles_are_listed_newest_first() {
    let app = app();
    let first = create_cycle(&app, "RD").await;
    let second = create_cycle(&app, "IV").await;
    let listed = body_json(send(&app, "GET", "/v1/forensics/cycles", None).await).await;
    let cycles = listed["cycles"].as_array().unwrap();
    assert_eq!(cycles.len(), 2);
    assert_eq!(cycles[0]["id"], second.as_str(), "newest first");
    assert_eq!(cycles[1]["id"], first.as_str());
}

#[tokio::test]
async fn routers_do_not_share_cycles() {
    let a = app();
    let b = app();
    create_cycle(&a, "RD").await;
    let listed = body_json(send(&b, "GET", "/v1/forensics/cycles", None).await).await;
    assert!(
        listed["cycles"].as_array().unwrap().is_empty(),
        "cycle state leaked between routers"
    );
}