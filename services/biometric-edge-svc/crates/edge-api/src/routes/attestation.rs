//! POST /api/v1/biometric-edge/attestation  (openapi path)
//! POST /api/v1/edge/attestation/verify      (task-spec alias)
//!
//! Delegates to kit-attest crate in-process.
use axum::{http::StatusCode, routing::post, Json, Router};
use kit_attest::{verify_attestation, AttestError, AttestationPayload};
use serde::Serialize;

use crate::error::ApiError;

#[derive(Debug, Serialize)]
pub struct AttestationResponse {
    pub device_id: String,
    pub tier: String,
    pub expires_at: String,
    pub integrity_level: String,
    pub kit_cert_thumbprint: String,
    pub legacy_warning: bool,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/biometric-edge/attestation", post(attest))
        .route("/api/v1/edge/attestation/verify", post(attest))
}

async fn attest(
    Json(payload): Json<AttestationPayload>,
) -> Result<(StatusCode, Json<AttestationResponse>), ApiError> {
    let verdict = verify_attestation(&payload).map_err(|e| match e {
        AttestError::Tampered(m) | AttestError::InvalidSignature(m) => {
            ApiError::Forbidden(format!("attestation rejected: {m}"))
        }
        AttestError::Expired => ApiError::Forbidden("attestation expired".into()),
        AttestError::Unsupported => ApiError::BadRequest("unsupported attestation kind".into()),
        AttestError::Internal(m) => ApiError::Internal(m),
    })?;

    Ok((
        StatusCode::OK,
        Json(AttestationResponse {
            device_id: verdict.device_id,
            tier: format!("{:?}", verdict.tier),
            expires_at: verdict.expires_at.to_rfc3339(),
            integrity_level: format!("{:?}", verdict.integrity_level),
            kit_cert_thumbprint: verdict.kit_cert_thumbprint,
            legacy_warning: verdict.legacy_warning,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    fn valid_payload() -> serde_json::Value {
        serde_json::json!({
            "kind": "play_integrity",
            "token": "valid-integrity-token",
            "device_serial": "BVRD-KE-00412",
            "nonce": "nonce-abc"
        })
    }

    #[tokio::test]
    async fn attestation_verify_returns_200() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/attestation/verify")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&valid_payload()).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn tampered_attestation_returns_403() {
        let app = router();
        let payload = serde_json::json!({
            "kind": "play_integrity",
            "token": "TAMPERED:evil",
            "device_serial": "BVRD-KE-00001",
            "nonce": null
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/attestation/verify")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
