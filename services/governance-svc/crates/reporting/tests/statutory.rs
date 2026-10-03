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

//! Round-trip tests for the `/reporting/*` statutory surface.
//!
//! All five handlers answered from `vec![]` or `Default::default()`.
//! `get_report` returned a blank `StatutoryReport` for **any** id with `200 OK`,
//! so asking for a report that had never been generated returned an empty report
//! rather than a 404 — and `StatutoryReport::sig` was a non-optional `String`, so
//! that blank report carried an empty signature.
//!
//! The substantive rule is the gating one. A testimony workspace, a judiciary
//! handover and an SC/IT pre-registration all presuppose a report with a body.
//! `ReportStatus::allows_downstream` encodes that, and all three handlers check
//! it — none of them previously looked at the report at all.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_reporting::{router::build_router, state::AppState};
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
    app.clone().oneshot(b.body(payload).unwrap()).await.unwrap()
}

/// Generate a report and return its id. The report starts GENERATING.
async fn generate(app: &axum::Router, period: &str) -> String {
    let resp = send(
        app,
        "POST",
        "/reporting/reports",
        Some(json!({"period": period, "report_type": "ASSEMBLY"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let r = body_json(resp).await;
    assert_eq!(
        r["status"], "GENERATING",
        "a report with no body behind it is not ready"
    );
    r["report_id"].as_str().unwrap().to_string()
}

/// Move a report to READY.
async fn ready(app: &axum::Router, report_id: &str) {
    let resp = send(
        app,
        "POST",
        &format!("/reporting/reports/{report_id}/ready"),
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["status"], "READY");
}

#[tokio::test]
async fn a_generated_report_is_listed_and_has_a_content_hash() {
    let app = app();
    let id = generate(&app, "2027-Q1").await;

    let listed = body_json(send(&app, "GET", "/reporting/reports", None).await).await;
    let reports = listed["reports"].as_array().unwrap();
    assert_eq!(reports.len(), 1, "a generated report must be listed");
    assert_eq!(reports[0]["report_id"], id.as_str());
    assert_eq!(reports[0]["period"], "2027-Q1");
    assert_eq!(reports[0]["report_type"], "ASSEMBLY");

    let hash = reports[0]["content_hash"].as_str().unwrap();
    assert!(
        hash.starts_with("sha256:") && hash.len() > 16,
        "expected a real digest, got {hash:?}"
    );
}

#[tokio::test]
async fn a_report_records_why_it_cannot_render_its_own_body() {
    // Nothing here is signed and the body is assembled outside this service, so
    // the report says so rather than implying otherwise.
    let app = app();
    generate(&app, "2027-Q1").await;
    let report = body_json(send(&app, "GET", "/reporting/reports", None).await).await;
    let reason = report["reports"][0]["blocking_reason"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        !reason.is_empty(),
        "the blocking reason should be recorded on the report"
    );
}

#[tokio::test]
async fn becoming_ready_clears_the_blocking_reason() {
    let app = app();
    let id = generate(&app, "2027-Q1").await;
    ready(&app, &id).await;
    let report =
        body_json(send(&app, "GET", &format!("/reporting/reports/{id}"), None).await).await;
    assert_eq!(report["status"], "READY");
    assert!(
        report["blocking_reason"].is_null(),
        "readiness is the caller's assertion that the body exists"
    );
}

#[tokio::test]
async fn an_unknown_report_is_404_not_a_blank_one() {
    // Previously any id returned an empty StatutoryReport with an empty `sig`.
    let app = app();
    let resp = send(&app, "GET", "/reporting/reports/no-such-report", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn marking_an_unknown_report_ready_is_404() {
    let app = app();
    let resp = send(&app, "POST", "/reporting/reports/no-such/ready", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn generation_requires_a_period() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/reporting/reports",
        Some(json!({"period": "  "})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn testimony_requires_a_ready_report() {
    // The headline. Previously create_testimony_workspace returned a default
    // workspace for any request without looking at the report.
    let app = app();
    let id = generate(&app, "2027-Q1").await;

    let resp = send(
        &app,
        "POST",
        "/reporting/testimony-workspaces",
        Some(json!({"report_id": id, "assignee": "clerk-office"})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "a generating report has no body to testify about"
    );
    let detail = body_json(resp).await["detail"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        detail.contains("GENERATING"),
        "the reason should name the status: {detail}"
    );

    let listed = body_json(send(&app, "GET", "/reporting/testimony-workspaces", None).await).await;
    assert!(
        listed["workspaces"].as_array().unwrap().is_empty(),
        "a rejected workspace must not be stored"
    );
}

#[tokio::test]
async fn testimony_opens_once_the_report_is_ready() {
    let app = app();
    let id = generate(&app, "2027-Q1").await;
    ready(&app, &id).await;

    let workspace = body_json(
        send(
            &app,
            "POST",
            "/reporting/testimony-workspaces",
            Some(json!({"report_id": id.clone(), "assignee": "clerk-office"})),
        )
        .await,
    )
    .await;
    assert_eq!(workspace["report_id"], id.as_str());
    assert_eq!(workspace["assignee"], "clerk-office");
    assert_eq!(workspace["status"], "open");

    let listed = body_json(send(&app, "GET", "/reporting/testimony-workspaces", None).await).await;
    assert_eq!(
        listed["workspaces"].as_array().unwrap().len(),
        1,
        "an opened workspace must be listed; previously always empty"
    );
}

#[tokio::test]
async fn testimony_for_an_unknown_report_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/reporting/testimony-workspaces",
        Some(json!({"report_id": "no-such", "assignee": "a"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn testimony_requires_an_assignee() {
    let app = app();
    let id = generate(&app, "2027-Q1").await;
    ready(&app, &id).await;
    let resp = send(
        &app,
        "POST",
        "/reporting/testimony-workspaces",
        Some(json!({"report_id": id, "assignee": "  "})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_judicial_handover_requires_a_ready_report() {
    let app = app();
    let id = generate(&app, "2027-Q1").await;
    let resp = send(
        &app,
        "POST",
        "/reporting/judiciary-handovers",
        Some(json!({"report_id": id.clone(), "recipient_court": "High Court at Milimani"})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "a report cannot be handed to a court before it is ready"
    );

    ready(&app, &id).await;
    let handover = body_json(
        send(
            &app,
            "POST",
            "/reporting/judiciary-handovers",
            Some(json!({"report_id": id.clone(), "recipient_court": "High Court at Milimani"})),
        )
        .await,
    )
    .await;
    assert_eq!(handover["report_id"], id.as_str());
    assert_eq!(handover["recipient_court"], "High Court at Milimani");
    assert!(!handover["handover_ref"].is_null());
}

#[tokio::test]
async fn handing_a_report_to_a_court_marks_it_delivered() {
    let app = app();
    let id = generate(&app, "2027-Q1").await;
    ready(&app, &id).await;
    send(
        &app,
        "POST",
        "/reporting/judiciary-handovers",
        Some(json!({"report_id": id.clone(), "recipient_court": "High Court"})),
    )
    .await;
    let report =
        body_json(send(&app, "GET", &format!("/reporting/reports/{id}"), None).await).await;
    assert_eq!(
        report["status"], "DELIVERED",
        "delivery is a fact about the report, not just a handover record"
    );
}

#[tokio::test]
async fn a_delivered_report_cannot_return_to_generating() {
    let app = app();
    let id = generate(&app, "2027-Q1").await;
    ready(&app, &id).await;
    send(
        &app,
        "POST",
        "/reporting/judiciary-handovers",
        Some(json!({"report_id": id.clone(), "recipient_court": "High Court"})),
    )
    .await;
    let resp = send(
        &app,
        "POST",
        &format!("/reporting/reports/{id}/ready"),
        None,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "a delivered report is a historical record"
    );
}

#[tokio::test]
async fn a_handover_requires_a_court_and_a_real_report() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/reporting/judiciary-handovers",
        Some(json!({"report_id": "no-such", "recipient_court": "  "})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let missing = send(
        &app,
        "POST",
        "/reporting/judiciary-handovers",
        Some(json!({"report_id": "no-such", "recipient_court": "High Court"})),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn sc_it_preregistration_requires_a_ready_report() {
    let app = app();
    let id = generate(&app, "2027-Q1").await;
    let resp = send(
        &app,
        "POST",
        "/reporting/sc-preregistrations",
        Some(json!({"report_id": id.clone(), "sc_it_contact": "committee@example.invalid"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn sc_it_preregistration_records_no_confirmation_it_did_not_receive() {
    // There is no SC/IT committee endpoint in this workspace to acknowledge a
    // registration, so there is no confirmation reference to record.
    let app = app();
    let id = generate(&app, "2027-Q1").await;
    ready(&app, &id).await;
    let prereg = body_json(
        send(
            &app,
            "POST",
            "/reporting/sc-preregistrations",
            Some(json!({"report_id": id.clone(), "sc_it_contact": "committee@example.invalid"})),
        )
        .await,
    )
    .await;
    assert_eq!(prereg["report_id"], id.as_str());
    assert_eq!(prereg["sc_it_contact"], "committee@example.invalid");
    assert!(
        prereg["confirmation_ref"].is_null(),
        "a reference must not be invented where none was issued"
    );
}

#[tokio::test]
async fn sc_it_preregistration_validates_its_input() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/reporting/sc-preregistrations",
        Some(json!({"report_id": "no-such", "sc_it_contact": "  "})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let missing = send(
        &app,
        "POST",
        "/reporting/sc-preregistrations",
        Some(json!({"report_id": "no-such", "sc_it_contact": "c@example.invalid"})),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_two_surfaces_do_not_share_a_store() {
    let app = app();
    let id = generate(&app, "2027-Q1").await;
    ready(&app, &id).await;
    let contract = body_json(
        send(
            &app,
            "GET",
            "/api/v1/governance/reporting/parliamentary-reports",
            None,
        )
        .await,
    )
    .await;
    assert!(
        contract["items"].as_array().unwrap().is_empty(),
        "a legacy report must not appear in the contract surface"
    );
    assert!(!id.is_empty());
}

#[tokio::test]
async fn routers_do_not_share_reports() {
    let a = app();
    let b = app();
    generate(&a, "2027-Q1").await;
    let listed = body_json(send(&b, "GET", "/reporting/reports", None).await).await;
    assert!(listed["reports"].as_array().unwrap().is_empty());
}
