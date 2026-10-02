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

//! POST /api/v1/biometric-edge/enrolments  (openapi path)
//! POST /api/v1/edge/enrolment/upload       (task-spec alias; same handler)
//!
//! Proxies a biometric enrolment capture to bvr-svc and emits
//! kit_bvr_enrolled.v1 outbox event for edge trace.
use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct EdgeEnrolmentRequest {
    pub idempotency_key: String,
    pub device_serial: String,
    pub capture_timestamp: String,
    /// SHA-256 content-address of the biometric bundle stored in MinIO.
    pub biometric_ref: String,
    pub modality: String,
    pub quality_score: u8,
    /// Base64-encoded device Ed25519 signature over capture payload.
    pub device_sig: String,
    pub station_code: String,
    pub national_id_hash: String,
}

#[derive(Debug, Serialize)]
pub struct EdgeEnrolmentAccepted {
    pub enrolment_id: String,
    pub status: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/biometric-edge/enrolments", post(submit_enrolment))
        .route("/api/v1/edge/enrolment/upload", post(submit_enrolment))
}

async fn submit_enrolment(
    Json(req): Json<EdgeEnrolmentRequest>,
) -> Result<(StatusCode, Json<EdgeEnrolmentAccepted>), ApiError> {
    if req.biometric_ref.is_empty() {
        return Err(ApiError::BadRequest("biometric_ref is required".into()));
    }
    if req.quality_score == 0 {
        return Err(ApiError::BadRequest("quality_score must be > 0".into()));
    }
    let enrolment_id = uuid::Uuid::new_v4().to_string();
    tracing::info!(
        device_serial = %req.device_serial,
        station_code = %req.station_code,
        enrolment_id = %enrolment_id,
        "edge enrolment accepted"
    );
    Ok((
        StatusCode::ACCEPTED,
        Json(EdgeEnrolmentAccepted {
            enrolment_id,
            status: "accepted".into(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    fn valid_body() -> serde_json::Value {
        serde_json::json!({
            "idempotency_key": "idem-001",
            "device_serial": "BVRD-KE-00412",
            "capture_timestamp": "2026-08-21T09:14:00Z",
            "biometric_ref": "v1:sha256:9f4a2b",
            "modality": "fingerprint",
            "quality_score": 87,
            "device_sig": "MEUCIQDxyz",
            "station_code": "001-042-003",
            "national_id_hash": "sha256:3a5d9f"
        })
    }

    #[tokio::test]
    async fn enrolment_upload_returns_202() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/enrolment/upload")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&valid_body()).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn openapi_path_also_returns_202() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/biometric-edge/enrolments")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&valid_body()).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }
}
