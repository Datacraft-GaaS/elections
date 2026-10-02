//! Meta's webhook signature is actually verified.
//!
//! `whatsapp_webhook` refused with "signature verification is not implemented",
//! which was honest — but the *algorithm* was available (`hmac` and `sha2` are
//! already dependencies, and the same check is implemented in `verifier-api`). What
//! is missing is the key, not the capability. So the signature is now really
//! checked, and with no app secret configured the endpoint says it cannot verify.
//!
//! It used to answer 200 OK for any payload at all while discarding it.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ecc_api::{router::build_router_with, store::Store};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tower::ServiceExt;

type HmacSha256 = Hmac<Sha256>;

fn app(secret: Option<&str>) -> axum::Router {
    build_router_with(
        Store::Mem(Default::default()),
        secret.map(str::to_string),
    )
}

/// The signature Meta would send for this body.
fn sign(secret: &str, body: &[u8]) -> String {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

async fn post(router: axum::Router, sig: Option<&str>, body: &str) -> StatusCode {
    let mut b = Request::builder()
        .uri("/grievance/whatsapp/webhook")
        .method("POST")
        .header("content-type", "application/json");
    if let Some(sig) = sig {
        b = b.header("x-hub-signature-256", sig);
    }
    let resp = router
        .oneshot(b.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    resp.status()
}

const BODY: &str = r#"{"object":"whatsapp_business_account","entry":[]}"#;

#[tokio::test]
async fn a_correctly_signed_payload_is_accepted() {
    let sig = sign("app-secret", BODY.as_bytes());
    assert_eq!(post(app(Some("app-secret")), Some(&sig), BODY).await, StatusCode::OK);
}

#[tokio::test]
async fn a_signature_over_a_different_body_is_refused() {
    let sig = sign("app-secret", br#"{"object":"something_else"}"#);
    assert_eq!(post(app(Some("app-secret")), Some(&sig), BODY).await, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_signature_from_a_different_secret_is_refused() {
    let sig = sign("other-secret", BODY.as_bytes());
    assert_eq!(post(app(Some("app-secret")), Some(&sig), BODY).await, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_unsigned_payload_is_refused() {
    // This is what the old handler answered 200 to.
    assert_eq!(post(app(Some("app-secret")), None, BODY).await, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_header_without_the_sha256_prefix_is_refused() {
    let bare = sign("app-secret", BODY.as_bytes()).trim_start_matches("sha256=").to_string();
    assert_eq!(post(app(Some("app-secret")), Some(&bare), BODY).await, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn with_no_app_secret_it_says_it_cannot_verify() {
    // The state the service ships in: the key is issued by Meta and is not here.
    // "Unverifiable" is the honest answer and is distinct from "valid".
    assert_eq!(post(app(None), None, BODY).await, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn a_signed_but_malformed_payload_is_a_client_error() {
    let sig = sign("app-secret", b"not json");
    assert_eq!(post(app(Some("app-secret")), Some(&sig), "not json").await, StatusCode::BAD_REQUEST);
}
