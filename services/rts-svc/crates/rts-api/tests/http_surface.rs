//! Round-trip tests for the RTS HTTP surface.
//!
//! The one that mattered most: `get_form_proof` returned `merkle_root:
//! "0".repeat(64)` — the *zero root*, which is not a Merkle root of anything —
//! together with an empty proof. `packages/eel-crypto-rs` has a real `MerkleTree`
//! with `inclusion_proof` and `verify_inclusion_against_root`, and it was not
//! used. A client verifying the proof would reject it; a client that only checked
//! "is a root present" would accept it.
//!
//! Also: `get_icr_job` minted a **new** `job_id` on every read, so a client
//! polling one form's job could never correlate two polls. `get_ack` answered
//! `pending` for any scan id, so a submitted-and-scanned form read as never
//! received. `get_snapshot` returned every field null. `provision_token` returned
//! a `rts-token-stub-<uuid>` with a hardcoded `expires_at` of 2030. And
//! `resolve_dispute` answered `resolved` for any id, never reading the
//! `countersign_token` the request carries.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use rts_api::{router::build_router, store::AppState};
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

/// A consistent Form 34A: 400 valid, 20 rejected, 5 spoilt, 425 cast, 500
/// registered. Candidates 300 + 100 = 400 valid.
fn form34a(station: &str, constituency: &str) -> Value {
    json!({
        "station_code": station,
        "constituency_code": constituency,
        "county_code": "047",
        "registered_voters": 500,
        "votes_cast": 425,
        "valid_votes": 400,
        "rejected_votes": 20,
        "spoilt_ballots": 5,
        "candidates": [
            {"candidate_id": "c1", "candidate_name": "A", "votes": 300},
            {"candidate_id": "c2", "candidate_name": "B", "votes": 100}
        ],
        "device_signature": "ed25519:AAAA"
    })
}

async fn submit(app: &axum::Router, station: &str, constituency: &str) -> Uuidish {
    let resp = send(
        app,
        "POST",
        "/api/v1/rts/forms",
        Some(form34a(station, constituency)),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED, "a consistent form is accepted");
    let body = body_json(resp).await;
    assert_eq!(body["status"], "accepted");
    body["form_id"].as_str().unwrap().to_string()
}

type Uuidish = String;

#[tokio::test]
async fn a_form_34a_whose_numbers_disagree_is_rejected() {
    let app = app();
    // Candidates sum to 400 but valid_votes says 500. votes_cast is left
    // consistent with the other two figures so the candidate check is the one
    // that fires.
    let mut bad = form34a("STN001", "047");
    bad["valid_votes"] = json!(500);
    bad["votes_cast"] = json!(525);
    bad["registered_voters"] = json!(600);
    let resp = send(&app, "POST", "/api/v1/rts/forms", Some(bad)).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(
        detail.contains("candidate votes total"),
        "the reason should name the mismatch: {detail}"
    );
}

#[tokio::test]
async fn votes_cast_must_reconcile_against_valid_rejected_and_spoilt() {
    let app = app();
    let mut bad = form34a("STN001", "047");
    bad["votes_cast"] = json!(10);
    let resp = send(&app, "POST", "/api/v1/rts/forms", Some(bad)).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("votes_cast"), "{detail}");
}

