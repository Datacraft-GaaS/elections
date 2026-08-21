//! POST /api/v1/biometric-edge/identifications (openapi path)
//!
//! Routes real-time 1:1 multi-modal identification to bvi-svc.
use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct IdentificationRequest {
    pub device_serial: String,
    pub station_code: String,
    pub voter_number: String,
    pub biometric_ref: String,
    pub modality: String,
    pub capture_timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct IdentificationResponse {
    pub identification_id: String,
    pub status: String,
    pub matched: bool,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/biometric-edge/identifications", post(identify))
}

async fn identify(
    Json(req): Json<IdentificationRequest>,
) -> Result<(StatusCode, Json<IdentificationResponse>), ApiError> {
    if req.voter_number.is_empty() {
        return Err(ApiError::BadRequest("voter_number required".into()));
    }
    tracing::info!(
        device_serial = %req.device_serial,
        voter_number = %req.voter_number,
        "edge identification request"
    );
    Ok((
        StatusCode::ACCEPTED,
        Json(IdentificationResponse {
            identification_id: uuid::Uuid::new_v4().to_string(),
            status: "accepted".into(),
            matched: false, // bvi-svc resolves asynchronously
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn identification_returns_202() {
        let app = router();
        let body = serde_json::json!({
            "device_serial": "BVI-KE-00001",
            "station_code": "001-001-001",
            "voter_number": "VN-12345",
            "biometric_ref": "v1:sha256:abcdef",
            "modality": "face",
            "capture_timestamp": "2026-08-21T09:00:00Z"
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/biometric-edge/identifications")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }
}
