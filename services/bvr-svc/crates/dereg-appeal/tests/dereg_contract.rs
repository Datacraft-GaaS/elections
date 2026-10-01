//! Contract tests for dereg-appeal, pinned against
//! `packages/openapi-contracts/dereg/appeal.yaml`.
//!
//! **All four handlers were fabrications.** The previous tests asserted:
//!
//! - `list` returned 200 with an empty page, so a filed case was invisible.
//! - `get` returned 404 for every id, so a case could not be read back.
//! - `decide` returned 201 for any appeal id with `reinstated` derived from a
//!   `reinstate`/`uphold` pair the contract does not have, so a decision was
//!   recorded against a case that need not exist.
//! - USSD answered `END Service unavailable` to every caller.
//!
//! The verdict direction is the part worth stating: the contract's enum is
//! `[upheld, dismissed]`. **Upholding** the appeal means the deregistration was
//! wrong, so the voter is reinstated; **dismissing** it means the deregistration
//! stands. The old handler had that inverted, which is the direction that puts a
//! person back on the register on a dismissal.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use dereg_appeal::state::AppState;
use dereg_appeal::store::{DeregStore, SharedDeregStore};

fn state(store: SharedDeregStore) -> AppState {
    AppState {
        // The handlers under test read the in-memory store; the pool is carried by
        // the state but never touched, so a lazy handle is enough and no database
        // is needed to run these.
        pool: sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://localhost/dereg")
            .expect("lazy pool"),
        store,
    }
}

fn seeded() -> AppState {
    let store = DeregStore::new();
    store.file_appeal(
        "sha256:aaaa0000",
        "DEREG-2027-0042",
        "I was removed during the boundary re-drawing exercise.",
        "sha256:doc1111",
    );
    state(store)
}

const OFFICER: &str = "sha256:officer9999";

