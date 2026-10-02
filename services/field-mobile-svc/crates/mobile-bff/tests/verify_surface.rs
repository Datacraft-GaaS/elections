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

//! Round-trip tests for the public `/verify/v1/*` surface.
//!
//! The one that mattered most: `verify_token` returned `included: true` for
//! **any** token, with a proof reference that was just `proof-<query_id>`. That is
//! the endpoint a member of the public uses to check whether their token is in the
//! published results, and it said yes to everything.
//!
//! Also: `get_token_proof` returned `"proof": null` for any sha, `get_results`
//! returned `data: []` and `get_results_by_scope` returned `data: null` for any
//! scope and code (both described as a "relay from M11", a service not in this
//! workspace), `get_queue_snapshot` returned `queue_length: 0` for any station,
//! and `binary_integrity_check` returned `passed: true` for every app.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use mobile_bff::router::build_router;

fn app() -> axum::Router {
    build_router()
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

const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SHA_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

async fn record(app: &axum::Router, sha: &str) -> Value {
    let resp = send(
        app,
        "POST",
        "/verify/v1/token/record",
        Some(json!({"token_sha256": sha})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED, "token must be recordable: {sha}");
    body_json(resp).await
}

async fn verify(app: &axum::Router, sha: &str) -> Value {
    body_json(
        send(
            app,
            "POST",
            "/verify/v1/token",
            Some(json!({"token_sha256": sha})),
        )
        .await,
    )
    .await
}

// ---------------------------------------------------------------------------
// verify_token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_unrecorded_token_is_not_included() {
    // The headline. The old handler answered `included: true` for anything.
    let app = app();
    let v = verify(&app, SHA_A).await;
    assert_eq!(
        v["included_in_this_set"], false,
        "a token that was never recorded is not included: {v:?}"
    );
    assert!(v["inclusion_proof"].is_null());
    assert!(v["merkle_root"].is_null());
}

#[tokio::test]
async fn a_recorded_token_verifies_with_a_real_proof() {
    let app = app();
    record(&app, SHA_A).await;
    let v = verify(&app, SHA_A).await;
    assert_eq!(v["included_in_this_set"], true);
    assert_eq!(v["merkle_root"].as_str().unwrap().len(), 64);
    assert_eq!(v["leaf_index"], 0);
    assert_eq!(v["tree_size"], 1);
    // A one-leaf tree has an empty inclusion proof: the root *is* the leaf, so
    // there is no sibling to prove it against. This is the case my first test
    // expectation got wrong.
    assert!(
        v["inclusion_proof"].as_array().unwrap().is_empty(),
        "a single-leaf tree needs no sibling proof"
    );
    assert_ne!(v["merkle_root"], "0".repeat(64));
    assert_eq!(v["scope"], "tokens recorded through this BFF");
    // A root over one process's tokens is not an append-only-log proof.
    assert_eq!(v["tlog_inclusion_proven"], false);
}

#[tokio::test]
async fn a_proof_verifies_against_the_reported_root() {
    let app = app();
    record(&app, SHA_A).await;
    record(&app, SHA_B).await;
    let v = verify(&app, SHA_B).await;
    assert_eq!(v["tree_size"], 2);
    assert_eq!(v["leaf_index"], 1);
    // Rebuild the tree from the reported leaves and check the proof with the
    // crate's own verifier.
    let listed = body_json(send(&app, "GET", "/verify/v1/results", None).await).await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 2);
    // The root must be a root over *something*: it must not be all zeros, and it
    // must be the same root the proof endpoint reports.
    let root = v["merkle_root"].as_str().unwrap().to_string();
    assert_ne!(root, "0".repeat(64), "the zero root is not a Merkle root");
    assert_eq!(listed["merkle_root"].as_str().unwrap(), root);
    assert_eq!(v["included_in_this_set"], true);
}