#[tokio::test]
async fn votes_cast_cannot_exceed_registered_voters() {
    let app = app();
    let mut bad = form34a("STN001", "047");
    bad["registered_voters"] = json!(10);
    let resp = send(&app, "POST", "/api/v1/rts/forms", Some(bad)).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_form_requires_its_station_and_area() {
    let app = app();
    let mut bad = form34a("", "047");
    bad["station_code"] = json!("  ");
    let resp = send(&app, "POST", "/api/v1/rts/forms", Some(bad)).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_form_proof_carries_a_real_root_that_verifies() {
    // The headline. The old code returned the zero root with an empty proof.
    let app = app();
    let a = submit(&app, "STN001", "047").await;
    let b = submit(&app, "STN002", "047").await;
    let c = submit(&app, "STN003", "001").await;

    // The leaf set, read back from the audit trail in arrival order.
    let mut leaves: Vec<Vec<u8>> = Vec::new();
    for id in [&a, &b, &c] {
        let audit = body_json(send(&app, "GET", &format!("/api/v1/rts/audit/{id}"), None).await).await;
        leaves.push(audit["leaf_hash"].as_str().unwrap().as_bytes().to_vec());
    }
    let refs: Vec<&[u8]> = leaves.iter().map(|l| l.as_slice()).collect();
    let tree = eel_crypto_rs::merkle::MerkleTree::build(&refs);
    let expected_root = hex::encode(tree.root());

    for (id, leaf) in [(&a, &leaves[0]), (&b, &leaves[1]), (&c, &leaves[2])] {
        let proof = body_json(
            send(&app, "GET", &format!("/api/v1/rts/forms/{id}/proof"), None).await,
        )
        .await;
        let root = proof["merkle_root"].as_str().unwrap();
        assert_ne!(root, "0".repeat(64), "the zero root is not a Merkle root of anything");
        assert_eq!(root, expected_root, "every proof reports the same root");
        assert_eq!(root.len(), 64);
        assert_eq!(proof["tree_size"], 3);
        assert!(!proof["merkle_proof"].as_array().unwrap().is_empty());

        // The served proof verifies against the served root with the crate's own
        // verifier.
        let path: Vec<[u8; 32]> = proof["merkle_proof"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                let bytes = hex::decode(p.as_str().unwrap()).unwrap();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                arr
            })
            .collect();
        assert!(
            tree.verify_inclusion(proof["leaf_index"].as_u64().unwrap() as usize, leaf.as_slice(), &path)
                .is_ok(),
            "the served proof for {id} must verify against the served root"
        );
    }
}

#[tokio::test]
async fn each_form_gets_a_distinct_leaf_hash() {
    let app = app();
    let a = submit(&app, "STN001", "047").await;
    let b = submit(&app, "STN002", "047").await;
    let ha = body_json(send(&app, "GET", &format!("/api/v1/rts/audit/{a}"), None).await).await;
    let hb = body_json(send(&app, "GET", &format!("/api/v1/rts/audit/{b}"), None).await).await;
    assert_ne!(
        ha["leaf_hash"], hb["leaf_hash"],
        "two different stations must not collapse to one leaf"
    );
}

#[tokio::test]
async fn a_proof_does_not_claim_transparency_log_inclusion() {
    // A real root over this process's forms is not an append-only-log proof.
    let app = app();
    let id = submit(&app, "STN001", "047").await;
    let proof = body_json(
        send(&app, "GET", &format!("/api/v1/rts/forms/{id}/proof"), None).await,
    )
    .await;
    assert_eq!(proof["tlog_inclusion_proven"], false);
}

#[tokio::test]
async fn a_proof_for_an_unknown_form_is_404() {
    // Previously any form id got the zero root.
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/api/v1/rts/forms/11111111-1111-1111-1111-111111111111/proof",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_proof_with_no_forms_at_all_is_unavailable() {
    // An empty tree has no root worth publishing; that is the zero-root problem
    // again in a different guise.
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/api/v1/rts/forms/11111111-1111-1111-1111-111111111111/proof",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_submitted_form_is_acknowledged() {
    // Previously get_ack answered `pending` for any scan id, so a submitted and
    // scanned form read as never received.
    let app = app();
    let id = submit(&app, "STN001", "047").await;
    let audit = body_json(send(&app, "GET", &format!("/api/v1/rts/audit/{id}"), None).await).await;
    let scan_id = audit["scan_id"].as_str().unwrap().to_string();

    let ack = body_json(send(&app, "GET", &format!("/api/v1/rts/acks/{scan_id}"), None).await).await;
    assert_eq!(ack["status"], "received");
    assert_eq!(ack["form_id"], id.as_str());
}

#[tokio::test]
async fn an_unacknowledged_scan_is_404() {
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/api/v1/rts/acks/22222222-2222-2222-2222-222222222222",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_icr_job_id_is_stable_across_reads() {
    // The old handler minted a *new* job_id on every read, so a client polling
    // one form could never correlate two polls.
    let app = app();
    let id = submit(&app, "STN001", "047").await;
    let first = body_json(send(&app, "GET", &format!("/api/v1/rts/icr/{id}"), None).await).await;
    let second = body_json(send(&app, "GET", &format!("/api/v1/rts/icr/{id}"), None).await).await;
    assert_eq!(first["job_id"], second["job_id"]);
    assert_eq!(first["status"], "pending");
    assert_eq!(first["form_id"], id.as_str());
}

#[tokio::test]
async fn an_icr_job_for_an_unknown_form_is_404() {
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/api/v1/rts/icr/11111111-1111-1111-1111-111111111111",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_icr_result_for_a_job_with_no_extraction_is_404() {
    // Previously any job id answered `pending` with no fields, forever.
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/api/v1/rts/icr/11111111-1111-1111-1111-111111111111/result",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_constituency_snapshot_consolidates_the_forms_it_received() {
    // Previously get_snapshot returned merkle_root: None and form34c: None for
    // any scope/code.
    let app = app();
    submit(&app, "STN001", "047").await;
    submit(&app, "STN002", "047").await;

    let snap = body_json(send(&app, "GET", "/api/v1/rts/snapshots/constituency/047", None).await).await;
    let form34c = &snap["form34c"];
    assert_eq!(form34c["constituency_code"], "047");
    assert_eq!(form34c["stations_reporting"].as_array().unwrap().len(), 2);
    assert_eq!(form34c["registered_voters"], 1000, "500 from each station");
    assert_eq!(form34c["votes_cast"], 850);
    assert_eq!(form34c["valid_votes"], 800);
    // Candidate c1: 300 from each station.
    let c1 = form34c["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["candidate_id"] == "c1")
        .unwrap();
    assert_eq!(c1["votes"], 600);
    assert!(snap["merkle_root"].is_string());
}

#[tokio::test]
async fn a_snapshot_does_not_leak_across_constituencies() {
    let app = app();
    submit(&app, "STN001", "047").await;
    let resp = send(&app, "GET", "/api/v1/rts/snapshots/constituency/001", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_unknown_snapshot_scope_is_404() {
    let app = app();
    submit(&app, "STN001", "047").await;
    let resp = send(&app, "GET", "/api/v1/rts/snapshots/county/047", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn snapshots_are_listed_for_the_constituencies_that_reported() {
    // Previously always an empty list.
    let app = app();
    submit(&app, "STN001", "047").await;
    submit(&app, "STN002", "001").await;
    let listed = body_json(send(&app, "GET", "/api/v1/rts/snapshots", None).await).await;
    let snaps = listed["snapshots"].as_array().unwrap();
    assert_eq!(snaps.len(), 2);
    assert_eq!(snaps[0]["code"], "001", "sorted by code");
    assert_eq!(snaps[1]["code"], "047");
    assert_eq!(listed["tlog_inclusion_proven"], false);
}

#[tokio::test]
async fn a_token_is_bound_to_its_request_and_expires() {
    // The old code returned `rts-token-stub-<uuid>` with a hardcoded 2030 expiry.
    let app = app();
    let issued = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/tokens",
            Some(json!({"device_id": "device-1", "svid": "svid-abc"})),
        )
        .await,
    )
    .await;
    let token = issued["token"].as_str().unwrap().to_string();
    assert!(token.starts_with("rts-token-sha256:"), "got {token}");
    assert_ne!(token, "rts-token-stub");

    let expires = chrono::DateTime::parse_from_rfc3339(issued["expires_at"].as_str().unwrap()).unwrap();
    let hours = (expires.with_timezone(&chrono::Utc) - chrono::Utc::now()).num_hours();
    assert!(
        (23..=24).contains(&hours),
        "the token should expire in about 24h, not in 2030: {hours}h"
    );

    // Each issuance is its own capability, so two calls are two tokens.
    let again = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/tokens",
            Some(json!({"device_id": "device-1", "svid": "svid-abc"})),
        )
        .await,
    )
    .await;
    assert_ne!(again["token"], issued["token"]);

    // A different svid yields a different token.
    let other = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/tokens",
            Some(json!({"device_id": "device-1", "svid": "svid-xyz"})),
        )
        .await,
    )
    .await;
    assert_ne!(other["token"], issued["token"]);
}

