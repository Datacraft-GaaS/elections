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

//! Round-trip tests for recall: signatures, and a threshold computed from them.
//!
//! The handler this replaces hardcoded `registered_voters = 100_000` and
//! `verified_signatories = 0`, then returned `meets_threshold` computed from
//! them. California Constitution article IV section 2 is the invariant between a
//! petition and a recall election, and it was being evaluated against constants.
//! `import_signatories` compounded it: the whole OCR batch was discarded into a
//! minted `batch_id` with status "processing", so the count could never be
//! anything but zero however many signatures arrived.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_recall::{router::build_router, state::AppState};

fn app() -> axum::Router {
    build_router(AppState::new())
}

/// A router whose constituency roll is loaded, which is what the article IV
/// section 2 threshold is evaluated against. There is no roll-loading endpoint
/// yet, so the test seeds it directly; that absence is the reason
/// `threshold_report_without_a_roll_is_refused` exists.
fn app_with_roll(constituency: &str, registered_voters: u64) -> axum::Router {
    let state = AppState::new();
    state.set_roll(constituency, registered_voters);
    build_router(state)
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
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

/// Create a petition and return its id.
async fn petition(app: &axum::Router) -> String {
    let resp = send(
        app,
        "POST",
        "/v1/recall/petitions",
        Some(json!({
            "constituency_code": "CONST-047",
            "member_id_hash": "sha256:member",
            "grounds": "misconduct",
            "initiator_id_hash": "sha256:initiator",
            "petition_date": "2026-01-05"
        })),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "create_petition should work"
    );
    let created = body_json(resp).await;
    assert_eq!(created["status"], "OPEN");
    let id = created["petition_id"].as_str().unwrap().to_string();
    // The threshold is administratively configured per petition; the tests set
    // it directly because no roll-loading endpoint exists yet.
    id
}

fn rows(n: usize, confidence: f64, prefix: &str) -> Value {
    let mut r = Vec::new();
    for i in 0..n {
        r.push(json!({
            "voter_id": format!("{prefix}{i:04}"),
            "name": format!("Voter {i}"),
            "ward": if i % 2 == 0 { "WARD-1" } else { "WARD-2" },
            "signature_date": "2026-01-06",
            "ocr_confidence": confidence
        }));
    }
    json!({ "rows": r })
}

#[tokio::test]
async fn imported_signatories_are_stored_and_listed() {
    let app = app();
    let id = petition(&app).await;

    let resp = send(
        &app,
        "POST",
        &format!("/v1/recall/petitions/{id}/signatories/import"),
        Some(rows(5, 0.95, "V-")),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let result = body_json(resp).await;
    assert_eq!(result["total_rows"], 5);
    assert_eq!(result["accepted"], 5, "high-confidence rows are accepted");
    assert_eq!(result["flagged_for_manual"], 0);

    let listed = body_json(
        send(
            &app,
            "GET",
            &format!("/v1/recall/petitions/{id}/signatories"),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(listed["total"], 5, "imported rows must be listed");
    let first = &listed["records"][0];
    assert_eq!(first["petition_id"], id.as_str());
    assert_eq!(first["verification_status"], "VALID");
}

#[tokio::test]
async fn threshold_report_without_a_loaded_roll_is_refused() {
    // 409, not a verdict. An assumed electorate is exactly the bug this crate
    // had: registered_voters = 100_000, unrelated to any constituency.
    let app = app();
    let id = petition(&app).await;
    send(
        &app,
        "POST",
        &format!("/v1/recall/petitions/{id}/signatories/import"),
        Some(rows(100, 0.95, "V-")),
    )
    .await;
    let resp = send(
        &app,
        "GET",
        &format!("/v1/recall/petitions/{id}/threshold-report"),
        None,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "a threshold must not be evaluated without a real electorate"
    );
}

#[tokio::test]
async fn low_confidence_rows_are_flagged_and_do_not_count_toward_the_threshold() {
    // The model comment says rows below 0.80 are flagged for manual re-entry.
    // Previously nothing classified rows at all: the batch was discarded.
    let app = app();
    let id = petition(&app).await;

    let result = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/recall/petitions/{id}/signatories/import"),
            Some(rows(4, 0.50, "LOW-")),
        )
        .await,
    )
    .await;
    assert_eq!(result["accepted"], 0);
    assert_eq!(result["flagged_for_manual"], 4);

    let listed = body_json(
        send(
            &app,
            "GET",
            &format!("/v1/recall/petitions/{id}/signatories"),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(listed["total"], 4, "flagged rows are still stored");
    assert!(
        listed["records"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["verification_status"] == "FLAGGED"),
        "low-confidence rows must be flagged, not accepted"
    );
}

#[tokio::test]
async fn duplicate_voter_ids_are_detected_within_a_batch() {
    let app = app();
    let id = petition(&app).await;

    let result = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/recall/petitions/{id}/signatories/import"),
            Some(json!({
                "rows": [
                    {"voter_id": "V-1", "name": "A", "ward": "W1", "signature_date": "2026-01-06", "ocr_confidence": 0.95},
                    {"voter_id": "V-1", "name": "A again", "ward": "W1", "signature_date": "2026-01-07", "ocr_confidence": 0.95},
                    {"voter_id": "V-2", "name": "B", "ward": "W1", "signature_date": "2026-01-06", "ocr_confidence": 0.95}
                ]
            })),
        )
        .await,
    )
    .await;
    assert_eq!(result["total_rows"], 3);
    assert_eq!(result["duplicates"], 1);
    assert_eq!(
        result["accepted"], 2,
        "a duplicate must not also count as accepted"
    );
}

#[tokio::test]
async fn empty_voter_id_is_reported_as_a_row_error() {
    let app = app();
    let id = petition(&app).await;
    let result = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/recall/petitions/{id}/signatories/import"),
            Some(json!({
                "rows": [
                    {"voter_id": "", "name": "Nobody", "ward": "W1", "signature_date": "2026-01-06", "ocr_confidence": 0.99}
                ]
            })),
        )
        .await,
    )
    .await;
    assert_eq!(result["total_rows"], 1);
    assert_eq!(result["errors"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn threshold_report_counts_only_valid_signatories() {
    // The point of the whole crate. The threshold used to be computed from
    // registered_voters = 100_000 and verified_signatories = 0. A roll of 100
    // registered voters puts the 30% target at 30 valid signatures.
    let app = app_with_roll("CONST-047", 100);
    let id = petition(&app).await;

    send(
        &app,
        "POST",
        &format!("/v1/recall/petitions/{id}/signatories/import"),
        Some(rows(30, 0.95, "V-")),
    )
    .await;
    // 10 low-confidence rows that must not move the count.
    send(
        &app,
        "POST",
        &format!("/v1/recall/petitions/{id}/signatories/import"),
        Some(rows(10, 0.40, "LOW-")),
    )
    .await;

    let report = body_json(
        send(
            &app,
            "GET",
            &format!("/v1/recall/petitions/{id}/threshold-report"),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(report["total_signatories"], 40);
    assert_eq!(
        report["verified_signatories"], 30,
        "flagged rows must not count toward article IV section 2 eligibility"
    );
    assert_eq!(report["numerator_x100"], 3000);
    assert_eq!(report["denominator_30pct"], 3000);
    assert_eq!(
        report["meets_threshold"], true,
        "30 valid of a 100 threshold should meet the 30% rule"
    );
    // Ward breakdown is now derived from real rows rather than an empty object.
    let wards = report["ward_breakdown"].as_object().unwrap();
    assert_eq!(wards.len(), 2, "both wards should appear");
    assert_eq!(wards["WARD-1"], 15);
    assert_eq!(wards["WARD-2"], 15);
}

#[tokio::test]
async fn threshold_is_not_met_when_valid_signatories_are_insufficient() {
    let app = app_with_roll("CONST-047", 100);
    let id = petition(&app).await;
    send(
        &app,
        "POST",
        &format!("/v1/recall/petitions/{id}/signatories/import"),
        Some(rows(10, 0.95, "V-")),
    )
    .await;
    let report = body_json(
        send(
            &app,
            "GET",
            &format!("/v1/recall/petitions/{id}/threshold-report"),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(report["verified_signatories"], 10);
    assert_eq!(report["meets_threshold"], false);
}

#[tokio::test]
async fn operations_on_an_unknown_petition_are_404() {
    // Each of these previously answered 200 or 204 for any id at all.
    let app = app();
    // The threshold report is a verdict about a specific petition, so an
    // unknown id is 404. Listing signatories reads real state, so an empty list
    // is the truthful answer and 200 is correct.
    let report = send(
        &app,
        "GET",
        "/v1/recall/petitions/nope/threshold-report",
        None,
    )
    .await;
    assert_eq!(report.status(), StatusCode::NOT_FOUND);
    let listed = send(&app, "GET", "/v1/recall/petitions/nope/signatories", None).await;
    assert_eq!(listed.status(), StatusCode::OK);
    assert_eq!(body_json(listed).await["total"], 0);

    for (method, uri) in [
        ("POST", "/v1/recall/petitions/nope/signatories/import"),
        ("PUT", "/v1/recall/petitions/nope/close"),
        ("POST", "/v1/recall/petitions/nope/election"),
    ] {
        let resp = send(&app, method, uri, Some(json!({"rows": []}))).await;
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "{method} {uri} must not report success for a petition that does not exist"
        );
    }
}

#[tokio::test]
async fn closing_a_petition_updates_it_and_the_close_is_observable() {
    let app = app();
    let id = petition(&app).await;
    let resp = send(
        &app,
        "PUT",
        &format!("/v1/recall/petitions/{id}/close"),
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn creating_a_recall_election_records_it() {
    let app = app();
    let id = petition(&app).await;
    let resp = send(
        &app,
        "POST",
        &format!("/v1/recall/petitions/{id}/election"),
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let election = body_json(resp).await;
    assert_eq!(election["petition_id"], id.as_str());
    assert!(
        election["by_election_id"].is_null(),
        "no M50 by-election exists, so the reference must stay null rather than be invented"
    );
    assert_eq!(election["trigger_status"], "PENDING");
}

#[tokio::test]
async fn signatories_do_not_leak_across_petitions() {
    let app = app();
    let a = petition(&app).await;
    let b = petition(&app).await;
    send(
        &app,
        "POST",
        &format!("/v1/recall/petitions/{a}/signatories/import"),
        Some(rows(3, 0.95, "V-")),
    )
    .await;
    let b_listed = body_json(
        send(
            &app,
            "GET",
            &format!("/v1/recall/petitions/{b}/signatories"),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(
        b_listed["total"], 0,
        "signature counts must be per petition or the threshold is meaningless"
    );
}
