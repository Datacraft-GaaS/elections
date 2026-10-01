//! Round-trip tests for the `/api/v1/mobile/bff/*` and `/v1/agent/*` surfaces.
//!
//! The fabrications, all of which presented invented values as measurements:
//!
//! - `get_station_status` returned `turnout_count: 1247`, `kit_battery_level: 87.5`,
//!   `connectivity_status: "online"` and `queue_depth: 3` for **every** station.
//! - `get_attestation_verdict` decided the verdict from the **device id string**,
//!   ignored the attestation object entirely, and minted an observer licence on a
//!   MEETS.
//! - `create_bff_session` swallowed a failed OIDC exchange and fell back to
//!   `mock_access_token`, then wrapped it in a string shaped like a JWT.
//! - `refresh_bff_session` minted a JWT-shaped string from a UUID with no session
//!   lookup, so a refresh worked for any caller.
//! - `list_dispatch_assignments` returned two fabricated assignments for any
//!   worker; `update_dispatch_assignment` keyed its idempotency store on a fresh
//!   UUID each call, so it never hit, and answered `updated: true` for any id.
//! - `report_incident` and `create_incident` discarded the body and cached `{}`.
//! - `register_push_token` minted an id and stored nothing.
//! - `get_observer_access_pass` minted a pass for anyone who asked.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use mobile_bff::router::build_router;