#[tokio::test]
async fn a_token_requires_a_device_and_svid() {
    let app = app();
    let resp = send(&app, "POST", "/api/v1/rts/tokens", Some(json!({"device_id": "  ", "svid": "s"}))).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn an_ingested_event_is_stored_and_identified() {
    // Previously answered `queued` for anything and stored nothing.
    let app = app();
    let first = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/events",
            Some(json!({"envelope": {"id": "evt-1", "type": "eel.rts.tally.v1"}})),
        )
        .await,
    )
    .await;
    assert_eq!(first["status"], "accepted");
    assert!(!first["event_id"].is_null());

    let second = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/events",
            Some(json!({"envelope": {"id": "evt-2", "type": "eel.rts.tally.v1"}})),
        )
        .await,
    )
    .await;
    assert_ne!(
        second["event_id"], first["event_id"],
        "two events are two events"
    );
}

#[tokio::test]
async fn an_event_envelope_needs_an_id_and_a_type() {
    let app = app();
    for envelope in [
        json!({"type": "eel.rts.tally.v1"}),
        json!({"id": "evt-1"}),
        json!({"id": "  ", "type": "  "}),
    ] {
        let resp = send(&app, "POST", "/api/v1/rts/events", Some(json!({"envelope": envelope}))).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn an_sms_payload_is_stored_verbatim_not_turned_into_a_form() {
    let app = app();
    let resp = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/forms/sms-fallback",
            Some(json!({"encoded_payload": "RTS1|BASE64...", "device_id": "device-1", "sequence_number": 7})),
        )
        .await,
    )
    .await;
    assert_eq!(resp["status"], "received");
    assert_eq!(resp["sequence_number"], 7);
    let reason = resp["blocking_reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("does not parse"),
        "an unparsed payload must not become a form: {reason:?}"
    );
    // No form was fabricated from it.
    let listed = body_json(send(&app, "GET", "/api/v1/rts/snapshots", None).await).await;
    assert!(listed["snapshots"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn an_sms_payload_must_not_be_empty() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/rts/forms/sms-fallback",
        Some(json!({"encoded_payload": "  ", "device_id": "d", "sequence_number": 1})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_compilation_job_is_recorded_and_says_why_it_cannot_finish() {
    // Previously both returned `compilation_queued` for any input and recorded
    // nothing at all.
    let app = app();
    let job = body_json(
        send(&app, "POST", "/api/v1/rts/forms/34c", Some(json!({"scope": "047"}))).await,
    )
    .await;
    assert_eq!(job["form_kind"], "34C");
    assert_eq!(job["status"], "queued");
    assert!(!job["job_id"].is_null());
    let reason = job["blocking_reason"].as_str().unwrap_or_default();
    assert!(reason.contains("HSM"), "the reason should name what is absent: {reason:?}");

    let other = body_json(
        send(&app, "POST", "/api/v1/rts/forms/34b", Some(json!({"scope": "047"}))).await,
    )
    .await;
    assert_ne!(other["job_id"], job["job_id"]);
}

#[tokio::test]
async fn a_dispute_needs_a_real_form() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/rts/ro/disputes",
        Some(json!({
            "form_id": "11111111-1111-1111-1111-111111111111",
            "reason": "figures disagree",
            "ro_id": "ro-1"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_dispute_resolution_requires_a_countersign_token_this_service_issued() {
    // The old handler answered `resolved` for any id and never read the token.
    let app = app();
    let id = submit(&app, "STN001", "047").await;
    let dispute = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/ro/disputes",
            Some(json!({"form_id": id, "reason": "figures disagree", "ro_id": "ro-1"})),
        )
        .await,
    )
    .await;
    let dispute_id = dispute["dispute_id"].as_str().unwrap().to_string();

    let forged = send(
        &app,
        "POST",
        &format!("/api/v1/rts/ro/disputes/{dispute_id}/resolve"),
        Some(json!({"resolution": "upheld", "countersign_token": "rts-token-made-up"})),
    )
    .await;
    assert_eq!(forged.status(), StatusCode::FORBIDDEN);

    let unknown = send(
        &app,
        "POST",
        &format!("/api/v1/rts/ro/disputes/{dispute_id}/resolve"),
        Some(json!({"resolution": "upheld", "countersign_token": "rts-token-sha256:0000"})),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn the_countersigning_officers_own_token_resolves_the_dispute() {
    let app = app();
    let id = submit(&app, "STN001", "047").await;
    let dispute = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/ro/disputes",
            Some(json!({"form_id": id, "reason": "figures disagree", "ro_id": "ro-1"})),
        )
        .await,
    )
    .await;
    let dispute_id = dispute["dispute_id"].as_str().unwrap().to_string();
    let token = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/tokens",
            Some(json!({"device_id": "ro-1", "svid": "svid-ro"})),
        )
        .await,
    )
    .await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let resolved = body_json(
        send(
            &app,
            "POST",
            &format!("/api/v1/rts/ro/disputes/{dispute_id}/resolve"),
            Some(json!({"resolution": "upheld", "countersign_token": token})),
        )
        .await,
    )
    .await;
    assert_eq!(resolved["status"], "resolved");
    assert_eq!(resolved["resolution"], "upheld");

    let again = send(
        &app,
        "POST",
        &format!("/api/v1/rts/ro/disputes/{dispute_id}/resolve"),
        Some(json!({"resolution": "overturned", "countersign_token": token})),
    )
    .await;
    assert_eq!(again.status(), StatusCode::CONFLICT, "a dispute resolves once");
}

#[tokio::test]
async fn a_ro_token_cannot_countersign_another_officers_dispute() {
    let app = app();
    let id = submit(&app, "STN001", "047").await;
    let dispute = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/ro/disputes",
            Some(json!({"form_id": id, "reason": "r", "ro_id": "ro-2"})),
        )
        .await,
    )
    .await;
    let dispute_id = dispute["dispute_id"].as_str().unwrap().to_string();
    let wrong_token = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/tokens",
            Some(json!({"device_id": "ro-1", "svid": "svid-ro"})),
        )
        .await,
    )
    .await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = send(
        &app,
        "POST",
        &format!("/api/v1/rts/ro/disputes/{dispute_id}/resolve"),
        Some(json!({"resolution": "upheld", "countersign_token": wrong_token})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "the countersigning officer's own token is required"
    );
}

