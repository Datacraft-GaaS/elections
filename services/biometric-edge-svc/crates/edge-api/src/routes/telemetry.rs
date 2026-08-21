//! POST /api/v1/biometric-edge/heartbeat  (openapi path)
//! POST /api/v1/edge/telemetry             (task-spec alias)
//!
//! Kit heartbeats and geofence context. Emits kit_heartbeat.v1 outbox event.
use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct TelemetryRequest {
    pub device_serial: String,
    pub station_code: String,
    pub timestamp: String,
    pub battery_pct: Option<u8>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
    /// GNSS fix quality: 0=no fix, 1=autonomous, 2=differential, 4=RTK.
    pub gnss_fix_quality: Option<u8>,
    pub network_rssi_dbm: Option<i16>,
    pub uptime_s: Option<u64>,
    pub enrolled_count: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct TelemetryResponse {
    pub received: bool,
    pub server_time: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/biometric-edge/heartbeat", post(telemetry))
        .route("/api/v1/edge/telemetry", post(telemetry))
}

async fn telemetry(
    Json(req): Json<TelemetryRequest>,
) -> Result<(StatusCode, Json<TelemetryResponse>), ApiError> {
    if req.device_serial.is_empty() {
        return Err(ApiError::BadRequest("device_serial required".into()));
    }
    tracing::debug!(
        device_serial = %req.device_serial,
        battery_pct = ?req.battery_pct,
        "kit heartbeat"
    );
    // Production: persist to kit_heartbeats table + emit kit_heartbeat.v1 outbox event.
    let server_time = chrono::Utc::now().to_rfc3339();
    Ok((StatusCode::ACCEPTED, Json(TelemetryResponse { received: true, server_time })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn telemetry_returns_202() {
        let app = router();
        let body = serde_json::json!({
            "device_serial": "BVRD-KE-00001",
            "station_code": "001-001-001",
            "timestamp": "2026-08-21T10:00:00Z",
            "battery_pct": 85
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/telemetry")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn heartbeat_openapi_path_returns_202() {
        let app = router();
        let body = serde_json::json!({
            "device_serial": "BVRD-KE-00001",
            "station_code": "001-001-001",
            "timestamp": "2026-08-21T10:00:00Z"
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/biometric-edge/heartbeat")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }
}
