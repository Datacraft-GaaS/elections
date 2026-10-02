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

//! A Form 34A countersignature is actually checked.
//!
//! `verify_form34a` used to discard `ro_signature`, log the RO id, and answer
//! `status: "verified"`. So a countersignature was never read and the answer said
//! it had been -- and "verified" on a form a returning officer signs is the most
//! consequential string on the endpoint.
//!
//! These tests install a real Ed25519 trust anchor, so the positive path is a
//! genuine signature verification rather than a stub.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use rts_ro_api::{handlers, ro_trust::RoTrust};

/// The verifying key for a signing key, as hex.
fn public_hex(sk: &ed25519_dalek::SigningKey) -> String {
    hex::encode(eel_crypto_rs::ed25519::public_key(sk))
}

/// A countersignature over `message`, as hex.
fn sign_hex(sk: &ed25519_dalek::SigningKey, message: &[u8]) -> String {
    hex::encode(eel_crypto_rs::ed25519::sign(sk, message).to_bytes())
}

async fn post(trust: RoTrust, uri: &str, body: Value) -> (StatusCode, Value) {
    let app = rts_ro_api::router::build_router_with(trust.shared());
    let resp = app
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

fn request(ro_id: &str, signature: &str) -> Value {
    json!({
        "ro_id": ro_id,
        "ro_signature": signature,
        "idempotency_key": "018f6c2e-0000-7000-8000-0000000000aa",
        "verified_at": "2027-08-21T09:00:00Z"
    })
}

const FORM: &str = "018f6c2e-0000-7000-8000-00000000beef";

#[tokio::test]
async fn a_real_countersignature_verifies() {
    let kp = eel_crypto_rs::ed25519::generate_keypair();
    let ro_id = "RO-0001";
    let message = handlers::countersigned_message(
        &FORM.parse().unwrap(),
        ro_id,
        "2027-08-21T09:00:00Z",
    );
    let sig = sign_hex(&kp, message.as_bytes());

    let (status, body) = post(
        RoTrust::new().trust(ro_id, public_hex(&kp)),
        &format!("/api/v1/rts/form34a/{FORM}/verify"),
        request(ro_id, &sig),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["status"], "verified");
}

#[tokio::test]
async fn a_countersignature_over_a_different_form_does_not_verify() {
    let kp = eel_crypto_rs::ed25519::generate_keypair();
    let ro_id = "RO-0002";
    // Signed over a different form id.
    let other = "018f6c2e-0000-7000-8000-0000000000ff";
    let message = handlers::countersigned_message(
        &other.parse().unwrap(),
        ro_id,
        "2027-08-21T09:00:00Z",
    );
    let sig = sign_hex(&kp, message.as_bytes());

    let (status, body) = post(
        RoTrust::new().trust(ro_id, public_hex(&kp)),
        &format!("/api/v1/rts/form34a/{FORM}/verify"),
        request(ro_id, &sig),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["detail"].as_str().unwrap().contains("does not verify"));
}

#[tokio::test]
async fn a_counter_signature_from_another_officer_does_not_verify() {
    let mine = eel_crypto_rs::ed25519::generate_keypair();
    let theirs = eel_crypto_rs::ed25519::generate_keypair();
    let ro_id = "RO-0003";
    // The trust anchor holds *my* key, but the signature is from another HSM.
    let message = handlers::countersigned_message(
        &FORM.parse().unwrap(),
        ro_id,
        "2027-08-21T09:00:00Z",
    );
    let sig = sign_hex(&theirs, message.as_bytes());

    let (status, _) = post(
        RoTrust::new().trust(ro_id, public_hex(&mine)),
        &format!("/api/v1/rts/form34a/{FORM}/verify"),
        request(ro_id, &sig),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn with_no_trust_anchor_it_says_it_cannot_verify() {
    // The default state: no RO HSM keys are provisioned in this workspace. It must
    // refuse rather than answer "verified", which is what it used to do for every
    // request regardless of the signature.
    let kp = eel_crypto_rs::ed25519::generate_keypair();
    let ro_id = "RO-UNPROVISIONED";
    let message = handlers::countersigned_message(
        &FORM.parse().unwrap(),
        ro_id,
        "2027-08-21T09:00:00Z",
    );
    let sig = sign_hex(&kp, message.as_bytes());
    // An empty trust store: this officer is not provisioned, which is the state
    // the process ships in.
    let (status, body) = post(
        RoTrust::new(),
        &format!("/api/v1/rts/form34a/{FORM}/verify"),
        request(ro_id, &sig),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body["detail"].as_str().unwrap().contains("trust anchor"));
}

#[tokio::test]
async fn an_absent_or_malformed_signature_is_a_client_error() {
    let kp = eel_crypto_rs::ed25519::generate_keypair();
    for sig in ["", "   ", "not-hex", &"ab".repeat(40)] {
        let (status, _) = post(
            RoTrust::new().trust("RO-0004", public_hex(&kp)),
            &format!("/api/v1/rts/form34a/{FORM}/verify"),
            request("RO-0004", sig),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "signature {sig:?} must be refused");
    }
}