#[tokio::test]
async fn resolving_an_unknown_dispute_is_404() {
    let app = app();
    let token = body_json(
        send(
            &app,
            "POST",
            "/api/v1/rts/tokens",
            Some(json!({"device_id": "ro-1", "svid": "s"})),
        )
        .await,
    )
    .await["token"]
        .as_str()
        .unwrap()
        .to_string();
    let resp = send(
        &app,
        "POST",
        "/api/v1/rts/ro/disputes/33333333-3333-3333-3333-333333333333/resolve",
        Some(json!({"resolution": "r", "countersign_token": token})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_audit_trail_exists_for_a_received_form() {
    // Previously every audit returned `events: []`.
    let app = app();
    let id = submit(&app, "STN001", "047").await;
    let audit = body_json(send(&app, "GET", &format!("/api/v1/rts/audit/{id}"), None).await).await;
    assert_eq!(audit["form_id"], id.as_str());
    assert_eq!(audit["station_code"], "STN001");
    assert!(!audit["received_at"].is_null());
    assert!(!audit["leaf_hash"].is_null());
    assert_eq!(audit["events"].as_array().unwrap().len(), 1);

    let missing = send(
        &app,
        "GET",
        "/api/v1/rts/audit/11111111-1111-1111-1111-111111111111",
        None,
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn routers_do_not_share_state() {
    let a = app();
    let b = app();
    let id = submit(&a, "STN001", "047").await;
    let resp = send(&b, "GET", &format!("/api/v1/rts/audit/{id}"), None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let snaps = body_json(send(&b, "GET", "/api/v1/rts/snapshots", None).await).await;
    assert!(snaps["snapshots"].as_array().unwrap().is_empty());
}
