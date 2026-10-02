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

//! The BVR register, read and written.
//!
//! Four handlers in this crate had no register to read, because `AppState` was an
//! IPRS URL and nothing else. The tables have been in
//! `services/bvr-svc/migrations/` the whole time: `voters` (V1),
//! `audit_voters_history` (V2), `merkle_roots` and `register_snapshots` (V3),
//! `sar_requests` (V4).
//!
//! Each test below names the behaviour it replaced.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use bvr_api::{
    state::AppState,
    store::{BvrStore, MerkleRootResponse, VoterSummary},
};

const OFFICER: &str = "018f6c2e-0000-7000-8000-00000000abcd";
const VOTER: &str = "018f6c2e-0000-7000-8000-000000000001";

fn app() -> (axum::Router, std::sync::Arc<BvrStore>) {
    let store = BvrStore::in_memory();
    store.put_voter(VoterSummary {
        voter_id: VOTER.to_string(),
        polling_station_code: "NRB-001-PS-042".to_string(),
        status: "active".to_string(),
        version: 3,
        election_scope: "2027-general".to_string(),
    });
    let state = AppState::new("http://iprs.invalid", store.clone());
    (bvr_api::app(state), store)
}

async fn call(
    router: axum::Router,
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
    let resp = router.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn a_voter_is_read_from_the_register() {
    // Previously: full_name "Placeholder Voter", county 047, station 047-001-001,
    // status active -- for any voter id. It invented a person's name and where they
    // vote. The contract's VoterSummary has no name at all, and `full_name_enc` is
    // a field-encrypted blob, so there was no name to return anyway.
    let (router, _store) = app();
    let (status, body) = call(router, "GET", &format!("/api/v1/bvr/voters/{VOTER}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["polling_station_code"], "NRB-001-PS-042");
    assert_eq!(body["status"], "active");
    assert_eq!(body["version"], 3);
    assert!(body.get("full_name").is_none(), "this endpoint must not return a name");
}

#[tokio::test]
async fn a_voter_not_on_the_register_is_not_found() {
    let (router, _store) = app();
    let (status, _) = call(router, "GET", "/api/v1/bvr/voters/nobody", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_status_change_actually_happens_and_is_recorded() {
    // Previously: logged and returned 204, so the change was acknowledged and never
    // made. The contract asks for 200 with the updated VoterSummary.
    let (router, store) = app();
    let (status, body) = call(
        router,
        "PATCH",
        &format!("/api/v1/bvr/voters/{VOTER}/status"),
        Some(json!({
            "status": "suspended",
            "reason_code": "court_order",
            "reason_detail": "Stay on the register pending the case.",
            "officer_id": OFFICER
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "suspended");
    // The version moves, so a caller can see the change happened.
    assert_eq!(body["version"], 4);

    // And the state really changed, not just the response.
    let stored = store.voter(VOTER).await.unwrap().unwrap();
    assert_eq!(stored.status, "suspended");
    assert_eq!(stored.version, 4);

    // The contract says these transitions are event-sourced, so there is a record
    // of who made it and from what.
    let history = store.history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].from_status, "active");
    assert_eq!(history[0].to_status, "suspended");
    assert_eq!(history[0].reason_code, "court_order");
    assert_eq!(history[0].officer_id, OFFICER);
    assert_eq!(history[0].version_from, 3);
    assert_eq!(history[0].version_to, 4);
}

#[tokio::test]
async fn a_status_change_off_the_contract_enum_is_refused() {
    let (router, _store) = app();
    for (status, code) in [
        ("deleted", "court_order"),
        ("suspended", "because_i_said_so"),
        ("", "court_order"),
    ] {
        let (code_status, _) = call(
            router.clone(),
            "PATCH",
            &format!("/api/v1/bvr/voters/{VOTER}/status"),
            Some(json!({ "status": status, "reason_code": code, "officer_id": OFFICER })),
        )
        .await;
        assert_eq!(code_status, StatusCode::BAD_REQUEST, "{status}/{code} must be refused");
    }
}

#[tokio::test]
async fn a_status_change_to_an_unknown_voter_is_not_found() {
    let (router, _store) = app();
    let (status, _) = call(
        router,
        "PATCH",
        "/api/v1/bvr/voters/nobody/status",
        Some(json!({ "status": "suspended", "reason_code": "court_order", "officer_id": OFFICER })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_sar_is_recorded_rather_than_only_ticketed() {
    // Previously: minted a sar_id and answered 202, leaving a ticket for a request
    // that was never stored.
    let (router, store) = app();
    let hash = "a".repeat(64);
    let (status, body) = call(
        router,
        "POST",
        "/api/v1/bvr/sar",
        Some(json!({
            "voter_id": VOTER,
            "requestor_national_id_hash": hash,
            "delivery_channel": "email",
            "requestor_email": "voter@example.invalid"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["status"], "received");
    // 21 days from submission, per the Kenya Data Protection Act 2019.
    let due = body["due_date"].as_str().unwrap();
    assert_eq!(due.len(), 10, "{due}");

    let sars = store.sars();
    assert_eq!(sars.len(), 1, "the request must be recorded, not just acknowledged");
    assert_eq!(sars[0].requestor_national_id_hash, hash);
}

#[tokio::test]
async fn a_sar_never_stores_the_raw_national_id() {
    // The old handler took `requester_national_id` in the clear and wrote it to
    // the ticket table. The contract asks for a hash.
    let (router, _store) = app();
    let (status, _) = call(
        router,
        "POST",
        "/api/v1/bvr/sar",
        Some(json!({
            "voter_id": VOTER,
            "requester_national_id": "12345678",
            "delivery_channel": "email"
        })),
    )
    .await;
    // The field is not in the request schema, so deserialization rejects it.
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_sar_rejects_a_malformed_hash_or_channel() {
    let (router, _store) = app();
    for body in [
        json!({ "voter_id": "v", "requestor_national_id_hash": "12345678", "delivery_channel": "email" }),
        json!({ "voter_id": "v", "requestor_national_id_hash": "a".repeat(64), "delivery_channel": "carrier_pigeon" }),
        json!({ "voter_id": "v", "requestor_national_id_hash": "a".repeat(64), "delivery_channel": "email", "requestor_email": "not-an-address" }),
    ] {
        let (status, _) = call(router.clone(), "POST", "/api/v1/bvr/sar", Some(body)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn an_unpublished_merkle_root_is_not_a_zero_root() {
    // Previously: "0".repeat(64) with seq 0 -- the root of an empty tree presented
    // as the root of the register, and a root is what a client checks proofs
    // against.
    let (router, _store) = app();
    let (status, body) = call(router, "GET", "/api/v1/bvr/merkle-root/latest", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("64 zeros"));
}

#[tokio::test]
async fn an_unsigned_merkle_root_says_so_and_still_returns_the_root() {
    let (router, _store) = app();
    _store.put_merkle_root(MerkleRootResponse {
        root_hash: "f".repeat(64),
        tree_size: 22_500_000,
        timestamp: "2027-08-21T00:00:00Z".to_string(),
        signature: None,
        snapshot_id: Some("SNAP-1".to_string()),
        election_scope: "2027-general".to_string(),
    });
    let (status, body) = call(router, "GET", "/api/v1/bvr/merkle-root/latest", None).await;
    // The contract requires a signature over (root_hash || tree_size || timestamp)
    // for a root to be a verification anchor. No HSM has signed.
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body["error"].as_str().unwrap().contains("unsigned"));
    // The root itself is real, so it is returned rather than withheld.
    assert_eq!(body["root"]["root_hash"], "f".repeat(64));
    assert_eq!(body["root"]["tree_size"], 22_500_000);
}

#[tokio::test]
async fn a_signed_merkle_root_is_returned_in_full() {
    let (router, _store) = app();
    _store.put_merkle_root(MerkleRootResponse {
        root_hash: "e".repeat(64),
        tree_size: 22_500_000,
        timestamp: "2027-08-21T00:00:00Z".to_string(),
        signature: Some("c2lnbmF0dXJl".to_string()),
        snapshot_id: Some("SNAP-1".to_string()),
        election_scope: "2027-general".to_string(),
    });
    let (status, body) = call(router, "GET", "/api/v1/bvr/merkle-root/latest", None).await;
    assert_eq!(status, StatusCode::OK);
    for required in ["root_hash", "tree_size", "timestamp", "signature", "snapshot_id"] {
        assert!(body.get(required).is_some(), "missing {required}");
    }
}

#[tokio::test]
async fn a_handoff_token_is_not_redeemed_without_being_checked() {
    // Previously: status "redeemed" for any token, telling a caller their token had
    // been consumed when nothing was verified and nothing was recorded.
    let (router, _store) = app();
    let (status, body) = call(router, "POST", "/api/v1/bvr/handoff/whatever", None).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert!(body["error"].as_str().unwrap().contains("signed by the citizen portal"));
}
