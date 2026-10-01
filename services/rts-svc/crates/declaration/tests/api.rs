//! Round-trip tests for the declaration sub-router.
//!
//! Every handler here previously answered from the path id alone. `get_declaration`
//! returned `status: "pending"` for **any** id, so reading a declaration that did
//! not exist reported a real-looking pending declaration. `declare_result` returned
//! `status: "declared"` for any id with no winner, no `declared_at`, and no check
//! that a tally had been evaluated — the Art. 138(4) rules engine in
//! `crate::rules` was never called from anywhere. `set_runoff` and `set_tie`
//! asserted a run-off or tie against a declaration that need not exist, and
//! `get_certificate` returned `not_issued` without reading anything, which made
//! the EC-LEG-06 rule that a tie-pending declaration may never be certified
//! unreachable.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use declaration::{api::router, store::AppState};
fn app() -> axum::Router {
    router(AppState::new())
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 8 * 1024 * 1024)
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

async fn initiate(app: &axum::Router) -> String {
    let resp = send(
        app,
        "POST",
        "/api/v1/declarations",
        Some(json!({
            "contest_id": "president",
            "election_id": "2027-general",
            "tally_snapshot_id": "tally-snap-1"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = body_json(resp).await;
    assert_eq!(body["status"], "pending");
    body["declaration_id"].as_str().unwrap().to_string()
}

/// A candidate spread across 47 counties, matching the distribution idiom the
/// crate's own rules tests use: votes are spread over the first 24 counties.
fn dist(candidate_id: &str, votes: u64) -> Value {
    let per_county_total = (1000u64 + 46) / 47;
    let per_county_votes = (votes + 46) / 47;
    let counties: Vec<Value> = (0..47)
        .map(|i| {
            json!({
                "county_code": format!("{:02}", i + 1),
                "votes": if i < 24 { per_county_votes } else { 0 },
                "county_total_valid": per_county_total
            })
        })
        .collect();
    json!({
        "candidate_id": candidate_id,
        "national_votes": votes,
        "county_votes": counties
    })
}

/// A tally where `a` wins outright: strict majority **and** the 25% threshold in
/// 24 of 47 counties. Art. 138(4) requires both.
fn winning_tally() -> Value {
    json!({
        "total_valid_votes": 1000,
        "candidates": [dist("a", 600), dist("b", 400)]
    })
}

/// A tally with no outright winner and no tie: 490/480/30 out of 1000. No
/// candidate reaches a majority, and the top two differ, so a run-off is
/// required.
fn runoff_tally() -> Value {
    json!({
        "total_valid_votes": 1000,
        "candidates": [dist("a", 490), dist("b", 480), dist("c", 30)]
    })
}

/// A tally with an exact tie on the top two: 500/500 out of 1000.
fn tie_tally() -> Value {
    json!({
        "total_valid_votes": 1000,
        "candidates": [dist("a", 500), dist("b", 500)]
    })
}

/// A national majority that fails the county threshold: `a` takes 900 votes in a
/// single county, so the 24-of-47 test is not met.
fn majority_without_county_threshold_tally() -> Value {
    json!({
        "total_valid_votes": 1000,
        "candidates": [
            {
                "candidate_id": "a",
                "national_votes": 900,
                "county_votes": [{"county_code": "01", "votes": 900, "county_total_valid": 1000}]
            },
            dist("b", 100)
        ]
    })
}

async fn declare(app: &axum::Router, id: &str, tally: Value) -> axum::response::Response {
    send(
        app,
        "PUT",
        &format!("/api/v1/declarations/{id}/declare"),
        Some(json!({
            "declared_by": "returning-officer",
            "ceremony_id": "ceremony-1",
            "hsm_signature": "ed25519:AAAA",
            "tally": tally
        })),
    )
    .await
}

#[tokio::test]
async fn an_initiated_declaration_is_readable() {
    let app = app();
    let id = initiate(&app).await;
    let body = body_json(send(&app, "GET", &format!("/api/v1/declarations/{id}"), None).await).await;
    assert_eq!(body["declaration_id"], id.as_str());
    assert_eq!(body["status"], "pending");
    assert!(body["winner_id"].is_null());
    assert!(body["declared_at"].is_null());
}

#[tokio::test]
async fn an_unknown_declaration_is_404_not_a_pending_one() {
    // Previously any id returned status "pending" with 200, which reads as a real
    // declaration that simply had not been made.
    let app = app();
    let resp = send(&app, "GET", "/api/v1/declarations/no-such", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn initiation_requires_a_contest_and_election() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/declarations",
        Some(json!({"contest_id": "  ", "election_id": "e", "tally_snapshot_id": "s"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_majority_plus_county_threshold_declares_a_winner() {
    let app = app();
    let id = initiate(&app).await;
    let body = body_json(declare(&app, &id, winning_tally()).await).await;
    assert_eq!(body["status"], "declared");
    assert_eq!(body["winner_id"], "a");
    assert!(!body["declared_at"].is_null());
    assert_eq!(body["outcome"]["Declared"]["winner_id"], "a");
}

#[tokio::test]
async fn a_majority_without_the_county_threshold_does_not_declare() {
    // Art. 138(4) requires BOTH conditions. A national majority alone is not a
    // win, and the old handler called any submission "declared".
    let app = app();
    let id = initiate(&app).await;
    let body = body_json(declare(&app, &id, majority_without_county_threshold_tally()).await)
        .await;
    assert_ne!(
        body["status"], "declared",
        "a bare national majority is not a declaration: {body:?}"
    );
    assert!(body["winner_id"].is_null());
}

#[tokio::test]
async fn no_majority_declares_a_runoff_not_a_winner() {
    let app = app();
    let id = initiate(&app).await;
    let body = body_json(declare(&app, &id, runoff_tally()).await).await;
    assert_eq!(body["status"], "runoff_required");
    assert!(body["winner_id"].is_null());
    assert!(body["outcome"]["RunoffRequired"].is_object());
}

#[tokio::test]
async fn an_exact_tie_declares_tie_pending() {
    let app = app();
    let id = initiate(&app).await;
    let body = body_json(declare(&app, &id, tie_tally()).await).await;
    assert_eq!(body["status"], "tie_pending");
    assert!(body["winner_id"].is_null());
}

#[tokio::test]
async fn declaring_records_that_the_signature_is_unverified() {
    // The ceremony signature is recorded as supplied but not verified: the HSM
    // signer's public key is not reachable from this process.
    let app = app();
    let id = initiate(&app).await;
    let body = body_json(declare(&app, &id, winning_tally()).await).await;
    let reason = body["blocking_reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("HSM"),
        "the reason should name what is missing: {reason:?}"
    );
}

#[tokio::test]
async fn declaring_an_unknown_declaration_is_404() {
    // Previously returned 202 "declared" for any id.
    let app = app();
    let resp = declare(&app, "no-such", winning_tally()).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_declaration_cannot_be_made_twice() {
    // A second submission would silently replace the first outcome.
    let app = app();
    let id = initiate(&app).await;
    assert_eq!(declare(&app, &id, winning_tally()).await.status(), StatusCode::ACCEPTED);
    let second = declare(&app, &id, runoff_tally()).await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
    let detail = body_json(second).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("declared"), "the reason should name the status: {detail}");
}

#[tokio::test]
async fn a_runoff_cannot_be_asserted_where_the_rules_do_not_produce_one() {
    // Previously set_runoff returned runoff_required for any id at all.
    let app = app();
    let id = initiate(&app).await;
    let resp = send(
        &app,
        "PUT",
        &format!("/api/v1/declarations/{id}/runoff"),
        Some(winning_tally()),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "a tally with a winner is not a run-off"
    );
    let state = body_json(send(&app, "GET", &format!("/api/v1/declarations/{id}"), None).await).await;
    assert_eq!(state["status"], "pending", "the declaration must be unchanged");
}

#[tokio::test]
async fn a_runoff_is_recorded_where_the_rules_produce_one() {
    let app = app();
    let id = initiate(&app).await;
    let resp = send(
        &app,
        "PUT",
        &format!("/api/v1/declarations/{id}/runoff"),
        Some(runoff_tally()),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    assert_eq!(body_json(resp).await["status"], "runoff_required");
}

#[tokio::test]
async fn a_tie_cannot_be_asserted_where_the_rules_find_a_winner() {
    let app = app();
    let id = initiate(&app).await;
    let resp = send(
        &app,
        "PUT",
        &format!("/api/v1/declarations/{id}/tie"),
        Some(winning_tally()),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let state = body_json(send(&app, "GET", &format!("/api/v1/declarations/{id}"), None).await).await;
    assert_eq!(state["status"], "pending");
}

#[tokio::test]
async fn a_tie_is_recorded_where_the_rules_produce_one() {
    let app = app();
    let id = initiate(&app).await;
    let resp = send(&app, "PUT", &format!("/api/v1/declarations/{id}/tie"), Some(tie_tally())).await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    assert_eq!(body_json(resp).await["status"], "tie_pending");
}

#[tokio::test]
async fn a_tie_pending_declaration_may_never_be_certified() {
    // EC-LEG-06. The aggregate owns this rule; the old handler answered
    // "not_issued" without reading anything, so the rule was unreachable.
    let app = app();
    let id = initiate(&app).await;
    declare(&app, &id, tie_tally()).await;
    let resp = send(&app, "GET", &format!("/api/v1/declarations/{id}/certificate"), None).await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "a tie-pending declaration must not be certified"
    );
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("tie_pending"), "the reason should name the status: {detail}");
}

#[tokio::test]
async fn a_declared_result_may_be_certified() {
    let app = app();
    let id = initiate(&app).await;
    declare(&app, &id, winning_tally()).await;
    let cert = body_json(
        send(&app, "GET", &format!("/api/v1/declarations/{id}/certificate"), None).await,
    )
    .await;
    assert_eq!(cert["status"], "issued");
    assert_eq!(cert["winner_id"], "a");
}

#[tokio::test]
async fn a_certificate_for_an_unknown_declaration_is_404() {
    let app = app();
    let resp = send(&app, "GET", "/api/v1/declarations/no-such/certificate", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_pending_declaration_may_not_be_certified() {
    let app = app();
    let id = initiate(&app).await;
    let resp = send(&app, "GET", &format!("/api/v1/declarations/{id}/certificate"), None).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn an_amendment_must_name_the_tally_it_amends() {
    // The old handler minted an amendment_id and dropped the lineage entirely,
    // so an amendment could be applied on top of any tally.
    let app = app();
    let id = initiate(&app).await;
    declare(&app, &id, winning_tally()).await;

    let wrong = send(
        &app,
        "POST",
        &format!("/api/v1/declarations/{id}/amend"),
        Some(json!({
            "reason": "recount ordered",
            "amended_by": "officer",
            "approved_by": "chair",
            "previous_snapshot_hash": "some-other-snapshot",
            "new_snapshot_hash": "tally-snap-2"
        })),
    )
    .await;
    assert_eq!(wrong.status(), StatusCode::CONFLICT);
    let detail = body_json(wrong).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(
        detail.contains("tally-snap-1"),
        "the reason should name the tally it is on: {detail}"
    );
}

#[tokio::test]
async fn a_valid_amendment_is_recorded_with_append_only_lineage() {
    let app = app();
    let id = initiate(&app).await;
    declare(&app, &id, winning_tally()).await;

    let body = body_json(
        send(
            &app,
            "POST",
            &format!("/api/v1/declarations/{id}/amend"),
            Some(json!({
                "reason": "recount ordered by the High Court",
                "amended_by": "officer",
                "approved_by": "chair",
                "court_order_ref": "HCC-2027-114",
                "previous_snapshot_hash": "tally-snap-1",
                "new_snapshot_hash": "tally-snap-2"
            })),
        )
        .await,
    )
    .await;
    assert_eq!(body["status"], "amended");
    assert_eq!(body["amendment_count"], 1);
    assert!(!body["amendment_id"].is_null());

    // The lineage only grows.
    send(
        &app,
        "POST",
        &format!("/api/v1/declarations/{id}/amend"),
        Some(json!({
            "reason": "second recount",
            "amended_by": "officer",
            "approved_by": "chair",
            "previous_snapshot_hash": "tally-snap-1",
            "new_snapshot_hash": "tally-snap-3"
        })),
    )
    .await;
    let state = body_json(send(&app, "GET", &format!("/api/v1/declarations/{id}"), None).await).await;
    assert_eq!(state["status"], "amended");
}

#[tokio::test]
async fn a_pending_declaration_has_nothing_to_amend() {
    let app = app();
    let id = initiate(&app).await;
    let resp = send(
        &app,
        "POST",
        &format!("/api/v1/declarations/{id}/amend"),
        Some(json!({
            "reason": "r",
            "amended_by": "o",
            "approved_by": "a",
            "previous_snapshot_hash": "tally-snap-1",
            "new_snapshot_hash": "tally-snap-2"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn an_amendment_requires_a_reason_and_a_new_snapshot() {
    let app = app();
    let id = initiate(&app).await;
    declare(&app, &id, winning_tally()).await;
    let resp = send(
        &app,
        "POST",
        &format!("/api/v1/declarations/{id}/amend"),
        Some(json!({
            "reason": "  ",
            "amended_by": "o",
            "approved_by": "a",
            "previous_snapshot_hash": "tally-snap-1",
            "new_snapshot_hash": ""
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn an_unknown_declaration_has_no_petition_window() {
    let app = app();
    let resp = send(&app, "GET", "/api/v1/declarations/no-such/petition-window", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_new_declaration_has_an_unopened_petition_window() {
    let app = app();
    let id = initiate(&app).await;
    let window = body_json(
        send(&app, "GET", &format!("/api/v1/declarations/{id}/petition-window"), None).await,
    )
    .await;
    assert_eq!(window["status"], "unopened");
    assert!(window["window_open_at"].is_null());
}

#[tokio::test]
async fn a_judicial_acknowledgement_requires_a_petition() {
    // Previously any id and any timestamp was accepted.
    let app = app();
    let id = initiate(&app).await;
    let resp = send(
        &app,
        "PUT",
        &format!("/api/v1/declarations/{id}/judiciary-ack"),
        Some(json!({"registrar_id": "registrar-1", "acknowledged_at": "2026-09-01T10:00:00Z"})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "an acknowledgement follows a petition; none was filed"
    );
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("unopened"), "the reason should name the status: {detail}");
}

#[tokio::test]
async fn an_acknowledgement_for_an_unknown_declaration_is_404() {
    let app = app();
    let resp = send(
        &app,
        "PUT",
        "/api/v1/declarations/no-such/judiciary-ack",
        Some(json!({"registrar_id": "r", "acknowledged_at": "2026-09-01T10:00:00Z"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn declarations_do_not_share_state() {
    let a = app();
    let b = app();
    let id = initiate(&a).await;
    let resp = send(&b, "GET", &format!("/api/v1/declarations/{id}"), None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}