//! Round-trip tests for the `/legal-audit/*` surface.
//!
//! The crate is described as "§39 rules engine, per-event compliance verdicts,
//! tlog-hashed (WP-5.1)" and none of the three existed. `list_transmissions` and
//! `list_verdicts` returned `vec![]`, `record_transmission` returned a defaulted
//! record, `get_verdict` returned an **empty** `ComplianceVerdict` for any id,
//! `party_summary` returned zeroed counts, and `generate_report` answered `202
//! Accepted` with no body.
//!
//! The empty verdict was the dangerous shape: `passed` defaults to `false`, so an
//! unevaluated record read as a *failed* compliance check carrying a rule id of
//! `""` — authoritative-looking and meaningless.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_legal_audit::{router::build_router, state::AppState};
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

async fn record(app: &axum::Router, sender: &str, basis: &str) -> Value {
    let resp = send(
        app,
        "POST",
        "/legal-audit/transmissions",
        Some(json!({
            "event_type": "results-transmission",
            "sender": sender,
            "recipient": "IEBC",
            "legal_basis": basis,
            "payload": {"constituency": "047"}
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    body_json(resp).await
}

#[tokio::test]
async fn a_recorded_transmission_is_listed_and_evaluated() {
    let app = app();
    let created = record(&app, "party-a", "s39-elections-act").await;
    let id = created["transmission"]["record_id"].as_str().unwrap().to_string();
    assert!(created["verdicts"].as_array().unwrap().len() > 0);

    let listed = body_json(send(&app, "GET", "/legal-audit/transmissions", None).await).await;
    let transmissions = listed["transmissions"].as_array().unwrap();
    assert_eq!(
        transmissions.len(),
        1,
        "a recorded transmission must be listed; previously always empty"
    );
    assert_eq!(transmissions[0]["record_id"], id.as_str());
}

#[tokio::test]
async fn a_transmission_without_a_legal_basis_fails() {
    // No instrument means no justification, so it cannot be compliant.
    let app = app();
    let created = record(&app, "party-a", "").await;
    let verdicts = created["verdicts"].as_array().unwrap();
    assert!(!verdicts.is_empty());
    assert!(
        verdicts.iter().all(|v| v["passed"] == false),
        "an unjustified transmission cannot pass: {verdicts:?}"
    );
}

#[tokio::test]
async fn an_unknown_legal_basis_is_flagged_rather_than_accepted() {
    let app = app();
    let created = record(&app, "party-a", "invented-instrument").await;
    let verdicts = created["verdicts"].as_array().unwrap();
    assert!(
        verdicts.iter().any(|v| v["rule_id"] == "unrecognised-basis" && v["passed"] == false),
        "{verdicts:?}"
    );
}

#[tokio::test]
async fn a_known_basis_passes() {
    let app = app();
    let created = record(&app, "party-a", "s39-elections-act").await;
    let verdicts = created["verdicts"].as_array().unwrap();
    assert!(
        verdicts.iter().all(|v| v["passed"] == true),
        "a recognised instrument should pass: {verdicts:?}"
    );
    assert!(verdicts.iter().all(|v| !v["section_ref"].as_str().unwrap_or("").is_empty()));
}

#[tokio::test]
async fn a_payload_hash_is_computed_when_the_caller_supplies_none() {
    let app = app();
    let created = record(&app, "party-a", "s39-elections-act").await;
    let hash = created["transmission"]["payload_hash"].as_str().unwrap();
    assert!(
        hash.starts_with("sha256:") && hash.len() > 16,
        "expected a real digest, got {hash:?}"
    );
}

#[tokio::test]
async fn a_caller_supplied_hash_is_preserved() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/legal-audit/transmissions",
        Some(json!({
            "event_type": "results-transmission",
            "sender": "party-a",
            "recipient": "IEBC",
            "legal_basis": "s39-elections-act",
            "payload_hash": "sha256:already-computed-upstream"
        })),
    )
    .await;
    assert_eq!(
        body_json(resp).await["transmission"]["payload_hash"],
        "sha256:already-computed-upstream"
    );
}

#[tokio::test]
async fn a_recorded_transmission_requires_its_parties() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/legal-audit/transmissions",
        Some(json!({"event_type": "", "sender": "p", "recipient": "IEBC"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn an_unrecorded_record_has_no_verdict() {
    // Previously any id returned an empty ComplianceVerdict, whose `passed:
    // false` read as a failed check with no rule.
    let app = app();
    let resp = send(&app, "GET", "/legal-audit/transmissions/no-such/verdict", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_record_returns_only_its_own_verdicts() {
    let app = app();
    let first = record(&app, "party-a", "s39-elections-act").await;
    let second = record(&app, "party-b", "").await;
    let id_b = second["transmission"]["record_id"].as_str().unwrap().to_string();
    let id_a = first["transmission"]["record_id"].as_str().unwrap().to_string();

    let b = body_json(
        send(&app, "GET", &format!("/legal-audit/transmissions/{id_b}/verdict"), None).await,
    )
    .await;
    let verdicts = b["verdicts"].as_array().unwrap();
    assert!(!verdicts.is_empty());
    assert!(verdicts.iter().all(|v| v["record_id"] == id_b.as_str()));
    assert!(verdicts.iter().all(|v| v["passed"] == false), "party-b has no basis");

    let a = body_json(
        send(&app, "GET", &format!("/legal-audit/transmissions/{id_a}/verdict"), None).await,
    )
    .await;
    assert!(a["verdicts"].as_array().unwrap().iter().all(|v| v["passed"] == true));
}

#[tokio::test]
async fn the_verdict_chain_links_every_verdict_in_order() {
    // Each verdict commits to its predecessor, so the chain head differs per
    // position and reordering is detectable.
    let app = app();
    record(&app, "party-a", "s39-elections-act").await;
    record(&app, "party-b", "s39-elections-act").await;

    let listed = body_json(send(&app, "GET", "/legal-audit/verdicts", None).await).await;
    let verdicts = listed["verdicts"].as_array().unwrap();
    assert!(
        verdicts.len() >= 2,
        "both records must contribute verdicts: {verdicts:?}"
    );
    let hashes: Vec<&str> = verdicts.iter().map(|v| v["verdict_hash"].as_str().unwrap()).collect();
    let unique: std::collections::HashSet<&&str> = hashes.iter().collect();
    assert_eq!(unique.len(), hashes.len(), "every verdict hash must be unique");

    assert_eq!(
        listed["chain_head"], *hashes.last().unwrap(),
        "the chain head is the last verdict's hash"
    );
}

#[tokio::test]
async fn a_party_summary_counts_only_evaluated_transmissions() {
    let app = app();
    record(&app, "party-a", "s39-elections-act").await;
    record(&app, "party-a", "").await;

    let summary = body_json(
        send(&app, "GET", "/legal-audit/parties/party-a/summary", None).await,
    )
    .await;
    assert_eq!(summary["total_transmissions"], 2);
    assert_eq!(summary["compliant"], 1, "one transmission has a valid basis");
    assert_eq!(summary["non_compliant"], 1);
}

#[tokio::test]
async fn a_party_summary_for_an_unknown_party_is_404() {
    // Previously returned a zeroed summary, which reads as "compliant".
    let app = app();
    let resp = send(&app, "GET", "/legal-audit/parties/nobody/summary", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn summaries_do_not_mix_parties() {
    let app = app();
    record(&app, "party-a", "s39-elections-act").await;
    record(&app, "party-b", "").await;
    let b = body_json(send(&app, "GET", "/legal-audit/parties/party-b/summary", None).await).await;
    assert_eq!(b["total_transmissions"], 1);
    assert_eq!(b["non_compliant"], 1);
}

#[tokio::test]
async fn the_report_is_derived_from_what_was_stored() {
    // Previously 202 with no body: nothing distinguished a completed audit from
    // one that had not run.
    let app = app();
    record(&app, "party-a", "s39-elections-act").await;
    record(&app, "party-b", "").await;

    let report = body_json(send(&app, "POST", "/legal-audit/reports", None).await).await;
    let s = &report["compliance_summary"];
    assert_eq!(s["transmissions"], 2);
    assert_eq!(s["verdicts_failed"], s["verdicts_failed"].as_u64().unwrap());
    assert!(s["verdicts_failed"].as_u64().unwrap() > 0, "party-b fails every rule");
    assert_eq!(
        s["transmissions_without_verdicts"], 0,
        "every stored transmission was evaluated"
    );
    assert_eq!(
        s["chain_verifies"], true,
        "the report re-derives the chain and it must verify: {s:?}"
    );
    assert!(!report["findings"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn the_report_never_claims_tlog_inclusion() {
    let app = app();
    record(&app, "party-a", "s39-elections-act").await;
    let report = body_json(send(&app, "POST", "/legal-audit/reports", None).await).await;
    assert_eq!(
        report["compliance_summary"]["tlog_inclusion_proven"], false,
        "there is no append-only log here, so inclusion must never be claimed"
    );
}

#[tokio::test]
async fn an_empty_store_produces_an_empty_but_honest_report() {
    let app = app();
    let report = body_json(send(&app, "POST", "/legal-audit/reports", None).await).await;
    assert_eq!(report["compliance_summary"]["transmissions"], 0);
    assert_eq!(report["compliance_summary"]["verdicts"], 0);
    assert_eq!(report["compliance_summary"]["chain_head"], "genesis");
    assert!(
        report["findings"].as_array().unwrap().is_empty(),
        "no findings is correct when nothing was recorded"
    );
}

#[tokio::test]
async fn routers_do_not_share_state() {
    let a = app();
    let b = app();
    record(&a, "party-a", "s39-elections-act").await;
    let listed = body_json(send(&b, "GET", "/legal-audit/transmissions", None).await).await;
    assert!(listed["transmissions"].as_array().unwrap().is_empty());
    let resp = send(&b, "GET", "/legal-audit/parties/party-a/summary", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}