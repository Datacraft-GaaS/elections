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

//! Round-trip tests for the `/oag/*` surface.
//!
//! The crate had **no state module at all**. `list_engagements` and
//! `list_published_reports` returned `vec![]`, `create_engagement` and
//! `register_published_report` returned `T::default()`, `get_engagement` echoed
//! the path id into a default struct, and `extract_evidence` / `record_response`
//! did the same. So a client could read an engagement that had never been
//! created, evidence could be extracted from a non-existent engagement, and a
//! response could be recorded repeatedly against the same engagement.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_oag::{router::build_router, state::AppState};
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

async fn engagement(app: &axum::Router) -> String {
    let resp = send(
        app,
        "POST",
        "/oag/engagements",
        Some(json!({"scope": "Audit of the 2027 general election", "oag_ref": "OAG/AUD/1/2027"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let e = body_json(resp).await;
    assert_eq!(e["status"], "SCOPING", "a new engagement opens in scoping");
    e["engagement_id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn a_created_engagement_is_listed_and_fetchable() {
    // Previously both reads returned an empty/default shape, so an engagement
    // could be created and never seen again.
    let app = app();
    let id = engagement(&app).await;

    let listed = body_json(send(&app, "GET", "/oag/engagements", None).await).await;
    let engagements = listed["engagements"].as_array().unwrap();
    assert_eq!(engagements.len(), 1, "a created engagement must be listed");
    assert_eq!(engagements[0]["engagement_id"], id.as_str());
    assert_eq!(engagements[0]["oag_ref"], "OAG/AUD/1/2027");

    let one = body_json(send(&app, "GET", &format!("/oag/engagements/{id}"), None).await).await;
    assert_eq!(one["engagement_id"], id.as_str());
    assert_eq!(one["scope"], "Audit of the 2027 general election");
}

#[tokio::test]
async fn an_unknown_engagement_is_404_not_a_default() {
    // get_engagement echoed the path id into a default struct, so asking for an
    // engagement that never existed returned a blank engagement with 200.
    let app = app();
    let resp = send(&app, "GET", "/oag/engagements/no-such-engagement", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn opening_an_engagement_requires_a_scope_and_ref() {
    let app = app();
    for body in [
        json!({"scope": "", "oag_ref": "OAG/1"}),
        json!({"scope": "s", "oag_ref": "  "}),
    ] {
        let resp = send(&app, "POST", "/oag/engagements", Some(body)).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn evidence_is_extracted_with_a_real_digest() {
    let app = app();
    let id = engagement(&app).await;
    let extract = body_json(
        send(
            &app,
            "POST",
            &format!("/oag/engagements/{id}/evidence"),
            Some(json!({
                "description": "Voters register extract, constituency 047",
                "extracted_by": "officer-hash-abc"
            })),
        )
        .await,
    )
    .await;
    assert_eq!(extract["engagement_id"], id.as_str());
    assert_eq!(extract["extracted_by"], "officer-hash-abc");
    let hash = extract["payload_hash"].as_str().unwrap();
    assert!(
        hash.starts_with("sha256:") && hash.len() > 16,
        "expected a real digest, got {hash:?}"
    );
}

#[tokio::test]
async fn evidence_extraction_needs_a_real_engagement() {
    // Previously any id produced an extract with the id echoed back.
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/oag/engagements/no-such/evidence",
        Some(json!({"description": "d", "extracted_by": "o"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_same_extract_yields_the_same_digest() {
    // The digest covers the engagement, the description and the extractor, so it
    // is reproducible from the stored fields.
    let app = app();
    let id = engagement(&app).await;
    let first = body_json(
        send(
            &app,
            "POST",
            &format!("/oag/engagements/{id}/evidence"),
            Some(json!({"description": "Register extract", "extracted_by": "officer-1"})),
        )
        .await,
    )
    .await;
    let second = body_json(
        send(
            &app,
            "POST",
            &format!("/oag/engagements/{id}/evidence"),
            Some(json!({"description": "Register extract", "extracted_by": "officer-1"})),
        )
        .await,
    )
    .await;
    assert_eq!(
        first["payload_hash"], second["payload_hash"],
        "the digest is over the content, not the timestamp"
    );
    assert_ne!(first["extract_id"], second["extract_id"], "each extract is its own record");
}

#[tokio::test]
async fn evidence_extraction_requires_content() {
    let app = app();
    let id = engagement(&app).await;
    for body in [
        json!({"description": "  ", "extracted_by": "o"}),
        json!({"description": "d", "extracted_by": ""}),
    ] {
        let resp = send(
            &app,
            "POST",
            &format!("/oag/engagements/{id}/evidence"),
            Some(body),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn recording_a_response_moves_the_engagement_to_responded() {
    let app = app();
    let id = engagement(&app).await;
    let response = body_json(
        send(
            &app,
            "POST",
            &format!("/oag/engagements/{id}/response"),
            Some(json!({
                "summary": "Querier found no material misstatement",
                "disposition": "no-action",
                "oag_signatory": "signatory-hash-1"
            })),
        )
        .await,
    )
    .await;
    assert_eq!(response["engagement_id"], id.as_str());
    assert_eq!(response["disposition"], "no-action");

    let engagement = body_json(send(&app, "GET", &format!("/oag/engagements/{id}"), None).await).await;
    assert_eq!(
        engagement["status"], "RESPONDED",
        "the engagement itself must record that it was answered"
    );

    let read = body_json(
        send(&app, "GET", &format!("/oag/engagements/{id}/response"), None).await,
    )
    .await;
    assert_eq!(read["response_id"], response["response_id"]);
}

#[tokio::test]
async fn a_second_response_is_refused() {
    // The headline. Previously every call minted a fresh response id for the
    // same engagement, so the OAG's answer was silently replaced.
    let app = app();
    let id = engagement(&app).await;
    let body = json!({"summary": "s", "disposition": "no-action", "oag_signatory": "sig"});
    assert_eq!(
        send(&app, "POST", &format!("/oag/engagements/{id}/response"), Some(body.clone()))
            .await
            .status(),
        StatusCode::CREATED
    );
    let second = send(&app, "POST", &format!("/oag/engagements/{id}/response"), Some(body)).await;
    assert_eq!(
        second.status(),
        StatusCode::CONFLICT,
        "the OAG must not answer the same engagement twice"
    );
    let detail = body_json(second).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("RESPONDED"), "the reason should name the status: {detail}");
}

#[tokio::test]
async fn evidence_cannot_be_extracted_after_a_response() {
    let app = app();
    let id = engagement(&app).await;
    send(
        &app,
        "POST",
        &format!("/oag/engagements/{id}/response"),
        Some(json!({"summary": "s", "disposition": "no-action", "oag_signatory": "sig"})),
    )
    .await;
    let resp = send(
        &app,
        "POST",
        &format!("/oag/engagements/{id}/evidence"),
        Some(json!({"description": "late evidence", "extracted_by": "o"})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "an answered engagement is closed to further extraction"
    );
}

#[tokio::test]
async fn a_response_needs_a_real_engagement() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/oag/engagements/no-such/response",
        Some(json!({"summary": "s", "disposition": "d", "oag_signatory": "sig"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_response_requires_a_summary_and_disposition() {
    let app = app();
    let id = engagement(&app).await;
    for body in [
        json!({"summary": "  ", "disposition": "d", "oag_signatory": "sig"}),
        json!({"summary": "s", "disposition": "", "oag_signatory": "sig"}),
    ] {
        let resp = send(&app, "POST", &format!("/oag/engagements/{id}/response"), Some(body)).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn an_unanswered_engagement_has_no_response() {
    let app = app();
    let id = engagement(&app).await;
    let resp = send(&app, "GET", &format!("/oag/engagements/{id}/response"), None).await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "no response recorded is different from an empty one"
    );
}

#[tokio::test]
async fn responses_do_not_leak_between_engagements() {
    let app = app();
    let a = engagement(&app).await;
    let b = engagement(&app).await;
    send(
        &app,
        "POST",
        &format!("/oag/engagements/{a}/response"),
        Some(json!({"summary": "s", "disposition": "no-action", "oag_signatory": "sig"})),
    )
    .await;
    let resp = send(&app, "GET", &format!("/oag/engagements/{b}/response"), None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_published_report_is_listed() {
    let app = app();
    let created = body_json(
        send(
            &app,
            "POST",
            "/oag/published-reports",
            Some(json!({
                "title": "Auditor-General report on the 2027 general election",
                "report_type": "special-audit",
                "uri": "https://oag.example.invalid/report-1"
            })),
        )
        .await,
    )
    .await;
    assert!(!created["pub_report_id"].is_null());
    assert!(
        created["hash"].is_null(),
        "this service does not hold the report contents, so it must not invent a content hash"
    );

    let listed = body_json(send(&app, "GET", "/oag/published-reports", None).await).await;
    assert_eq!(
        listed["reports"].as_array().unwrap().len(),
        1,
        "a registered report must be listed; previously always empty"
    );
}

#[tokio::test]
async fn a_caller_supplied_report_hash_is_kept() {
    let app = app();
    let created = body_json(
        send(
            &app,
            "POST",
            "/oag/published-reports",
            Some(json!({
                "title": "t",
                "report_type": "annual",
                "uri": "https://oag.example.invalid/r",
                "hash": "sha256:computed-elsewhere"
            })),
        )
        .await,
    )
    .await;
    assert_eq!(created["hash"], "sha256:computed-elsewhere");
}

#[tokio::test]
async fn publishing_requires_a_title_and_uri() {
    let app = app();
    for body in [
        json!({"title": "", "report_type": "a", "uri": "u"}),
        json!({"title": "t", "report_type": "a", "uri": "  "}),
    ] {
        let resp = send(&app, "POST", "/oag/published-reports", Some(body)).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn engagements_are_listed_newest_first() {
    let app = app();
    let first = engagement(&app).await;
    let second = engagement(&app).await;
    let listed = body_json(send(&app, "GET", "/oag/engagements", None).await).await;
    let engagements = listed["engagements"].as_array().unwrap();
    assert_eq!(engagements.len(), 2);
    assert_eq!(engagements[0]["engagement_id"], second.as_str());
    assert_eq!(engagements[1]["engagement_id"], first.as_str());
}

#[tokio::test]
async fn routers_do_not_share_engagements() {
    let a = app();
    let b = app();
    engagement(&a).await;
    let listed = body_json(send(&b, "GET", "/oag/engagements", None).await).await;
    assert!(listed["engagements"].as_array().unwrap().is_empty());
    let resp = send(&b, "GET", "/oag/published-reports", None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let reports = body_json(resp).await;
    assert!(reports["reports"].as_array().unwrap().is_empty());
}