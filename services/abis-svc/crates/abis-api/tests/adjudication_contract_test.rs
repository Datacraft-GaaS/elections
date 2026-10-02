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

//! The adjudication case lifecycle, end to end.
//!
//! Five of the seven adjudication handlers answered 501 with "no store behind it",
//! and `list_adjudication` answered 200 with an empty page and `total: 0` — which
//! read as a real query that found nothing. A case, an assignment, its evidence and
//! its decision are ordinary state, so they now live in `adjudication.rs` and these
//! tests drive the real routes.
//!
//! `resolve_adjudication` on the legacy PATCH route is still a 501 and is covered
//! in `api_test.rs`: it needs a biometric comparison engine, which is not here.
//!
//! There is no `POST /cases` on the contract, so cases are raised through
//! `AdjudicationStore::open_case` — which is what the enrolment pipeline calls when
//! a capture needs human review. These tests seed that way rather than inventing a
//! route.

use abis_api::{
    adjudication::AdjudicationStore,
    app as build_app,
    state::AppState,
    store::AbisStore,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

fn seeded() -> (axum::Router, abis_api::adjudication::SharedAdjudicationStore) {
    let adjudication = AdjudicationStore::new().shared();
    adjudication.open_case("job-1", "subject-1", "normal");
    adjudication.open_case("job-2", "subject-2", "high");
    let state = AppState {
        db: sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://127.0.0.1:1/none")
            .expect("connect_lazy cannot fail"),
        store: AbisStore::new(),
        adjudication: adjudication.clone(),
    };
    (build_app(Arc::new(state)), adjudication)
}

async fn call(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut b = Request::builder().method(method).uri(uri);
    let req = match body {
        Some(v) => {
            b = b.header("content-type", "application/json");
            b.body(Body::from(v.to_string())).unwrap()
        }
        None => b.body(Body::empty()).unwrap(),
    };
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

const CASES: &str = "/v1/adjudication/cases";

#[tokio::test]
async fn a_raised_case_is_listed() {
    let (app, _adj) = seeded();
    let (status, body) = call(app, "GET", CASES, None).await;
    assert_eq!(status, StatusCode::OK);
    // Previously: an empty page with `total: 0`, which read as a query that found
    // nothing rather than a service that holds nothing.
    assert_eq!(body["total"], 2);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(body["items"][0]["state"], "open");
}

#[tokio::test]
async fn the_status_filter_and_paging_are_applied() {
    let (app, adj) = seeded();
    let second = adj.cases()[1].case_id.clone();
    let (status, _) = call(app.clone(), "GET", CASES, None).await;
    assert_eq!(status, StatusCode::OK);

    // Assign the second case so there is one assigned and one open.
    let (status, _) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{second}/assign"),
        Some(json!({ "adjudicator_id": "adj-1" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = call(app.clone(), "GET", &format!("{CASES}?status=assigned"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1, "the status filter must actually filter");
    assert_eq!(body["items"][0]["case_id"], second);

    let (status, body) = call(app, "GET", &format!("{CASES}?page_size=1&page=2"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 2, "total counts the whole filtered set, not the page");
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn min_score_filters_to_cases_that_were_actually_compared() {
    let (app, adj) = seeded();
    let id = adj.cases()[0].case_id.clone();
    // An undecided case has no comparison behind it, so it has no score and cannot
    // satisfy a threshold — including a threshold of zero.
    let (status, body) = call(app.clone(), "GET", &format!("{CASES}?min_score=0"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0);

    let (status, _) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{id}/decide"),
        Some(json!({
            "outcome": "no_match",
            "adjudicator_id": "adj-1",
            "resolution_notes": "not the same person"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // The decision records the outcome but no score: this endpoint decides a case,
    // it does not compare templates.
    let (_status, body) = call(app, "GET", &format!("{CASES}?min_score=0"), None).await;
    assert_eq!(
        body["total"], 0,
        "a decided case with no comparison score still cannot satisfy a threshold"
    );
}

#[tokio::test]
async fn a_case_is_assigned_once_and_reassignment_is_refused() {
    let (app, adj) = seeded();
    let id = adj.cases()[0].case_id.clone();

    let (status, body) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{id}/assign"),
        Some(json!({ "adjudicator_id": "adj-1" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "assigned");
    assert_eq!(body["assigned_to"], "adj-1");

    // Same adjudicator again: idempotent.
    let (status, _) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{id}/assign"),
        Some(json!({ "adjudicator_id": "adj-1" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // A different adjudicator: refused rather than silently moved.
    let (status, _) = call(
        app,
        "POST",
        &format!("{CASES}/{id}/assign"),
        Some(json!({ "adjudicator_id": "adj-2" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn an_escalation_records_who_sent_it_and_why() {
    let (app, adj) = seeded();
    let id = adj.cases()[0].case_id.clone();
    // Previously: 200 with `status: "escalated"` and the reason echoed back, with
    // nothing escalated.
    let (status, body) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{id}/escalate"),
        Some(json!({ "reason": "capture quality too low to decide", "escalated_by": "adj-1" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "escalated");
    assert!(body["escalation_reason"]
        .as_str()
        .unwrap()
        .contains("capture quality"));

    // An anonymous escalation no queue can triage is refused.
    let (status, _) = call(
        app,
        "POST",
        &format!("{CASES}/{}/escalate", adj.cases()[1].case_id),
        Some(json!({ "reason": "  ", "escalated_by": "  " })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn evidence_is_recorded_and_read_back() {
    let (app, adj) = seeded();
    let id = adj.cases()[0].case_id.clone();

    // Previously: 202 with a minted evidence_id for an upload never received.
    let (status, body) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{id}/evidence"),
        Some(json!({ "kind": "probe", "url": "s3://abis-evidence/abc123.png" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["kind"], "probe");
    let evidence_id = body["evidence_id"].as_str().expect("evidence_id").to_string();

    let (status, body) = call(app.clone(), "GET", &format!("{CASES}/{id}/evidence"), None).await;
    assert_eq!(status, StatusCode::OK);
    let items = body["evidence"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["evidence_id"], evidence_id);
}

#[tokio::test]
async fn an_unknown_case_is_not_an_empty_evidence_list() {
    let (app, _adj) = seeded();
    // An empty list reads as "this case has no evidence", which is a claim about
    // the case rather than about storage. The two must be distinguishable.
    let (status, _) = call(app, "GET", &format!("{CASES}/no-such-case/evidence"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn evidence_without_a_kind_or_url_is_refused() {
    let (app, adj) = seeded();
    let id = adj.cases()[0].case_id.clone();
    for body in [json!({ "kind": "", "url": "s3://x" }), json!({ "kind": "probe", "url": "" })] {
        let (status, _) = call(app.clone(), "POST", &format!("{CASES}/{id}/evidence"), Some(body)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn a_case_is_decided_once_and_then_is_closed_to_changes() {
    let (app, adj) = seeded();
    let id = adj.cases()[0].case_id.clone();

    // The tier-3 dual sign-off check is real and is kept.
    let (status, _) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{id}/decide"),
        Some(json!({
            "outcome": "match",
            "adjudicator_id": "tier3",
            "resolution_notes": "needs a second pair of eyes"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "tier3 requires a second adjudicator");

    let decision = json!({
        "outcome": "match",
        "adjudicator_id": "tier3",
        "second_adjudicator_id": "adj-senior",
        "resolution_notes": "same person, re-enrolled elsewhere"
    });
    let (status, body) = call(app.clone(), "POST", &format!("{CASES}/{id}/decide"), Some(decision.clone())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "decided");
    assert_eq!(body["decision"]["outcome"], "match");
    assert_eq!(body["decision"]["second_adjudicator_id"], "adj-senior");

    // Decided is terminal.
    let (status, _) = call(app.clone(), "POST", &format!("{CASES}/{id}/decide"), Some(decision)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{id}/assign"),
        Some(json!({ "adjudicator_id": "adj-9" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn nothing_can_be_done_to_a_case_that_does_not_exist() {
    let (app, _adj) = seeded();
    let missing = "no-such-case";
    let (status, _) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{missing}/assign"),
        Some(json!({ "adjudicator_id": "adj-1" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{missing}/escalate"),
        Some(json!({ "reason": "why", "escalated_by": "me" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        app.clone(),
        "POST",
        &format!("{CASES}/{missing}/decide"),
        Some(json!({ "outcome": "match", "adjudicator_id": "adj-1", "resolution_notes": "x" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

