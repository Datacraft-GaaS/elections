//! Contract tests for verifier-api, pinned against
//! `packages/openapi-contracts/verifier/{verify,ussd,whatsapp}.yaml`.
//!
//! **Five of these previously pinned a fabrication.** `registration_lookup_shape`
//! asserted `200 OK` with a `registered` field present, from a handler that
//! returned `registered: false` for every request. `station_detail_echoes_code`
//! asserted `200 OK` for a station that had never been registered.
//! `result_summary_shape` asserted `200 OK` with `results` present, from a handler
//! returning `results: []`. `ussd_session_ends` asserted `action == "end"`, which
//! is what the handler answered to every caller. And
//! `whatsapp_webhook_acknowledges` asserted that an **unsigned** body was
//! acknowledged.
//!
//! Each now pins the real behaviour, and each names the behaviour it replaced.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use verifier_api::{
    http::{router, sign},
    store::{
        Coordinates, LeadingCandidate, ResultSummary, StationDetail, VerifierStore,
    },
};

fn station() -> StationDetail {
    StationDetail {
        station_code: "KE-047-0042".to_string(),
        station_name: "Westlands Primary School".to_string(),
        county_code: "047".to_string(),
        county_name: "Nairobi".to_string(),
        constituency_code: "KE-047".to_string(),
        constituency_name: "Westlands".to_string(),
        ward_name: "Parklands/Highridge".to_string(),
        registered_voters: 1_204,
        coordinates: Coordinates { lat: -1.2536, lng: 36.8077 },
        streams: Some(6),
        address: Some("Off Waiyaki Way".to_string()),
        accessibility_features: Some(vec!["ramp".to_string()]),
        opening_time: Some("06:00:00".to_string()),
        closing_time: Some("17:00:00".to_string()),
    }
}

fn summary() -> ResultSummary {
    ResultSummary {
        election_id: "KE-GE-2027".to_string(),
        election_name: "Kenya General Election 2027".to_string(),
        status: "provisional".to_string(),
        total_valid_votes: 4_210_553,
        total_registered_voters: Some(21_000_000),
        turnout_pct: 62.1,
        leading_candidate: LeadingCandidate {
            display_name: "A. Candidate".to_string(),
            party_acronym: "ABC".to_string(),
            votes: 1_100_000,
            vote_pct: 26.1,
        },
        top_candidates: None,
        as_of: chrono::Utc::now().to_rfc3339(),
    }
}

/// A store with nothing published, so "absent" is the default under test.
fn empty() -> axum::Router {
    router(VerifierStore::new().shared())
}

fn seeded() -> axum::Router {
    let store = VerifierStore::new();
    store.register_station(station());
    store.record_results(summary());
    router(store.shared())
}

/// A store with published data *and* the telco USSD secret, for signed USSD calls.
fn secured_seeded() -> axum::Router {
    let store = VerifierStore::new().with_ussd_webhook_secret("telco-secret");
    store.register_station(station());
    store.record_results(summary());
    router(store.shared())
}

fn secured() -> axum::Router {
    router(
        VerifierStore::new()
            .with_whatsapp_app_secret("app-secret")
            .with_whatsapp_verify_token("verify-token")
            .with_ussd_webhook_secret("telco-secret")
            .shared(),
    )
}

