//! Round-trip tests for court-interface: real witnesses, real requests, and a
//! bundle that says what it contains.
//!
//! The handlers in this group all keyed off a petition id and all answered from
//! nothing. `get_evidence_bundle` returned 200 with a default-constructed
//! bundle for **any** petition, which in a filing context is worse than an
//! error: an empty bundle and a populated one are indistinguishable from the
//! response alone. `register_witness` returned 200 with a default witness, so a
//! witness appeared registered having never consented. `request_evidence`
//! returned 202 for a request that was never made.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_court::{router::build_router, state::AppState};

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

async fn petition(app: &axum::Router) -> String {
    let resp = send(
        app,
        "POST",
        "/court/petitions",
        Some(json!({
            "petition_ref": "PET-2026-001",
            "petitioner_id_hash": "sha256:p",
            "respondent_code": "CAND-1",
            "contest_code": "CONST-047",
            "grounds": "irregularities in tallying",
            "filed_at": "2026-09-01"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED, "file_petition must work");
    let created = body_json(resp).await;
    created["petition_id"].as_str().expect("petition_id").to_string()
}

fn witness_body() -> Value {
    json!({
        "full_name": "DrAmina Yusuf",
        "expertise": "voter register reconciliation",
        "organisation": "Independent Electoral Commission"
    })
}

#[tokio::test]
async fn registered_witness_is_stored_and_listed() {
    let app = app();
    let id = petition(&app).await;

    let created = body_json(
        send(
            &app,
            "POST",
            &format!("/court/petitions/{id}/witnesses"),
            Some(witness_body()),
        )
        .await,
    )
    .await;
    assert_eq!(created["full_name"], "DrAmina Yusuf");
    assert_eq!(created["petition_id"], id.as_str());
    assert!(!created["registered_at"].is_null());

    let listed = body_json(send(&app, "GET", &format!("/court/petitions/{id}/witnesses"), None).await).await;
    let witnesses = listed["witnesses"].as_array().unwrap();
    assert_eq!(witnesses.len(), 1, "a registered witness must be listed");
    assert_eq!(witnesses[0]["full_name"], "DrAmina Yusuf");
}

#[tokio::test]
async fn a_petition_with_no_witness_lists_empty() {
    // Not 404: listing witnesses reads real state, so an empty array is the
    // truthful answer. Previously it was empty too, but always, for every id.
    let app = app();
    let id = petition(&app).await;
    let listed = body_json(send(&app, "GET", &format!("/court/petitions/{id}/witnesses"), None).await).await;
    assert_eq!(listed["witnesses"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn evidence_request_is_recorded_and_appears_in_the_bundle() {
    let app = app();
    let id = petition(&app).await;

    let accepted = send(
        &app,
        "POST",
        &format!("/court/petitions/{id}/evidence/request"),
        Some(json!({
            "evidence_types": ["voter_register", "transfer_log"],
            "deadline": "2026-10-01",
            "court_ref": "HC/2026/77"
        })),
    )
    .await;
    assert_eq!(accepted.status(), StatusCode::ACCEPTED);

    let bundle = body_json(send(&app, "GET", &format!("/court/petitions/{id}/evidence/bundle"), None).await).await;
    assert_eq!(bundle["petition_id"], id.as_str());
    let items = bundle["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "the recorded request must appear in the bundle");
    assert!(items[0]["item_type"].as_str().unwrap().contains("voter_register"));
    assert_eq!(items[0]["court_ref"], "HC/2026/77");
    assert_eq!(
        items[0]["verified"], false,
        "a recorded request is not verified evidence"
    );
}

#[tokio::test]
async fn evidence_bundle_is_404_before_any_request_exists() {
    // The headline behaviour. This used to answer 200 with a well-formed bundle
    // containing nothing, and a caller checking what the court held could not
    // tell that from a populated bundle.
    let app = app();
    let id = petition(&app).await;
    let resp = send(&app, "GET", &format!("/court/petitions/{id}/evidence/bundle"), None).await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "no request means no bundle, not an empty one"
    );
}

#[tokio::test]
async fn supplementary_request_is_stored_with_the_fields_supplied() {
    let app = app();
    let id = petition(&app).await;
    let created = body_json(
        send(
            &app,
            "POST",
            &format!("/court/petitions/{id}/supplementary"),
            Some(json!({
                "description": "Further witness statement from the returning officer",
                "requested_by": "registrar"
            })),
        )
        .await,
    )
    .await;
    assert_eq!(created["petition_id"], id.as_str());
    assert_eq!(created["description"], "Further witness statement from the returning officer");
    assert_eq!(created["requested_by"], "registrar");
    assert_eq!(created["status"], "OPEN");
    assert!(
        !created["request_id"].as_str().unwrap_or("").is_empty(),
        "a stored request needs an id"
    );
}

#[tokio::test]
async fn everything_on_an_unknown_petition_is_404() {
    // Each of these previously answered 200 or 202 for any id at all.
    let app = app();
    for (method, uri, body) in [
        ("GET", "/court/petitions/never-filed/evidence/bundle", None),
        ("GET", "/court/petitions/never-filed/witnesses", None),
        (
            "POST",
            "/court/petitions/never-filed/witnesses",
            Some(witness_body()),
        ),
        (
            "POST",
            "/court/petitions/never-filed/evidence/request",
            Some(json!({"evidence_types": [], "deadline": "2026-10-01", "court_ref": "X"})),
        ),
        (
            "POST",
            "/court/petitions/never-filed/supplementary",
            Some(json!({"description": "d", "requested_by": "r"})),
        ),
    ] {
        let resp = send(&app, method, uri, body).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{method} {uri}");
    }
}

#[tokio::test]
async fn witnesses_do_not_leak_across_petitions() {
    // Or a witness could appear on a petition they never gave evidence to.
    let app = app();
    let a = petition(&app).await;
    let b = petition(&app).await;
    send(
        &app,
        "POST",
        &format!("/court/petitions/{a}/witnesses"),
        Some(witness_body()),
    )
    .await;
    let b_listed = body_json(
        send(&app, "GET", &format!("/court/petitions/{b}/witnesses"), None).await,
    )
    .await;
    assert!(
        b_listed["witnesses"].as_array().unwrap().is_empty(),
        "witnesses leaked between petitions"
    );
}

#[tokio::test]
async fn evidence_requests_do_not_leak_across_petitions() {
    let app = app();
    let a = petition(&app).await;
    let b = petition(&app).await;
    send(
        &app,
        "POST",
        &format!("/court/petitions/{a}/evidence/request"),
        Some(json!({"evidence_types": ["tally"], "deadline": "2026-10-01", "court_ref": "A"})),
    )
    .await;
    let b_bundle = send(
        &app,
        "GET",
        &format!("/court/petitions/{b}/evidence/bundle"),
        None,
    )
    .await;
    assert_eq!(
        b_bundle.status(),
        StatusCode::NOT_FOUND,
        "another petition's evidence request leaked into this bundle"
    );
}