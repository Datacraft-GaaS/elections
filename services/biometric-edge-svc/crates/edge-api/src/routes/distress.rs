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

//! POST /api/v1/edge/distress
//!
//! Silent duress alert (EC-IDN-08). The kit triggers this when the officer
//! presents the designated distress biometric. The UI proceeds normally; this
//! endpoint routes a covert alert to the ECC (Emergency Coordination Centre).
//! The handler MUST return 202 regardless of ECC reachability to avoid
//! UI anomalies that could alert a coercer.
use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct DistressRequest {
    pub device_serial: String,
    pub station_code: String,
    pub timestamp: String,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
    /// Opaque encrypted distress context — decrypted only by ECC.
    pub encrypted_context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DistressResponse {
    /// Always true; kit must not surface this status to the UI.
    pub received: bool,
}

pub fn router() -> Router {
    Router::new().route("/api/v1/edge/distress", post(distress))
}

async fn distress(
    Json(req): Json<DistressRequest>,
) -> Result<(StatusCode, Json<DistressResponse>), ApiError> {
    // Always accept; any error is suppressed to prevent UI leak.
    tracing::warn!(
        device_serial = %req.device_serial,
        station_code = %req.station_code,
        timestamp = %req.timestamp,
        "DISTRESS ALERT RECEIVED - routing to ECC"
    );
    // Production: emit kit_distress.v1 outbox event + fire ECC webhook with retry.
    // Failure must not propagate to the kit.
    Ok((
        StatusCode::ACCEPTED,
        Json(DistressResponse { received: true }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn distress_returns_202() {
        let app = router();
        let body = serde_json::json!({
            "device_serial": "BVRD-KE-00001",
            "station_code": "001-001-001",
            "timestamp": "2026-08-21T09:00:00Z",
            "gps_lat": -1.2921,
            "gps_lon": 36.8219
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/distress")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }
}
