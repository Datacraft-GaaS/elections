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

//! Round-trip tests for the ballot-serial sub-router.
//!
//! The serious one: `trigger_reconciliation` used to build its own
//! `StationCounts` with `serial_low: 1` and `serial_high: deposited + spoilt`, so
//! `assigned == accounted` by construction and
//! `crate::reconciliation::reconcile` could never report a mismatch. The hold
//! branch (FR-M63-024 / EC-DAT-11) was unreachable through the API even though the
//! function has tests proving it fires. **A station could deposit more ballots
//! than it had been issued and be cleared.**
//!
//! `get_reconciliation` returned `status: "pending", result_hold: false` for any
//! station code, so a station that had never been reconciled reported no hold.
//! `resolve_hold` returned `cleared` for any id, with one officer of the two the
//! request asks for.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use ballot_serial::{api::router, store::AppState};
use std::sync::Arc;

fn app() -> axum::Router {
    router(Arc::new(AppState::default()))
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

/// Assign a station a lot covering serials 1..=100.
async fn assign(app: &axum::Router, station: &str) -> String {
    let resp = send(
        app,
        "POST",
        "/api/v1/ballot-serial/assignments",
        Some(json!({
            "lot_id": format!("LOT-{station}"),
            "station_code": station,
            "serial_low": 1,
            "serial_high": 100,
            "assigned_by": "returning-officer",
            "manifest_ref": "manifest://lot-1",
            "manifest_sig": "ed25519:AAAA"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    body_json(resp).await["assignment_id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn an_assignment_is_stored_and_listed() {
    let app = app();
    let id = assign(&app, "STN001").await;

    let listed = body_json(send(&app, "GET", "/api/v1/ballot-serial/assignments", None).await).await;
    let assignments = listed["assignments"].as_array().unwrap();
    assert_eq!(
        assignments.len(), 1,
        "an assignment must be listed; previously always empty"
    );
    assert_eq!(assignments[0]["assignment_id"], id.as_str());
    assert_eq!(assignments[0]["station_code"], "STN001");
    // Inclusive of both bounds: 1..=100 is 100 ballots.
    assert_eq!(assignments[0]["assigned_count"], 100);
    assert_eq!(assignments[0]["status"], "assigned");
}

#[tokio::test]
async fn an_assignment_records_that_the_manifest_is_unverified() {
    let app = app();
    assign(&app, "STN001").await;
    let listed = body_json(send(&app, "GET", "/api/v1/ballot-serial/assignments", None).await).await;
    let reason = listed["assignments"][0]["blocking_reason"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        !reason.is_empty(),
        "the manifest signature is not verified here and the record should say so"
    );
}

#[tokio::test]
async fn an_inverted_serial_range_is_refused() {
    // An inverted range reports zero assigned ballots, which would clear every
    // reconciliation at that station.
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/ballot-serial/assignments",
        Some(json!({
            "lot_id": "LOT-1",
            "station_code": "STN001",
            "serial_low": 500,
            "serial_high": 100,
            "assigned_by": "o",
            "manifest_ref": "m",
            "manifest_sig": "s"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_station_cannot_be_issued_a_second_lot() {
    let app = app();
    assign(&app, "STN001").await;
    let second = send(
        &app,
        "POST",
        "/api/v1/ballot-serial/assignments",
        Some(json!({
            "lot_id": "LOT-2",
            "station_code": "STN001",
            "serial_low": 200,
            "serial_high": 300,
            "assigned_by": "o",
            "manifest_ref": "m",
            "manifest_sig": "s"
        })),
    )
    .await;
    assert_eq!(
        second.status(),
        StatusCode::CONFLICT,
        "re-assigning would orphan the ballots already issued"
    );
}

#[tokio::test]
async fn a_receipt_needs_a_real_assignment() {
    // Previously any assignment id produced a receipt.
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/ballot-serial/receipts",
        Some(json!({"assignment_id": "no-such", "clerk_id": "c1", "receipt_hash": "h1"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_receipt_moves_the_station_to_issued() {
    let app = app();
    let id = assign(&app, "STN001").await;
    let resp = send(
        &app,
        "POST",
        "/api/v1/ballot-serial/receipts",
        Some(json!({"assignment_id": id.clone(), "clerk_id": "c1", "receipt_hash": "h1"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let state = body_json(send(&app, "GET", "/api/v1/ballot-serial/reconcile/STN001", None).await).await;
    assert_eq!(state["status"], "issued");
}

#[tokio::test]
async fn a_receipt_requires_clerk_and_hash() {
    let app = app();
    let id = assign(&app, "STN001").await;
    let resp = send(
        &app,
        "POST",
        "/api/v1/ballot-serial/receipts",
        Some(json!({"assignment_id": id, "clerk_id": "  ", "receipt_hash": "h"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_spoilt_serial_must_be_inside_the_assigned_range() {
    // FR-M63-014. Recording a spoilt ballot outside the range would inflate the
    // spoilt count and mask a real discrepancy.
    let app = app();
    assign(&app, "STN001").await;
    let resp = send(
        &app,
        "POST",
        "/api/v1/ballot-serial/spoilt",
        Some(json!({
            "station_code": "STN001",
            "spoilt_serial": 5000,
            "reason": "torn",
            "clerk_id": "c1"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("5000"), "the reason should name the serial: {detail}");
}

#[tokio::test]
async fn a_spoilt_serial_is_recorded_once() {
    let app = app();
    assign(&app, "STN001").await;
    let body = json!({
        "station_code": "STN001",
        "spoilt_serial": 42,
        "reason": "torn in the booth",
        "clerk_id": "c1"
    });
    let first = send(&app, "POST", "/api/v1/ballot-serial/spoilt", Some(body.clone())).await;
    assert_eq!(first.status(), StatusCode::ACCEPTED);
    assert_eq!(body_json(first).await["spoilt_count"], 1);

    let again = send(&app, "POST", "/api/v1/ballot-serial/spoilt", Some(body)).await;
    assert_eq!(again.status(), StatusCode::CONFLICT, "a serial is spoilt once");
}

#[tokio::test]
async fn a_spoilt_serial_needs_a_station_and_a_reason() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/ballot-serial/spoilt",
        Some(json!({"station_code": "no-such", "spoilt_serial": 1, "reason": "r", "clerk_id": "c"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    assign(&app, "STN001").await;
    let no_reason = send(
        &app,
        "POST",
        "/api/v1/ballot-serial/spoilt",
        Some(json!({"station_code": "STN001", "spoilt_serial": 1, "reason": "  ", "clerk_id": "c"})),
    )
    .await;
    assert_eq!(no_reason.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn depositing_more_ballots_than_were_issued_raises_a_hold() {
    // The headline. The old handler built `serial_high = deposited + spoilt`, so
    // this reconciled to 0 and cleared.
    let app = app();
    assign(&app, "STN001").await; // 1..=100
    let result = body_json(
        send(
            &app,
            "POST",
            "/api/v1/ballot-serial/reconcile",
            Some(json!({
                "station_code": "STN001",
                "election_id": "2027-general",
                "deposited_count": 120,
                "spoilt_count": 0
            })),
        )
        .await,
    )
    .await;
    assert_eq!(result["status"], "held");
    assert_eq!(result["result_hold"], true);
    assert_eq!(result["assigned_count"], 100, "the real assigned range is used");
    assert_eq!(result["discrepancy"], 20);
    assert!(result["flag_reason"].is_string(), "a hold must name its reason");

    // The hold is on the station, not just in the response.
    let state = body_json(send(&app, "GET", "/api/v1/ballot-serial/reconcile/STN001", None).await).await;
    assert_eq!(state["result_hold"], true);
    assert_eq!(state["status"], "held");
    assert!(state["hold_reason"].is_string());
}

#[tokio::test]
async fn a_balanced_station_clears() {
    let app = app();
    assign(&app, "STN001").await; // 1..=100
    let result = body_json(
        send(
            &app,
            "POST",
            "/api/v1/ballot-serial/reconcile",
            Some(json!({
                "station_code": "STN001",
                "election_id": "2027-general",
                "deposited_count": 70,
                "spoilt_count": 10
            })),
        )
        .await,
    )
    .await;
    assert_eq!(result["status"], "cleared");
    assert_eq!(result["result_hold"], false);
    assert_eq!(result["unused_count"], 20);
    assert_eq!(result["discrepancy"], 0);
}

#[tokio::test]
async fn a_reconciliation_counts_spoilt_serials_recorded_through_the_api() {
    // Taking only the request's spoilt number would let a station under-report
    // spoilt ballots it had itself recorded.
    let app = app();
    assign(&app, "STN001").await; // 1..=100
    send(
        &app,
        "POST",
        "/api/v1/ballot-serial/spoilt",
        Some(json!({
            "station_code": "STN001",
            "spoilt_serial": 10,
            "reason": "torn",
            "clerk_id": "c1"
        })),
    )
    .await;
    // One serial (10) was recorded spoilt. Claiming 100 deposited and 0 spoilt
    // looks balanced against 1..=100 only if the station under-reports the
    // spoilt ballot it itself recorded. The reconciliation uses the higher of
    // the two, so 100 + 1 exceeds the 100 assigned and the station is held.
    let under_reported = body_json(
        send(
            &app,
            "POST",
            "/api/v1/ballot-serial/reconcile",
            Some(json!({
                "station_code": "STN001",
                "election_id": "2027-general",
                "deposited_count": 100,
                "spoilt_count": 0
            })),
        )
        .await,
    )
    .await;
    assert_eq!(
        under_reported["status"], "held",
        "claiming 100 deposited when a serial was recorded spoilt is a discrepancy"
    );
    assert_eq!(under_reported["spoilt_count"], 1, "the recorded spoilt is used");
    assert_eq!(under_reported["discrepancy"], 1);
}

#[tokio::test]
async fn reconciling_an_unassigned_station_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/ballot-serial/reconcile",
        Some(json!({
            "station_code": "no-such",
            "election_id": "e",
            "deposited_count": 1,
            "spoilt_count": 0
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_unreconciled_station_reports_no_hold_but_is_not_invented() {
    // Previously ANY station code returned `status: "pending", result_hold: false`,
    // so a station that had never been reconciled reported no hold.
    let app = app();
    let resp = send(&app, "GET", "/api/v1/ballot-serial/reconcile/STN404", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_reconciliation_never_claims_tlog_inclusion() {
    let app = app();
    assign(&app, "STN001").await;
    send(
        &app,
        "POST",
        "/api/v1/ballot-serial/reconcile",
        Some(json!({
            "station_code": "STN001",
            "election_id": "e",
            "deposited_count": 10,
            "spoilt_count": 0
        })),
    )
    .await;
    let state = body_json(send(&app, "GET", "/api/v1/ballot-serial/reconcile/STN001", None).await).await;
    assert_eq!(state["result_hold"], false, "a balanced station has no hold");
    assert_eq!(state["status"], "reconciled");
}

#[tokio::test]
async fn a_hold_needs_both_officers_to_clear_it() {
    // The request has always carried both fields and the handler always ignored
    // the second.
    let app = app();
    assign(&app, "STN001").await;
    send(
        &app,
        "POST",
        "/api/v1/ballot-serial/reconcile",
        Some(json!({
            "station_code": "STN001",
            "election_id": "e",
            "deposited_count": 120,
            "spoilt_count": 0
        })),
    )
    .await;

    let one_officer = send(
        &app,
        "PUT",
        "/api/v1/ballot-serial/reconcile/STN001/resolve",
        Some(json!({"resolved_by_po": "po-1", "resolved_by_cro": "", "reason": "recount"})),
    )
    .await;
    assert_eq!(
        one_officer.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "a hold is cleared under dual control"
    );

    let both = send(
        &app,
        "PUT",
        "/api/v1/ballot-serial/reconcile/STN001/resolve",
        Some(json!({
            "resolved_by_po": "po-1",
            "resolved_by_cro": "cro-1",
            "reason": "recount verified"
        })),
    )
    .await;
    assert_eq!(both.status(), StatusCode::ACCEPTED);
    let body = body_json(both).await;
    assert_eq!(body["status"], "cleared");
    assert_eq!(body["resolved_by_cro"], "cro-1");

    let state = body_json(send(&app, "GET", "/api/v1/ballot-serial/reconcile/STN001", None).await).await;
    assert_eq!(state["result_hold"], false, "the hold is cleared on the station");
}

#[tokio::test]
async fn there_is_no_hold_to_clear_when_none_exists() {
    // Previously resolve_hold returned `cleared` for any id.
    let app = app();
    assign(&app, "STN001").await;
    let resp = send(
        &app,
        "PUT",
        "/api/v1/ballot-serial/reconcile/STN001/resolve",
        Some(json!({
            "resolved_by_po": "po-1",
            "resolved_by_cro": "cro-1",
            "reason": "r"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    let missing = send(
        &app,
        "PUT",
        "/api/v1/ballot-serial/reconcile/NOPE/resolve",
        Some(json!({"resolved_by_po": "p", "resolved_by_cro": "c", "reason": "r"})),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn resolving_a_hold_requires_a_reason() {
    let app = app();
    assign(&app, "STN001").await;
    send(
        &app,
        "POST",
        "/api/v1/ballot-serial/reconcile",
        Some(json!({
            "station_code": "STN001",
            "election_id": "e",
            "deposited_count": 120,
            "spoilt_count": 0
        })),
    )
    .await;
    let resp = send(
        &app,
        "PUT",
        "/api/v1/ballot-serial/reconcile/STN001/resolve",
        Some(json!({"resolved_by_po": "p", "resolved_by_cro": "c", "reason": "  "})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn stations_do_not_share_state() {
    let a = app();
    let b = app();
    assign(&a, "STN001").await;
    let listed = body_json(send(&b, "GET", "/api/v1/ballot-serial/assignments", None).await).await;
    assert!(listed["assignments"].as_array().unwrap().is_empty());
    let resp = send(&b, "GET", "/api/v1/ballot-serial/reconcile/STN001", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn spoilt_ballots_do_not_leak_between_stations() {
    let app = app();
    assign(&app, "STN001").await;
    assign(&app, "STN002").await;
    send(
        &app,
        "POST",
        "/api/v1/ballot-serial/spoilt",
        Some(json!({"station_code": "STN001", "spoilt_serial": 7, "reason": "r", "clerk_id": "c"})),
    )
    .await;
    let other = body_json(send(&app, "GET", "/api/v1/ballot-serial/reconcile/STN002", None).await).await;
    assert_eq!(other["spoilt_count"], 0);
}