async fn send(app: axum::Router, req: Request<Body>) -> (StatusCode, String) {
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .uri(uri)
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let (status, text) = send(app, req).await;
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

async fn post_signed(app: axum::Router, uri: &str, body: Value, secret: &str) -> (StatusCode, Value) {
    let raw = body.to_string();
    let req = Request::builder()
        .uri(uri)
        .method("POST")
        .header("content-type", "application/json")
        .header("x-eel-webhook-sig", sign(secret, raw.as_bytes()))
        .body(Body::from(raw))
        .unwrap();
    let (status, text) = send(app, req).await;
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

async fn get(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder().uri(uri).method("GET").body(Body::empty()).unwrap();
    let (status, text) = send(app, req).await;
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

const HASH: &str = "a3f5c2d1b4e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2";

#[tokio::test]
async fn lookup_states_the_roll_is_absent_rather_than_claiming_a_negative() {
    // Previously: 200 with `registered: false`, no reason, for every request.
    let (status, body) = post_json(
        empty(),
        "/api/v1/verifier/lookups",
        json!({ "id_number_hash": HASH }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // The contract offers no 501 here, but it defines `not_registered_reason`, and
    // `roll_not_published` is exactly this situation. The distinction from
    // `not_found` is the whole point: one means "check back", the other means
    // "you are not registered".
    assert_eq!(body["not_registered_reason"], "roll_not_published");
    assert_ne!(body["not_registered_reason"], "not_found");
    assert_eq!(body["registered"], false);
    assert!(body["lookup_id"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(body["as_of"].as_str().is_some_and(|s| !s.is_empty()));
}

#[tokio::test]
async fn lookup_rejects_a_malformed_hash() {
    // The contract pattern is ^[0-9a-f]{64}$. The raw national ID must never be
    // transmitted, so a short or non-hex value is a client bug worth naming.
    for bad in ["abc", "ZZZ", &"a".repeat(63), &"a".repeat(65)] {
        let (status, _) = post_json(
            empty(),
            "/api/v1/verifier/lookups",
            json!({ "id_number_hash": bad }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad} should be rejected");
    }
}

#[tokio::test]
async fn station_detail_returns_the_contract_shape() {
    // Previously: 200 with `name: null, constituency: null` for any station.
    let (status, body) = get(seeded(), "/api/v1/verifier/station/KE-047-0042").await;
    assert_eq!(status, StatusCode::OK);
    for required in [
        "station_code",
        "station_name",
        "county_code",
        "county_name",
        "constituency_code",
        "constituency_name",
        "ward_name",
        "registered_voters",
        "coordinates",
    ] {
        assert!(body.get(required).is_some(), "missing {required}");
    }
    assert_eq!(body["coordinates"]["lat"].is_number(), true);
}

#[tokio::test]
async fn an_unpublished_station_is_not_found() {
    let (status, _) = get(seeded(), "/api/v1/verifier/station/KE-047-9999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn result_summary_returns_the_contract_shape() {
    let (status, body) = get(seeded(), "/api/v1/verifier/results/summary").await;
    assert_eq!(status, StatusCode::OK);
    for required in [
        "election_id",
        "election_name",
        "status",
        "total_valid_votes",
        "turnout_pct",
        "leading_candidate",
        "as_of",
    ] {
        assert!(body.get(required).is_some(), "missing {required}");
    }
    assert_eq!(body["leading_candidate"]["display_name"], "A. Candidate");
}

#[tokio::test]
async fn an_absent_result_summary_is_a_fault_not_an_empty_election() {
    // Previously: 200 with `results: []`, which reads as an election with no
    // results. The contract offers 200 or 500 here, not 404: the summary is
    // published every two minutes during tallying, so its absence is a fault in
    // the publishing chain.
    let (status, body) = get(empty(), "/api/v1/verifier/results/summary").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(body["detail"].as_str().unwrap_or_default().contains("published"));
}

#[tokio::test]
async fn ussd_refuses_without_a_configured_webhook_secret() {
    // The signature is checked before the body is parsed: a menu is only worth
    // serving to a gateway that authenticated.
    let (status, _) = post_json(
        empty(),
        "/api/v1/verifier/ussd/session",
        json!({ "session_id": "ATU-1", "msisdn_hash": HASH, "text": "", "service_code": "*384#" }),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn ussd_refuses_an_unsigned_or_mis_signed_call() {
    // Previously: 200 with `action: end` to every caller, signed or not.
    let body = json!({ "session_id": "ATU-1", "msisdn_hash": HASH, "text": "", "service_code": "*384#" });
    let (status, _) = post_json(secured(), "/api/v1/verifier/ussd/session", body.clone()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let raw = body.to_string();
    let req = Request::builder()
        .uri("/api/v1/verifier/ussd/session")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-eel-webhook-sig", sign("wrong-secret", raw.as_bytes()))
        .body(Body::from(raw))
        .unwrap();
    let (status, _) = send(secured(), req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn ussd_opens_a_menu_instead_of_hanging_up() {
    // Previously: `action: end` to every caller. The line hung up before a citizen
    // with a phone and no smartphone could state anything.
    let (status, body) = post_signed(
        secured_seeded(),
        "/api/v1/verifier/ussd/session",
        json!({ "session_id": "ATU-1", "msisdn_hash": HASH, "text": "", "service_code": "*384#" }),
        "telco-secret",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["action"], "continue");
    assert!(body["text"].as_str().unwrap().contains("Kituo"));
}

#[tokio::test]
async fn ussd_resolves_a_station_from_accumulated_input() {
    // `text` is the subscriber's accumulated `*`-delimited input, so the whole
    // history arrives each time and no server-side session is needed.
    let (status, body) = post_signed(
        secured_seeded(),
        "/api/v1/verifier/ussd/session",
        json!({
            "session_id": "ATU-2",
            "msisdn_hash": HASH,
            "text": "1*KE-047-0042",
            "service_code": "*384#"
        }),
        "telco-secret",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["action"], "end");
    let text = body["text"].as_str().unwrap();
    assert!(text.contains("Westlands Primary School"), "{text}");
    assert!(text.chars().count() <= 182, "USSD screens are firmware-limited");
}

#[tokio::test]
async fn ussd_does_not_invent_a_station_it_never_published() {
    let (status, body) = post_signed(
        secured_seeded(),
        "/api/v1/verifier/ussd/session",
        json!({
            "session_id": "ATU-3",
            "msisdn_hash": HASH,
            "text": "1*KE-047-9999",
            "service_code": "*384#"
        }),
        "telco-secret",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["text"].as_str().unwrap().contains("kilichochapishwa"));
}

#[tokio::test]
async fn ussd_rejects_a_malformed_msisdn_hash() {
    let (status, _) = post_signed(
        secured_seeded(),
        "/api/v1/verifier/ussd/session",
        json!({ "session_id": "ATU-4", "msisdn_hash": "12345", "text": "", "service_code": "*384#" }),
        "telco-secret",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn whatsapp_challenge_echoes_only_on_a_matching_token() {
    // Previously: `{"challenge": "verified"}` for any request, comparing nothing.
    let (status, text) = send(
        secured(),
        Request::builder()
            .uri("/api/v1/verifier/whatsapp/webhook?hub.mode=subscribe&hub.challenge=abc123&hub.verify_token=verify-token")
            .method("GET")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // The contract asks for the bare challenge as plain text.
    assert_eq!(text, "abc123");
}

#[tokio::test]
async fn whatsapp_challenge_is_forbidden_on_a_wrong_or_absent_token() {
    for token in ["wrong-token", ""] {
        let (status, _) = send(
            secured(),
            Request::builder()
                .uri(format!(
                    "/api/v1/verifier/whatsapp/webhook?hub.mode=subscribe&hub.challenge=abc&hub.verify_token={token}"
                ))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "token {token} must be refused");
    }
    // An unconfigured deployment must not accept every challenge.
    let (status, _) = send(
        empty(),
        Request::builder()
            .uri("/api/v1/verifier/whatsapp/webhook?hub.mode=subscribe&hub.challenge=abc&hub.verify_token=any")
            .method("GET")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn whatsapp_webhook_refuses_an_unsigned_body() {
    // Previously: `{"acknowledged": true}` for any body, signed or not.
    let (status, _) = post_json(
        secured(),
        "/api/v1/verifier/whatsapp/webhook",
        json!({ "entry": [] }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn whatsapp_webhook_refuses_when_no_app_secret_is_configured() {
    let (status, _) = post_json(
        empty(),
        "/api/v1/verifier/whatsapp/webhook",
        json!({ "entry": [] }),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn whatsapp_webhook_accepts_a_correctly_signed_body() {
    let raw = json!({ "object": "whatsapp_business_account" }).to_string();
    let req = Request::builder()
        .uri("/api/v1/verifier/whatsapp/webhook")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-hub-signature-256", sign("app-secret", raw.as_bytes()))
        .body(Body::from(raw))
        .unwrap();
    let (status, _) = send(secured(), req).await;
    assert_eq!(status, StatusCode::OK);
    // The test build suppresses this unused warning without hiding real ones.
    let _ = header::CONTENT_TYPE;
}