fn app() -> axum::Router {
    build_router()
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

async fn send_with_headers(
    app: &axum::Router,
    uri: &str,
    body: Value,
    headers: &[(&str, &str)],
) -> axum::response::Response {
    let mut b = Request::builder().uri(uri).method("POST").header("content-type", "application/json");
    for (k, v) in headers {
        b = b.header(*k, *v);
    }
    app.clone()
        .oneshot(b.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// station status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_station_that_never_checked_in_has_no_status() {
    // The headline. Previously every station reported a turnout of 1247 and a
    // battery of 87.5.
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/api/v1/mobile/bff/station/status?station_code=STN001",
        None,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "a station that has never checked in has no status to report"
    );
}

#[tokio::test]
async fn station_status_requires_a_station_code() {
    let app = app();
    let resp = send(&app, "GET", "/api/v1/mobile/bff/station/status", None).await;
    assert_ne!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// attestation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_attestation_is_never_met_on_the_device_id() {
    // The old handler read `device_id.starts_with("emulator")` and returned MEETS
    // for anything else, ignoring the attestation object entirely.
    for device in ["Pixel-8", "SM-A536B", "anything-at-all"] {
        let app = app();
        let verdict = body_json(
            send(
                &app,
                "POST",
                "/api/v1/mobile/bff/attestation/verdict",
                Some(json!({
                    "device_id": device,
                    "nonce": "n",
                    "attestation_object": "eyJhbGciOiJSU0EtT0FFUCJ9.payload.sig"
                })),
            )
            .await,
        )
        .await;
        assert_eq!(
            verdict["verdict"], "UNVERIFIED",
            "a device named {device} must not be attested: {verdict:?}"
        );
        assert!(
            verdict["license"].is_null(),
            "no licence rests on an unverified verdict"
        );
        let reason = verdict["blocking_reason"].as_str().unwrap_or_default();
        assert!(
            reason.contains("attestation service"),
            "the reason should name what is absent: {reason}"
        );
    }
}

#[tokio::test]
async fn a_rooted_device_named_as_ordinary_is_still_unverified() {
    // A device id is not evidence in either direction.
    let app = app();
    let verdict = body_json(
        send(
            &app,
            "POST",
            "/api/v1/mobile/bff/attestation/verdict",
            Some(json!({
                "device_id": "totally-normal-device",
                "nonce": "n",
                "attestation_object": "not-a-real-attestation"
            })),
        )
        .await,
    )
    .await;
    assert_eq!(verdict["verdict"], "UNVERIFIED");
}

#[tokio::test]
async fn an_attestation_keeps_the_digest_of_what_was_presented() {
    let app = app();
    let verdict = body_json(
        send(
            &app,
            "POST",
            "/api/v1/mobile/bff/attestation/verdict",
            Some(json!({"device_id": "d", "nonce": "n", "attestation_object": "abc"})),
        )
        .await,
    )
    .await;
    let digest = verdict["attestation_object_sha256"].as_str().unwrap();
    assert!(digest.starts_with("sha256:"), "got {digest:?}");
}

#[tokio::test]
async fn an_attestation_needs_a_device_and_an_object() {
    let app = app();
    for body in [
        json!({"device_id": "", "nonce": "n", "attestation_object": "a"}),
        json!({"device_id": "d", "nonce": "n", "attestation_object": "  "}),
    ] {
        let resp = send(
            &app,
            "POST",
            "/api/v1/mobile/bff/attestation/verdict",
            Some(body),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

// ---------------------------------------------------------------------------
// sessions
// ---------------------------------------------------------------------------

async fn create_session(app: &axum::Router) -> (String, String) {
    let body = body_json(
        send(
            app,
            "POST",
            "/api/v1/mobile/bff/sessions",
            Some(json!({"code": "auth-code", "state": "st", "nonce": "n-1", "scopes": ["openid"]})),
        )
        .await,
    )
    .await;
    (
        body["session_id"].as_str().unwrap().to_string(),
        body["refresh_token"].as_str().unwrap().to_string(),
    )
}

#[tokio::test]
async fn a_session_token_is_not_shaped_like_a_jwt() {
    // The old handler returned `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.<access>...`.
    // A string beginning `eyJhbGci` reads as signed to any client that inspects it,
    // and this service signs nothing.
    let app = app();
    let body = body_json(
        send(
            &app,
            "POST",
            "/api/v1/mobile/bff/sessions",
            Some(json!({"code": "c", "state": "s", "nonce": "n", "scopes": []})),
        )
        .await,
    )
    .await;
    let token = body["token"].as_str().unwrap();
    assert!(
        !token.starts_with("eyJ"),
        "the token must not be JWT-shaped: {token:?}"
    );
    assert!(!token.contains("..."), "no elided literal: {token:?}");
    assert!(token.starts_with("sha256:"), "got {token:?}");
}

#[tokio::test]
async fn a_session_records_that_the_oidc_exchange_did_not_complete() {
    let app = app();
    let body = body_json(
        send(
            &app,
            "POST",
            "/api/v1/mobile/bff/sessions",
            Some(json!({"code": "c", "state": "s", "nonce": "n", "scopes": ["openid"]})),
        )
        .await,
    )
    .await;
    assert_eq!(
        body["oidc_exchange"], "not-completed",
        "no identity provider is reachable, so the exchange did not happen"
    );
    let reason = body["blocking_reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("does not carry an OIDC identity"),
        "the reason should say what the session is: {reason}"
    );
    assert!(!body["refresh_token"].is_null());
    assert_eq!(body["expires_in"], 300);
}

#[tokio::test]
async fn a_session_needs_a_code_and_a_nonce() {
    let app = app();
    for body in [
        json!({"code": "", "state": "s", "nonce": "n", "scopes": []}),
        json!({"code": "c", "state": "s", "nonce": "  ", "scopes": []}),
    ] {
        let resp = send(&app, "POST", "/api/v1/mobile/bff/sessions", Some(body)).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn a_refresh_works_for_a_issued_session() {
    let app = app();
    let (session_id, refresh) = create_session(&app).await;
    let refreshed = body_json(
        send(
            &app,
            "POST",
            "/api/v1/mobile/bff/sessions/refresh",
            Some(json!({"refresh_token": refresh})),
        )
        .await,
    )
    .await;
    assert_eq!(refreshed["session_id"], session_id.as_str());
    assert!(refreshed["token"].as_str().unwrap().starts_with("sha256:"));
    assert_eq!(refreshed["scopes"], json!(["openid"]));
}

#[tokio::test]
async fn a_refresh_does_not_work_for_an_unknown_token() {
    // The old handler minted a JWT-shaped string from a UUID with no session
    // lookup, so a refresh worked for any caller.
    let app = app();
    for token in ["anything", "sha256:deadbeef", ""] {
        let resp = send(
            &app,
            "POST",
            "/api/v1/mobile/bff/sessions/refresh",
            Some(json!({"refresh_token": token})),
        )
        .await;
        assert!(
            resp.status() == StatusCode::UNAUTHORIZED
                || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "a refresh for {token:?} must not succeed, got {}",
            resp.status()
        );
    }
}

#[tokio::test]
async fn a_refresh_is_bound_to_its_own_session() {
    let app = app();
    let (_, refresh_a) = create_session(&app).await;
    let (session_b, _) = create_session(&app).await;
    let refreshed = body_json(
        send(
            &app,
            "POST",
            "/api/v1/mobile/bff/sessions/refresh",
            Some(json!({"refresh_token": refresh_a})),
        )
        .await,
    )
    .await;
    assert_ne!(
        refreshed["session_id"], session_b.as_str(),
        "a refresh token must only refresh its own session"
    );
}

// ---------------------------------------------------------------------------
// dispatch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_worker_with_no_assignments_has_none() {
    // The old handler returned `assign-001` active and `assign-002` pending for any
    // worker id.
    let app = app();
    let listed = body_json(
        send(
            &app,
            "GET",
            "/api/v1/mobile/bff/dispatch/assignments?worker_id=worker-1",
            None,
        )
        .await,
    )
    .await;
    assert_eq!(listed["count"], 0);
    assert!(listed["assignments"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn a_dispatch_update_needs_a_real_assignment() {
    // The old handler answered `updated: true` for any id.
    let app = app();
    let resp = send(
        &app,
        "PATCH",
        "/api/v1/mobile/bff/dispatch/assignments/assign-does-not-exist",
        Some(json!({"status": "accepted"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_dispatch_update_requires_a_status() {
    let app = app();
    let resp = send(
        &app,
        "PATCH",
        "/api/v1/mobile/bff/dispatch/assignments/assign-1",
        Some(json!({})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn an_unknown_dispatch_status_is_rejected() {
    let app = app();
    let resp = send(
        &app,
        "PATCH",
        "/api/v1/mobile/bff/dispatch/assignments/assign-1",
        Some(json!({"status": "teleported"})),
    )
    .await;
    // Either the assignment is unknown (404) or the status is (422); both refuse.
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_dispatch_listing_needs_a_worker() {
    let app = app();
    let resp = send(&app, "GET", "/api/v1/mobile/bff/dispatch/assignments", None).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ---------------------------------------------------------------------------
// incidents
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_reported_incident_is_recorded() {
    // The old handler discarded the body and cached `{}`.
    let app = app();
    let incident = json!({
        "station_code": "STN001",
        "incident_type": "seal-broken",
        "description": "ballot box seal was broken on arrival",
        "severity": "high"
    });
    let filed = body_json(
        send_with_headers(
            &app,
            "/api/v1/mobile/bff/incidents",
            incident.clone(),
            &[("idempotency-key", "key-1")],
        )
        .await,
    )
    .await;
    assert_eq!(filed["severity"], "high");
    assert!(!filed["incident_id"].is_null());
    assert_eq!(filed["station_known"], false, "STN001 has never checked in here");

    // A retry under the same key returns the same incident, not an empty object.
    let retry = body_json(
        send_with_headers(
            &app,
            "/api/v1/mobile/bff/incidents",
            incident,
            &[("idempotency-key", "key-1")],
        )
        .await,
    )
    .await;
    assert_eq!(
        retry["incident_id"], filed["incident_id"],
        "a retried POST must not file a second incident"
    );
    assert!(!retry["duplicate_of"].is_null());
}

#[tokio::test]
async fn an_incident_needs_a_type_a_description_and_a_known_severity() {
    let app = app();
    for body in [
        json!({"station_code": "S", "incident_type": "", "description": "d", "severity": "low"}),
        json!({"station_code": "S", "incident_type": "t", "description": "  ", "severity": "low"}),
        json!({"station_code": "S", "incident_type": "t", "description": "d", "severity": "catastrophic"}),
    ] {
        let resp = send(&app, "POST", "/api/v1/mobile/bff/incidents", Some(body.clone())).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    }
}

// ---------------------------------------------------------------------------
// push tokens and observer passes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_push_token_is_stored_and_never_echoed() {
    // The old handler minted a token id and stored nothing.
    let app = app();
    let created = body_json(
        send(
            &app,
            "POST",
            "/api/v1/mobile/bff/push-tokens",
            Some(json!({"device_id": "device-1", "app_token": "provider-secret", "platform": "android"})),
        )
        .await,
    )
    .await;
    assert!(created["token_id"].as_str().unwrap().starts_with("pt-"));
    assert!(
        created["token_hash"].as_str().unwrap().starts_with("sha256:"),
        "the provider token is a credential, so only its digest is kept"
    );
    assert!(
        !created.to_string().contains("provider-secret"),
        "the provider token must not be echoed back: {created}"
    );
}

#[tokio::test]
async fn a_push_token_needs_a_device_and_a_token() {
    let app = app();
    for body in [
        json!({"device_id": "", "app_token": "t", "platform": "android"}),
        json!({"device_id": "d", "app_token": "  ", "platform": "android"}),
    ] {
        let resp = send(&app, "POST", "/api/v1/mobile/bff/push-tokens", Some(body)).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn an_observer_pass_needs_the_accreditation_it_rests_on() {
    // The old handler minted a pass for anyone who asked.
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/mobile/bff/observer/access-pass",
        Some(json!({"organisation": "Daily Trust", "application_id": "", "accreditation_ref": "acc-1"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(
        detail.contains("accreditation"),
        "the reason should say a pass rests on an accreditation: {detail}"
    );
}

#[tokio::test]
async fn an_observer_pass_names_its_application() {
    let app = app();
    let pass = body_json(
        send(
            &app,
            "POST",
            "/api/v1/mobile/bff/observer/access-pass",
            Some(json!({
                "organisation": "Daily Trust",
                "application_id": "app-123",
                "accreditation_ref": "acc-1"
            })),
        )
        .await,
    )
    .await;
    assert!(pass["access_pass"].as_str().unwrap().starts_with("PASS-"));
    assert!(
        pass["accreditation_ref"].as_str().unwrap().contains("app-123"),
        "the pass should name the application it rests on: {pass}"
    );
    assert_eq!(pass["status"], "active");
    assert!(pass["valid_until"].as_str().unwrap() > pass["issued_at"].as_str().unwrap());
}

#[tokio::test]
async fn routers_do_not_share_state() {
    let a = app();
    let b = app();
    send(
        &a,
        "POST",
        "/api/v1/mobile/bff/push-tokens",
        Some(json!({"device_id": "d", "app_token": "t", "platform": "android"})),
    )
    .await;
    let (_id, refresh) = create_session(&a).await;
    let resp = send(
        &b,
        "POST",
        "/api/v1/mobile/bff/sessions/refresh",
        Some(json!({"refresh_token": refresh})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "session state leaked");
}
