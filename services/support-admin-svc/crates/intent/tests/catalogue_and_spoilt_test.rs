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

//! The adjudication catalogue is real, and a spoilt ballot is recorded.
//!
//! `list_adjudication_catalogue` returned `{ "catalogue": [] }`, which claims there
//! are no adjudication reason codes at all — on the endpoint an officer picks a code
//! from. The codes have been in `catalogue.rs` the whole time: A1-A8 for rejection
//! and V1-V3 for acceptance.
//!
//! `submit_spoilt_ballot` minted a `spoilt_id` and answered 201, leaving the caller
//! a ticket for a spoilt ballot that was never recorded. A spoilt-ballot record is
//! the evidence a spoiled vote is adjudicated against.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> (axum::Router, intent::http::SharedState) {
    let state = intent::http::AppState::new_shared();
    (intent::http::router(state.clone()), state)
}

async fn get(router: axum::Router, uri: &str) -> (StatusCode, Value) {
    let resp = router
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn post(router: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = router
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn the_catalogue_lists_the_reason_codes() {
    let (router, _state) = app();
    let (status, body) = get(router, "/v1/intent/catalogue").await;
    assert_eq!(status, StatusCode::OK);
    let codes = body["catalogue"].as_array().expect("catalogue array");
    assert!(
        codes.len() >= 11,
        "A1-A8 for rejection and V1-V3 for acceptance is 11; got {}",
        codes.len()
    );
    // And it is the crate's own catalogue, not a copy of it.
    assert_eq!(codes.len(), intent::catalogue::CATALOGUE.len());
}

#[tokio::test]
async fn a_spoilt_ballot_is_recorded_and_readable_back() {
    let (router, state) = app();
    let (status, body) = post(
        router.clone(),
        "/v1/intent/spoilt",
        json!({
            "station_code": "NRB-001-PS-042",
            "contest_id": "KE-NATIONAL-PRESIDENT-2027",
            "reason_code": "A3",
            "officer_id": "RO-0001",
            "witness_ids": ["RO-0002", "RO-0003"],
            "replacement_issued": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let spoilt_id = body["spoilt_id"].as_str().expect("spoilt_id").to_string();

    let stored = state.spoilt_ballots.lock().unwrap();
    assert_eq!(
        stored.len(),
        1,
        "the submission must be recorded, not just ticketed"
    );
    assert_eq!(stored[0]["spoilt_id"], spoilt_id);
    assert_eq!(stored[0]["station_code"], "NRB-001-PS-042");
    // The reason code is the point of the record, and it is validated against the
    // catalogue this crate publishes.
    assert_eq!(stored[0]["reason_code"], "A3");
    assert_eq!(stored[0]["replacement_issued"], true);
    assert_eq!(stored[0]["witness_ids"].as_array().unwrap().len(), 2);

    // And it is in the audit trail.
    let audit = state.audit_log.lock().unwrap();
    assert!(
        audit.iter().any(|e| e["subject"] == spoilt_id.as_str()),
        "a spoilt ballot must leave an audit entry"
    );
}

#[tokio::test]
async fn an_incomplete_or_uncatalogued_spoilt_ballot_is_refused() {
    let complete = json!({
        "station_code": "NRB-001-PS-042",
        "contest_id": "KE-NATIONAL-PRESIDENT-2027",
        "reason_code": "A3",
        "officer_id": "RO-0001",
        "witness_ids": ["RO-0002"]
    });
    // Missing pieces.
    for field in ["station_code", "contest_id", "reason_code", "officer_id"] {
        let mut body = complete.clone();
        body[field] = json!("  ");
        let (router, state) = app();
        let (status, _) = post(router, "/v1/intent/spoilt", body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "missing {field}");
        assert!(
            state.spoilt_ballots.lock().unwrap().is_empty(),
            "nothing is recorded for a refused submission"
        );
    }
    // A reason code the catalogue does not publish.
    let (router, state) = app();
    let mut body = complete.clone();
    body["reason_code"] = json!("MADE-UP-9");
    let (status, _) = post(router, "/v1/intent/spoilt", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(state.spoilt_ballots.lock().unwrap().is_empty());

    // No witnesses.
    let (router, state) = app();
    let mut body = complete;
    body["witness_ids"] = json!([]);
    let (status, _) = post(router, "/v1/intent/spoilt", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(state.spoilt_ballots.lock().unwrap().is_empty());
}
