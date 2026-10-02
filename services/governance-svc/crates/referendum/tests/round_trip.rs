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

//! Round-trip tests for referendum: a real results chain, and the declaration
//! invariant actually enforced.
//!
//! The chain this crate has is submit -> certify -> national aggregate ->
//! declare, and every link was severed. `submit_county_result` answered 201 with
//! a provisional result it never recorded, `certify_county_result` logged and
//! answered 204, `get_national_result` synthesised an aggregate from constants
//! with both thresholds evaluated at zero, and `declare_referendum` returned 204
//! while naming `NoDeclarationWithoutBothThresholds` in a comment without ever
//! calling `can_declare`.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_referendum::{router::build_router, state::AppState};

fn app() -> axum::Router {
    build_router(AppState::new())
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

async fn referendum(app: &axum::Router) -> String {
    let resp = send(
        app,
        "POST",
        "/v1/referendums",
        Some(json!({
            "election_id": "ELEC-2026",
            "question_text": {"en": "Shall the county ordinance be adopted?"},
            "scheduled_date": "2026-11-01"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED, "create_referendum must work");
    let created = body_json(resp).await;
    assert_eq!(created["referendum"]["county_threshold"], 24);
    let id = id_of(created["referendum"]["id"].as_str());
    id
}

fn id_of(v: Option<&str>) -> String {
    v.expect("expected an id").to_string()
}

async fn submit(app: &axum::Router, id: &str, county: &str, yes: u64, no: u64) {
    let resp = send(
        app,
        "POST",
        &format!("/v1/referendums/{id}/county-results"),
        Some(json!({
            "county_code": county,
            "yes_votes": yes,
            "no_votes": no,
            "rejected_votes": 0,
            "registered_voters": 1000
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    assert_eq!(
        created["county_yes_majority"],
        yes > no,
        "majority must be computed from the real tally"
    );
    assert_eq!(created["status"], "provisional");
}

#[tokio::test]
async fn created_referendum_is_listed_and_retrievable() {
    let app = app();
    let id = referendum(&app).await;

    let listed = body_json(send(&app, "GET", "/v1/referendums", None).await).await;
    assert_eq!(listed["total"], 1, "a created referendum must be listed");

    // Positive case: the id that was created reads back the record.
    let fetched = body_json(send(&app, "GET", &format!("/v1/referendums/{id}"), None).await).await;
    assert_eq!(fetched["id"], id.as_str());
    assert_eq!(fetched["election_id"], "ELEC-2026");

    // Previously get_referendum echoed any id into a default record and
    // answered 200, so this could not fail.
    let missing = send(&app, "GET", "/v1/referendums/never-created", None).await;
    assert_eq!(
        missing.status(),
        StatusCode::NOT_FOUND,
        "a referendum that does not exist must 404, not echo back a default"
    );
}

#[tokio::test]
async fn national_aggregate_is_computed_from_submitted_county_results() {
    // The whole point. This used to report counties_reported 0 and
    // counties_yes_majority 0 with both thresholds evaluated at zero.
    let app = app();
    let id = referendum(&app).await;
    submit(&app, &id, "001", 600, 400).await;
    submit(&app, &id, "002", 300, 700).await;
    submit(&app, &id, "003", 550, 450).await;

    let national = body_json(
        send(&app, "GET", &format!("/v1/referendums/{id}/national-result"), None).await,
    )
    .await;
    assert_eq!(national["counties_reported"], 3);
    assert_eq!(national["counties_yes_majority"], 2, "001 and 003 carried YES");
    assert_eq!(national["total_yes"], 1450);
    assert_eq!(national["total_no"], 1550);
    assert_eq!(
        national["meets_county_threshold"], false,
        "2 of 47 counties is far short of the 24 threshold"
    );
    assert_eq!(
        national["meets_national_majority"], false,
        "1450 yes against 1550 no is not a national majority"
    );
}

#[tokio::test]
async fn certifying_a_county_result_is_observable() {
    // Previously logged and returned 204 for a result that existed nowhere.
    let app = app();
    let id = referendum(&app).await;
    submit(&app, &id, "001", 600, 400).await;

    let certified = send(
        &app,
        "PUT",
        &format!("/v1/referendums/{id}/county-results/001/certify"),
        None,
    )
    .await;
    assert_eq!(certified.status(), StatusCode::NO_CONTENT);

    // Certifying a county that submitted nothing must not report success.
    let unknown = send(
        &app,
        "PUT",
        &format!("/v1/referendums/{id}/county-results/047/certify"),
        None,
    )
    .await;
    assert_eq!(
        unknown.status(),
        StatusCode::NOT_FOUND,
        "certifying a county with no submitted result previously returned 204"
    );
}

#[tokio::test]
async fn submission_to_an_unknown_referendum_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/referendums/never-created/county-results",
        Some(json!({"county_code": "001", "yes_votes": 1, "no_votes": 0})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn declaration_is_refused_when_the_threshold_is_not_met() {
    // The old handler returned 204 here, under a comment claiming the
    // NoDeclarationWithoutBothThresholds invariant was enforced.
    let app = app();
    let id = referendum(&app).await;
    submit(&app, &id, "001", 600, 400).await;

    let resp = send(&app, "POST", &format!("/v1/referendums/{id}/declare"), None).await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "declaring on 1 of 24 counties must be refused"
    );
    let problem = body_json(resp).await;
    let detail = problem["detail"].as_str().unwrap_or("");
    assert!(
        detail.contains("cannot declare"),
        "the refusal should say why, got: {detail}"
    );
}

#[tokio::test]
async fn declaration_is_permitted_once_both_thresholds_are_met() {
    // 24 of 47 counties with a YES majority, and yes exceeding no nationally.
    let app = app();
    let id = referendum(&app).await;
    for n in 1..=24 {
        submit(&app, &id, &format!("{n:03}"), 600, 400).await;
    }
    submit(&app, &id, "025", 100, 900).await;

    let national = body_json(
        send(&app, "GET", &format!("/v1/referendums/{id}/national-result"), None).await,
    )
    .await;
    assert_eq!(national["meets_county_threshold"], true);
    assert_eq!(national["meets_national_majority"], true);

    let resp = send(&app, "POST", &format!("/v1/referendums/{id}/declare"), None).await;
    assert_eq!(
        resp.status(),
        StatusCode::NO_CONTENT,
        "with both thresholds met the declaration should succeed"
    );

    let declared = body_json(send(&app, "GET", &format!("/v1/referendums/{id}"), None).await).await;
    assert_eq!(
        declared["status"], "declared",
        "the referendum status must reflect the declaration"
    );
}

#[tokio::test]
async fn declaration_of_an_unknown_referendum_is_404() {
    let app = app();
    let resp = send(&app, "POST", "/v1/referendums/never-created/declare", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn results_do_not_leak_across_referendums() {
    // Or the aggregate would be computed from another question's counties.
    let app = app();
    let a = referendum(&app).await;
    let b = referendum(&app).await;
    submit(&app, &a, "001", 600, 400).await;
    let b_national = body_json(
        send(&app, "GET", &format!("/v1/referendums/{b}/national-result"), None).await,
    )
    .await;
    assert_eq!(
        b_national["counties_reported"], 0,
        "county results leaked between referendums"
    );
}