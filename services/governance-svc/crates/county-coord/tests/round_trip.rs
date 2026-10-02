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

//! Round-trip tests for county-coord.
//!
//! These exist because the handlers in this crate previously returned success
//! responses describing work they never did: `create_incident` answered 201 for
//! an incident stored nowhere, `list_incidents` always returned an empty array,
//! `sign_mou` answered 204 for any id including ids that never existed, and
//! `resolve_incident` discarded the resolution notes it was given. Each of those
//! passed a status-code-only test, which is how a suite stayed green over
//! endpoints that lied.
//!
//! So these assert behaviour that was previously absent: create then list, sign
//! then observe the status change, and 404 for ids that were never created.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_county_coord::{router::build_router, state::AppState};

fn app() -> axum::Router {
    build_router(AppState::new())
}

async fn json_body(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn post(app: &axum::Router, uri: &str, body: Value) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn get(app: &axum::Router, uri: &str) -> axum::response::Response {
    app.clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn created_incident_appears_in_the_list() {
    let app = app();
    let resp = post(
        &app,
        "/v1/coordination/incidents",
        json!({
            "county_code": "047",
            "incident_type": "long_queue",
            "severity": "High",
            "description": "observed at 14:00"
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = json_body(resp).await;
    let id = created["id"].as_str().expect("created incident needs an id").to_string();

    let listed = json_body(get(&app, "/v1/coordination/incidents").await).await;
    let incidents = listed["incidents"].as_array().expect("incidents array");
    assert_eq!(incidents.len(), 1, "a created incident must be listed");
    assert_eq!(incidents[0]["id"], id.as_str());
    assert_eq!(incidents[0]["description"], "observed at 14:00");
    assert!(
        incidents[0]["resolved_at"].is_null(),
        "a new incident must not carry a resolution"
    );
}

#[tokio::test]
async fn resolving_an_incident_records_it_and_404s_for_unknown_ids() {
    let app = app();
    let created = json_body(
        post(
            &app,
            "/v1/coordination/incidents",
            json!({
                "county_code": "047",
                "incident_type": "long_queue",
                "severity": "High"
            }),
        )
        .await,
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();

    let resolved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/coordination/incidents/{id}/resolve"))
                .header("content-type", "application/json")
                .body(Body::from(json!({"resolution_notes": "queue cleared"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolved.status(), StatusCode::NO_CONTENT);

    let listed = json_body(get(&app, "/v1/coordination/incidents").await).await;
    let first = &listed["incidents"][0];
    assert!(
        !first["resolved_at"].is_null(),
        "resolution must be observable, not just acknowledged"
    );

    // The behaviour that did not exist before: resolving something that was
    // never created used to answer 204 and report success.
    let missing = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/coordination/incidents/never-created/resolve")
                .header("content-type", "application/json")
                .body(Body::from(json!({"resolution_notes": "x"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        missing.status(),
        StatusCode::NOT_FOUND,
        "resolving an incident that does not exist must not report success"
    );
}

#[tokio::test]
async fn created_mou_is_listed_and_signing_is_observable() {
    let app = app();
    let created = json_body(
        post(
            &app,
            "/v1/coordination/mous",
            json!({
                "county_code": "047",
                "ministry_code": "MOF",
                "party_type": "Ministry",
                "effective_date": "2026-01-01",
                "expiry_date": "2027-01-01"
            }),
        )
        .await,
    )
    .await;
    assert_eq!(created["status"], "Draft");
    let id = created["id"].as_str().unwrap().to_string();

    let signed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/coordination/mous/{id}/sign"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"officer_id": "officer-1", "signature_hash": "sha256:abc"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(signed.status(), StatusCode::NO_CONTENT);

    let listed = json_body(get(&app, "/v1/coordination/mous").await).await;
    let mous = listed["mous"].as_array().unwrap();
    assert_eq!(mous.len(), 1);
    assert_eq!(
        mous[0]["status"], "Signed",
        "a signed MOU must read back as signed, not stay draft"
    );
    assert!(
        !mous[0]["signed_at"].is_null(),
        "signed_at must be recorded when signing"
    );
}

#[tokio::test]
async fn signing_an_unknown_mou_is_404_not_204() {
    let app = app();
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/coordination/mous/never-created/sign")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"officer_id": "officer-1", "signature_hash": "sha256:abc"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "signing a non-existent MOU previously answered 204"
    );
}

#[tokio::test]
async fn submitted_dataset_is_listed() {
    let app = app();
    let created = json_body(
        post(
            &app,
            "/v1/coordination/datasets",
            json!({
                "county_code": "047",
                "dataset_type": "Population",
                "source_system": "BVR",
                "dataset_url": "s3://bucket/x.parquet",
                "schema_version": "v3",
                "lawful_basis_code": "LB-1",
                "purpose_code": "P-1",
                "retention_until": "2030-01-01"
            }),
        )
        .await,
    )
    .await;
    assert_eq!(created["dataset_type"], "Population");
    assert!(
        created["approved_by"].is_null(),
        "an unapproved submission must not claim approval"
    );

    let listed = json_body(get(&app, "/v1/coordination/datasets").await).await;
    let datasets = listed["datasets"].as_array().unwrap();
    assert_eq!(datasets.len(), 1, "a submitted dataset must be listed");
    assert_eq!(datasets[0]["id"], created["id"]);
}

#[tokio::test]
async fn empty_state_lists_are_empty_but_not_fabricated() {
    let app = app();
    for uri in [
        "/v1/coordination/incidents",
        "/v1/coordination/mous",
        "/v1/coordination/datasets",
    ] {
        let resp = get(&app, uri).await;
        assert_eq!(resp.status(), StatusCode::OK, "{uri}");
        let body = json_body(resp).await;
        let key = uri.rsplit('/').next().unwrap();
        let items = body[key].as_array().unwrap_or_else(|| {
            panic!("{uri} should return a `{key}` array, got {body}");
        });
        assert!(items.is_empty(), "{uri} on empty state should be empty");
    }
}

#[tokio::test]
async fn state_is_isolated_between_routers() {
    // Two routers built from separate AppState must not share incidents, or a
    // store that leaks across instances would make the round-trip test above
    // pass for the wrong reason.
    let a = app();
    let b = app();
    post(
        &a,
        "/v1/coordination/incidents",
        json!({"county_code": "047", "incident_type": "x", "severity": "High"}),
    )
    .await;
    let b_listed = json_body(get(&b, "/v1/coordination/incidents").await).await;
    assert!(
        b_listed["incidents"].as_array().unwrap().is_empty(),
        "state leaked between routers"
    );
}