async fn get(s: AppState, uri: &str) -> (StatusCode, Value) {
    let resp = dereg_appeal::routes::spec::router()
        .with_state(s)
        .oneshot(Request::builder().uri(uri).method("GET").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn post(s: AppState, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = dereg_appeal::routes::spec::router()
        .with_state(s)
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

fn first_appeal_id(s: &AppState) -> String {
    s.store.appeals()[0].appeal_id.clone()
}

#[tokio::test]
async fn a_filed_appeal_is_visible_in_the_list() {
    // Previously: an empty page for every call.
    let s = seeded();
    let (status, body) = get(s, "/api/v1/dereg/appeals").await;
    assert_eq!(status, StatusCode::OK);
    let items = body["items"].as_array().expect("items array");
    assert_eq!(items.len(), 1, "a filed appeal must be visible");
    // The contract's DeregAppeal requires all of these.
    for required in [
        "appeal_id",
        "voter_id_hash",
        "deregistration_ref",
        "grounds",
        "supporting_doc_hash",
        "status",
        "submitted_at",
    ] {
        assert!(items[0].get(required).is_some(), "missing {required}");
    }
    assert_eq!(items[0]["status"], "submitted");
}

#[tokio::test]
async fn an_appeal_reads_back_by_id() {
    // Previously: 404 for every id.
    let s = seeded();
    let id = first_appeal_id(&s);
    let (status, body) = get(s, &format!("/api/v1/dereg/appeals/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["appeal_id"], id);
    assert_eq!(body["deregistration_ref"], "DEREG-2027-0042");
}

#[tokio::test]
async fn an_unknown_appeal_is_not_found() {
    let (status, _) = get(seeded(), "/api/v1/dereg/appeals/8f14e45f-ceea-467a-9c6e-000000000000").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn upholding_an_appeal_reinstates_the_voter() {
    let s = seeded();
    let id = first_appeal_id(&s);
    let (status, body) = post(
        s.clone(),
        &format!("/api/v1/dereg/appeals/{id}/decisions"),
        json!({
            "verdict": "upheld",
            "grounds": "The boundary re-drawing did not cover this ward.",
            "decided_by_hash": OFFICER
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    // Upheld = the appeal succeeded = the deregistration was wrong = reinstated.
    assert_eq!(body["reinstated"], true);
    assert_eq!(body["verdict"], "upheld");
    // The contract's AppealDecision is allOf(AppealDecisionCreate) + three fields.
    for required in [
        "decision_id",
        "verdict",
        "grounds",
        "decided_by_hash",
        "reinstated",
        "decided_at",
    ] {
        assert!(body.get(required).is_some(), "missing {required}");
    }
    // The decision is a finding, not an applied register change.
    assert!(body["blocking_reason"].as_str().is_some_and(|r| r.contains("register service")));

    // The appeal's own status follows the verdict.
    let (status, appeal) = get(s, &format!("/api/v1/dereg/appeals/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(appeal["status"], "upheld");
}

#[tokio::test]
async fn dismissing_an_appeal_leaves_the_deregistration_standing() {
    // This is the direction the previous handler had backwards: a dismissal must
    // not put anyone back on the register.
    let s = seeded();
    let id = first_appeal_id(&s);
    let (status, body) = post(
        s.clone(),
        &format!("/api/v1/dereg/appeals/{id}/decisions"),
        json!({
            "verdict": "dismissed",
            "grounds": "The voter was deregistered for a recorded reason.",
            "decided_by_hash": OFFICER
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["reinstated"], false);
    let (status, appeal) = get(s, &format!("/api/v1/dereg/appeals/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(appeal["status"], "dismissed");
}

#[tokio::test]
async fn a_decision_cannot_be_recorded_against_nothing() {
    // Previously: 201 Created for any appeal id.
    let (status, body) = post(
        seeded(),
        "/api/v1/dereg/appeals/8f14e45f-ceea-467a-9c6e-000000000000/decisions",
        json!({
            "verdict": "upheld",
            "grounds": "Reinstated without a case.",
            "decided_by_hash": OFFICER
        }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["detail"].as_str().unwrap_or_default().contains("no appeal"));
}

#[tokio::test]
async fn a_verdict_outside_the_contract_enum_is_rejected() {
    // The contract enum is [upheld, dismissed]. The old handler accepted
    // "reinstate"/"uphold", which the contract does not have.
    let s = seeded();
    let id = first_appeal_id(&s);
    for bad in ["reinstate", "uphold", "REINSTATED", "yes", ""] {
        let (status, _) = post(
            s.clone(),
            &format!("/api/v1/dereg/appeals/{id}/decisions"),
            json!({ "verdict": bad, "grounds": "g", "decided_by_hash": OFFICER }),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "verdict {bad:?} must be refused");
    }
}

#[tokio::test]
async fn a_decision_without_grounds_or_an_officer_is_rejected() {
    let s = seeded();
    let id = first_appeal_id(&s);
    for body in [
        json!({ "verdict": "upheld", "grounds": "   ", "decided_by_hash": OFFICER }),
        json!({ "verdict": "upheld", "grounds": "g", "decided_by_hash": "" }),
    ] {
        let (status, _) =
            post(s.clone(), &format!("/api/v1/dereg/appeals/{id}/decisions"), body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn an_appeal_cannot_be_decided_twice() {
    let s = seeded();
    let id = first_appeal_id(&s);
    let uri = format!("/api/v1/dereg/appeals/{id}/decisions");
    let decision = json!({
        "verdict": "upheld",
        "grounds": "First decision.",
        "decided_by_hash": OFFICER
    });
    let (status, _) = post(s.clone(), &uri, decision.clone()).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = post(s, &uri, decision).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["detail"].as_str().unwrap_or_default().contains("already been decided"));
}

const HASH: &str = "b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3";

fn ussd_body(text: &str) -> Value {
    json!({ "session_id": "ATU-1", "msisdn_hash": HASH, "text": text, "service_code": "*384#" })
}

#[tokio::test]
async fn ussd_opens_a_menu_instead_of_hanging_up() {
    // Previously: `END Service unavailable` to every caller.
    let (status, body) = post(seeded(), "/api/v1/dereg/ussd/sessions", ussd_body("")).await;
    assert_eq!(status, StatusCode::OK);
    // This contract's action enum is [CON, END].
    assert_eq!(body["action"], "CON");
    assert!(body["response_text"].as_str().unwrap().contains('1'));
}

#[tokio::test]
async fn ussd_reports_a_real_appeal_by_its_notice_reference() {
    let s = seeded();
    let (status, body) =
        post(s.clone(), "/api/v1/dereg/ussd/sessions", ussd_body("1*DEREG-2027-0042")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["action"], "END");
    let text = body["response_text"].as_str().unwrap();
    assert!(text.contains("submitted"), "{text}");
    assert!(text.chars().count() <= 182, "USSD screens are firmware-limited");

    // And after a decision, the USSD reports the real outcome rather than a
    // standing "in progress".
    let id = first_appeal_id(&s);
    let (status, _) = post(
        s.clone(),
        &format!("/api/v1/dereg/appeals/{id}/decisions"),
        json!({ "verdict": "upheld", "grounds": "Wrong ward.", "decided_by_hash": OFFICER }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) =
        post(s, "/api/v1/dereg/ussd/sessions", ussd_body("1*DEREG-2027-0042")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["response_text"].as_str().unwrap().contains("Umeirudishwa"));
}

#[tokio::test]
async fn ussd_does_not_invent_an_appeal_it_never_held() {
    let (status, body) =
        post(seeded(), "/api/v1/dereg/ussd/sessions", ussd_body("1*DEREG-0000-0000")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["action"], "END");
    assert!(body["response_text"].as_str().unwrap().contains("Hakuna appeal"));
}

#[tokio::test]
async fn ussd_rejects_a_malformed_request() {
    // A raw MSISDN must never be transmitted; the contract wants 64 hex.
    for body in [
        json!({ "session_id": "ATU-1", "msisdn_hash": "0712345678", "text": "", "service_code": "*384#" }),
        json!({ "session_id": "", "msisdn_hash": HASH, "text": "", "service_code": "*384#" }),
        json!({ "session_id": "ATU-1", "msisdn_hash": HASH, "text": "", "service_code": "384" }),
    ] {
        let (status, _) = post(seeded(), "/api/v1/dereg/ussd/sessions", body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}
