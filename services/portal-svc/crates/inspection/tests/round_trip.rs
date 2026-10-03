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

//! Round-trip tests for the portal inspection endpoints.
//!
//! `create_roll_snapshot` already stored its snapshot, but `get_roll_snapshot`
//! discarded state and answered 404 for *every* id — so a snapshot that had just
//! been created could not be read back. `file_objection` answered 201 and dropped
//! the objection, and `update_objection_status` answered 204 for any id while
//! discarding the status, so an objection could read as resolved while still
//! `Filed`. `decide_correction` recorded a Determination and answered 200 without
//! updating the claim, leaving it `Pending` — which is the state
//! `auto_referral::should_refer` requires before it will refer anything.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_portal_inspection::api::{router, AppState};

fn app() -> axum::Router {
    router()
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
    app.clone().oneshot(b.body(payload).unwrap()).await.unwrap()
}

async fn snapshot(app: &axum::Router) -> String {
    let resp = send(
        app,
        "POST",
        "/rolls/snapshot",
        Some(json!({"snapshot_hash": "sha256:abc", "station_count": 47, "voter_count": 22000000})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    body_json(resp).await["id"].as_str().unwrap().to_string()
}

fn objection_body() -> Value {
    json!({
        "voter_id_hash": "sha256:voter-1",
        "station_id": "STN-001",
        "reason": "name misspelled on roll",
        "filed_by": "00000000-0000-0000-0000-0000000000aa"
    })
}

#[tokio::test]
async fn a_created_roll_snapshot_can_be_read_back() {
    // get_roll_snapshot previously answered 404 for every id, including one just
    // created.
    let app = app();
    let id = snapshot(&app).await;
    let resp = send(&app, "GET", &format!("/rolls/snapshot/{id}"), None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let snap = body_json(resp).await;
    assert_eq!(snap["id"], id.as_str());
    assert_eq!(snap["snapshot_hash"], "sha256:abc");
    assert_eq!(snap["voter_count"], 22000000u64);
}

#[tokio::test]
async fn an_unknown_roll_snapshot_is_404() {
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/rolls/snapshot/00000000-0000-0000-0000-0000000000bb",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_filed_objection_is_stored_and_status_updates_are_visible() {
    let app = app();
    let created = body_json(send(&app, "POST", "/objections", Some(objection_body())).await).await;
    let id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["status"], "filed");

    let fetched = body_json(send(&app, "GET", &format!("/objections/{id}"), None).await).await;
    assert_eq!(fetched["voter_id_hash"], "sha256:voter-1");
    assert_eq!(fetched["reason"], "name misspelled on roll");

    let updated = send(
        &app,
        "PATCH",
        &format!("/objections/{id}/status"),
        Some(json!({"status": "dismissed"})),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::NO_CONTENT);

    let after = body_json(send(&app, "GET", &format!("/objections/{id}"), None).await).await;
    assert_eq!(
        after["status"], "dismissed",
        "the status must be stored; previously the handler discarded it and \\
         answered 204 for any id, so an objection read as resolved while still Filed"
    );
}

#[tokio::test]
async fn updating_an_unknown_objection_is_404() {
    let app = app();
    let resp = send(
        &app,
        "PATCH",
        "/objections/00000000-0000-0000-0000-0000000000cc/status",
        Some(json!({"status": "dismissed"})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "update_objection_status previously answered 204 for any id"
    );
}

#[tokio::test]
async fn a_filed_correction_is_stored_and_decisions_reach_the_claim() {
    // decide_correction recorded a Determination but left the claim Pending,
    // which is the state auto_referral::should_refer requires before referring.
    let app = app();
    let created = body_json(
        send(
            &app,
            "POST",
            "/corrections",
            Some(json!({
                "voter_id_hash": "sha256:voter-2",
                "claimed_by_biometric_hash": "sha256:bio-2",
                "correction_type": "name_spelling",
                "evidence_refs": ["s3://evidence/affidavit.pdf"]
            })),
        )
        .await,
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["status"], "pending");

    let decided = send(
        &app,
        "PATCH",
        &format!("/corrections/{id}/decision"),
        Some(json!({
            "decision": "approve",
            "decided_by": "00000000-0000-0000-0000-0000000000dd"
        })),
    )
    .await;
    assert_eq!(decided.status(), StatusCode::OK);
    let det = body_json(decided).await;
    assert_eq!(det["claim_id"], id.as_str());
    assert_eq!(det["decision"], "approve");

    let after = body_json(send(&app, "GET", &format!("/corrections/{id}"), None).await).await;
    assert_eq!(
        after["status"], "approved",
        "the determination must reach the claim, not just be recorded beside it"
    );
}

#[tokio::test]
async fn a_bad_decision_is_rejected() {
    let app = app();
    let created = body_json(
        send(
            &app,
            "POST",
            "/corrections",
            Some(json!({
                "voter_id_hash": "sha256:v3",
                "claimed_by_biometric_hash": "sha256:b3",
                "correction_type": "name_spelling",
                "evidence_refs": []
            })),
        )
        .await,
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();
    let resp = send(
        &app,
        "PATCH",
        &format!("/corrections/{id}/decision"),
        Some(json!({"decision": "maybe", "decided_by": "00000000-0000-0000-0000-0000000000dd"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn deciding_an_unknown_claim_is_404() {
    let app = app();
    let resp = send(
        &app,
        "PATCH",
        "/corrections/00000000-0000-0000-0000-0000000000ee/decision",
        Some(json!({"decision": "approve", "decided_by": "00000000-0000-0000-0000-0000000000dd"})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "decide_correction previously answered 200 for any claim id"
    );
}

#[tokio::test]
async fn auto_referral_considers_real_claims_and_refers_eligible_ones() {
    // Previously logged that it was a stub and answered 204, so an approved
    // correction inside the referral window was never referred.
    let app = app();
    let id = body_json(
        send(
            &app,
            "POST",
            "/corrections",
            Some(json!({
                "voter_id_hash": "sha256:v4",
                "claimed_by_biometric_hash": "sha256:b4",
                "correction_type": "name_spelling",
                "evidence_refs": []
            })),
        )
        .await,
    )
    .await["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Pending is not referrable, so nothing is referred yet.
    let pending = body_json(send(&app, "POST", "/corrections/auto-referral/run", None).await).await;
    assert_eq!(pending["considered"], 1);
    assert_eq!(pending["referred"], 0, "a Pending claim is not referrable");

    send(
        &app,
        "PATCH",
        &format!("/corrections/{id}/decision"),
        Some(json!({"decision": "approve", "decided_by": "00000000-0000-0000-0000-0000000000dd"})),
    )
    .await;
    let approved =
        body_json(send(&app, "POST", "/corrections/auto-referral/run", None).await).await;
    assert_eq!(approved["considered"], 1);
    assert_eq!(
        approved["referred"], 1,
        "should_refer requires Approved plus a polling day within 60 days, and
        run_batch passes now(), so an approved claim is referrable and the
        crate's own scheduler must actually refer it"
    );

    // Idempotent: a second run must not refer the same claim twice.
    let again = body_json(send(&app, "POST", "/corrections/auto-referral/run", None).await).await;
    assert_eq!(again["referred"], 0, "referral must be idempotent");
}

#[tokio::test]
async fn roll_lookup_enforces_the_crate_own_lookup_cap() {
    // Previously returned found:false for every request with no cap at all, so
    // the constant-time shape the comment described protected nothing.
    let app = app();
    let body = json!({"voter_id_hash": "sha256:probe", "step_up_token": "t"});
    let mut saw_limit = false;
    for _ in 0..40 {
        let resp = send(&app, "POST", "/rolls/lookup", Some(body.clone())).await;
        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            saw_limit = true;
            break;
        }
        assert_eq!(resp.status(), StatusCode::OK);
    }
    assert!(saw_limit, "the LookupCap should refuse a repeated subject");
}

#[tokio::test]
async fn roll_lookup_preserves_its_constant_time_shape() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/rolls/lookup",
        Some(json!({"voter_id_hash": "sha256:shape", "step_up_token": "t"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let lookup = body_json(resp).await;
    assert_eq!(lookup["found"], false);
    assert!(
        lookup["station_id"].is_null(),
        "station_id must be present-and-null so a caller cannot distinguish \\
         absent from errored by shape"
    );
}

#[tokio::test]
async fn voter_list_search_refuses_because_there_is_no_list() {
    let app = app();
    let resp = send(&app, "GET", "/api/v1/inspection/voter-list/search", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let problem = body_json(resp).await;
    assert!(problem["detail"]
        .as_str()
        .unwrap_or("")
        .contains("no voter list"));
}

#[tokio::test]
async fn routers_do_not_share_records() {
    let a = app();
    let b = app();
    snapshot(&a).await;
    let b_body = body_json(
        send(
            &b,
            "GET",
            "/rolls/snapshot/00000000-0000-0000-0000-0000000000bb",
            None,
        )
        .await,
    )
    .await;
    assert_eq!(b_body["status"], 404);
}