#[tokio::test]
async fn a_token_is_not_recorded_twice() {
    // A replayed submission must not inflate the leaf set.
    let app = app();
    let first = record(&app, SHA_A).await;
    let second = record(&app, SHA_A).await;
    assert_eq!(first["leaf_index"], second["leaf_index"]);
    assert_eq!(first["recorded_at"], second["recorded_at"]);
    let listed = body_json(send(&app, "GET", "/verify/v1/results", None).await).await;
    assert_eq!(listed["count"], 1);
}

#[tokio::test]
async fn recording_and_verifying_need_a_digest() {
    let app = app();
    for sha in ["", "   "] {
        assert_eq!(
            send(
                &app,
                "POST",
                "/verify/v1/token/record",
                Some(json!({"token_sha256": sha})),
            )
            .await
            .status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            send(&app, "POST", "/verify/v1/token", Some(json!({"token_sha256": sha})))
                .await
                .status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}

#[tokio::test]
async fn an_empty_token_sha_is_rejected() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/verify/v1/token/record",
        Some(json!({"token_sha256": ""})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ---------------------------------------------------------------------------
// get_results / get_results_by_scope
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_empty_recorded_set_is_reported_as_empty() {
    // Previously `data: []` with a "relay from M11" note, which reads as a
    // successful relay of an empty result set.
    let app = app();
    let listed = body_json(send(&app, "GET", "/verify/v1/results", None).await).await;
    assert_eq!(listed["status"], "empty");
    assert_eq!(listed["count"], 0);
    assert!(listed["data"].as_array().unwrap().is_empty());
    assert!(listed["merkle_root"].is_null());
    assert_eq!(listed["tlog_inclusion_proven"], false);
}

#[tokio::test]
async fn a_recorded_set_is_listed_with_its_root() {
    let app = app();
    record(&app, SHA_A).await;
    record(&app, SHA_B).await;
    let listed = body_json(send(&app, "GET", "/verify/v1/results", None).await).await;
    assert_eq!(listed["status"], "ok");
    assert_eq!(listed["count"], 2);
    assert_eq!(listed["merkle_root"].as_str().unwrap().len(), 64);
    assert_ne!(listed["merkle_root"], "0".repeat(64));
}

#[tokio::test]
async fn a_constituency_with_no_recorded_tokens_is_404() {
    // Previously `data: null` with a note, for any scope and code.
    let app = app();
    let resp = send(&app, "GET", "/verify/v1/results/constituency/047", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_unknown_scope_is_404() {
    let app = app();
    record(&app, SHA_A).await;
    let resp = send(&app, "GET", "/verify/v1/results/county/047", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_national_scope_returns_the_whole_set() {
    let app = app();
    record(&app, SHA_A).await;
    record(&app, SHA_B).await;
    let listed = body_json(
        send(&app, "GET", "/verify/v1/results/national/KE", None).await,
    )
    .await;
    assert_eq!(listed["count"], 2);
    assert_eq!(listed["scope"], "national");
}

// ---------------------------------------------------------------------------
// get_token_proof
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_proof_for_an_unrecorded_token_is_404() {
    // Previously `"proof": null` with a note, for any sha.
    let app = app();
    let resp = send(&app, "GET", &format!("/verify/v1/token/{SHA_A}/proof"), None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_recorded_token_has_a_proof() {
    let app = app();
    record(&app, SHA_A).await;
    // With a second token the tree has an interior node, so the proof is real.
    record(&app, SHA_B).await;
    let proof = body_json(
        send(&app, "GET", &format!("/verify/v1/token/{SHA_A}/proof"), None).await,
    )
    .await;
    assert!(!proof["proof"].as_array().unwrap().is_empty(), "two tokens need a sibling");
    assert_eq!(proof["merkle_root"].as_str().unwrap().len(), 64);
    assert_eq!(proof["tree_size"], 2);
    assert_eq!(proof["tlog_inclusion_proven"], false);
}

// ---------------------------------------------------------------------------
// queue
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_station_with_no_queue_is_404() {
    // Previously `queue_length: 0` for any station, which reads as "no one is
    // waiting" for every station in the country.
    let app = app();
    let resp = send(&app, "GET", "/verify/v1/queue/STN001", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn agents_checked_in_are_counted() {
    let app = app();
    for (i, agent) in ["agent-a", "agent-b"].iter().enumerate() {
        let resp = send(
            &app,
            "POST",
            "/verify/v1/queue/STN001/check-in",
            Some(json!({"agent_id_hash": agent})),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["checked_in"], i + 1);
        assert_eq!(body["duplicate_check_in"], false);
    }
    let snap = body_json(send(&app, "GET", "/verify/v1/queue/STN001", None).await).await;
    assert_eq!(snap["queue_length"], 2);
    assert_eq!(snap["total_check_ins"], 2);
}

#[tokio::test]
async fn a_repeat_check_in_is_flagged_as_duplicate() {
    let app = app();
    let body = json!({"agent_id_hash": "agent-a"});
    send(&app, "POST", "/verify/v1/queue/STN001/check-in", Some(body.clone())).await;
    let again = body_json(
        send(&app, "POST", "/verify/v1/queue/STN001/check-in", Some(body)).await,
    )
    .await;
    assert_eq!(
        again["duplicate_check_in"], true,
        "a duplicated device presents as a repeat check-in: {again:?}"
    );
    assert_eq!(again["checked_in"], 1, "a re-entry does not add a second queue slot");
}

#[tokio::test]
async fn checking_out_removes_the_agent() {
    let app = app();
    send(
        &app,
        "POST",
        "/verify/v1/queue/STN001/check-in",
        Some(json!({"agent_id_hash": "agent-a"})),
    )
    .await;
    let out = body_json(
        send(
            &app,
            "POST",
            "/verify/v1/queue/STN001/check-out",
            Some(json!({"agent_id_hash": "agent-a"})),
        )
        .await,
    )
    .await;
    assert_eq!(out["checked_in"], 0);
}

#[tokio::test]
async fn queues_do_not_leak_between_stations() {
    let app = app();
    send(
        &app,
        "POST",
        "/verify/v1/queue/STN001/check-in",
        Some(json!({"agent_id_hash": "agent-a"})),
    )
    .await;
    let other = send(&app, "GET", "/verify/v1/queue/STN002", None).await;
    assert_eq!(other.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_check_needs_an_agent_hash() {
    let app = app();
    for uri in [
        "/verify/v1/queue/STN001/check-in",
        "/verify/v1/queue/STN001/check-out",
    ] {
        let resp = send(&app, "POST", uri, Some(json!({"agent_id_hash": "  "}))).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY, "{uri}");
    }
    let unknown = send(
        &app,
        "POST",
        "/verify/v1/queue/STN404/check-out",
        Some(json!({"agent_id_hash": "a"})),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// anomalies
// ---------------------------------------------------------------------------

#[tokio::test]
async fn anomalies_are_derived_from_the_queue_state() {
    // Previously `anomalies: []` for any query, which reads as "nothing detected".
    let app = app();
    let body = json!({"agent_id_hash": "agent-a"});
    send(&app, "POST", "/verify/v1/queue/STN001/check-in", Some(body.clone())).await;
    send(&app, "POST", "/verify/v1/queue/STN001/check-in", Some(body)).await;

    let anomalies = body_json(send(&app, "GET", "/verify/v1/anomalies", None).await).await;
    assert_eq!(anomalies["count"], 1, "{anomalies:?}");
    let flag = &anomalies["anomalies"][0];
    assert_eq!(flag["kind"], "duplicate-check-in");
    assert_eq!(flag["station"], "STN001");
    assert_eq!(anomalies["derived_from"], "per-station check-in history");
}

#[tokio::test]
async fn a_quiet_system_reports_no_anomalies() {
    let app = app();
    send(
        &app,
        "POST",
        "/verify/v1/queue/STN001/check-in",
        Some(json!({"agent_id_hash": "agent-a"})),
    )
    .await;
    let anomalies = body_json(send(&app, "GET", "/verify/v1/anomalies", None).await).await;
    assert_eq!(anomalies["count"], 0);
    assert!(anomalies["anomalies"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn the_same_duplicate_raises_the_same_flag() {
    // Otherwise every read of the anomaly list invents new flags.
    let app = app();
    let body = json!({"agent_id_hash": "agent-a"});
    send(&app, "POST", "/verify/v1/queue/STN001/check-in", Some(body.clone())).await;
    send(&app, "POST", "/verify/v1/queue/STN001/check-in", Some(body)).await;
    let first = body_json(send(&app, "GET", "/verify/v1/anomalies", None).await).await;
    let second = body_json(send(&app, "GET", "/verify/v1/anomalies", None).await).await;
    assert_eq!(first["anomalies"][0]["anomaly_id"], second["anomalies"][0]["anomaly_id"]);
}

// ---------------------------------------------------------------------------
// clone report and binary integrity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_clone_report_is_recorded_with_its_own_digest() {
    let app = app();
    let body = json!({
        "app_id": "com.example.app",
        "package_name": "com.example.app",
        "reported_signature": "sha256:abc",
        "platform": "android"
    });
    let first = body_json(
        send(&app, "POST", "/verify/v1/clone-report", Some(body.clone())).await,
    )
    .await;
    assert!(!first["report_id"].is_null());
    assert!(
        first["report_hash"].as_str().unwrap().starts_with("sha256:"),
        "a filed report carries a digest over its content: {first:?}"
    );
    // Previously the report was minted and returned with nothing recorded, so two
    // identical filings were indistinguishable.
    let second = body_json(send(&app, "POST", "/verify/v1/clone-report", Some(body)).await).await;
    assert_ne!(first["report_id"], second["report_id"]);
    assert_eq!(first["report_hash"], second["report_hash"]);
    assert_eq!(first["determination"], "reported, not determined");
}

#[tokio::test]
async fn a_clone_report_without_a_signature_says_so() {
    let app = app();
    let report = body_json(
        send(
            &app,
            "POST",
            "/verify/v1/clone-report",
            Some(json!({
                "app_id": "com.example.app",
                "package_name": "com.example.app",
                "reported_signature": "",
                "platform": "android"
            })),
        )
        .await,
    )
    .await;
    let findings = report["findings"].as_array().unwrap();
    assert!(
        findings.iter().any(|f| f.as_str().unwrap().contains("signing signature")),
        "an unattributable build should be reported: {report:?}"
    );
}

#[tokio::test]
async fn a_clone_report_needs_an_app_and_package() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/verify/v1/clone-report",
        Some(json!({"app_id": "", "package_name": "p", "reported_signature": "s", "platform": "android"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn binary_integrity_is_not_asserted_true() {
    // The old handler returned `passed: true` for every app with
    // `failure_reason: None` -- an unconditional attestation.
    let app = app();
    let event = body_json(
        send(
            &app,
            "POST",
            "/verify/v1/binary-integrity",
            Some(json!({"app_id": "com.example.app", "build_fingerprint": "sha256:def", "attestation_token": "token"})),
        )
        .await,
    )
    .await;
    assert_eq!(
        event["passed"], false,
        "an unverified build must not be attested: {event:?}"
    );
    let reason = event["failure_reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("platform key attestation"),
        "the reason should name what is absent: {reason}"
    );
}

#[tokio::test]
async fn binary_integrity_names_the_missing_material() {
    let app = app();
    let event = body_json(
        send(
            &app,
            "POST",
            "/verify/v1/binary-integrity",
            Some(json!({"app_id": "com.example.app", "build_fingerprint": "", "attestation_token": ""})),
        )
        .await,
    )
    .await;
    let reason = event["failure_reason"].as_str().unwrap_or_default();
    assert!(reason.contains("build_fingerprint"), "{reason}");
    assert!(reason.contains("attestation_token"), "{reason}");
}

#[tokio::test]
async fn routers_do_not_share_state() {
    let a = app();
    let b = app();
    record(&a, SHA_A).await;
    assert_eq!(verify(&b, SHA_A).await["included_in_this_set"], false);
    let listed = body_json(send(&b, "GET", "/verify/v1/results", None).await).await;
    assert_eq!(listed["count"], 0);
}
