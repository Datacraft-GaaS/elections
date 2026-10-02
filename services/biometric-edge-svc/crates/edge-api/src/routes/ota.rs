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

//! POST /api/v1/edge/ota/check — returns the latest signed OTA manifest for
//! the kit tier. Kit downloads the artifact via the returned URL.
use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct OtaCheckRequest {
    pub device_serial: String,
    /// Current firmware version reported by the kit.
    pub current_version: String,
    /// Kit tier (bvr-kit | bvi-kit | legacy-android).
    pub kit_tier: String,
}

#[derive(Debug, Serialize)]
pub struct OtaCheckResponse {
    pub update_available: bool,
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
    /// Ed25519 signature over the manifest JSON.
    pub manifest_sig: Option<String>,
}

pub fn router() -> Router {
    Router::new().route("/api/v1/edge/ota/check", post(ota_check))
}

async fn ota_check(
    Json(req): Json<OtaCheckRequest>,
) -> Result<(StatusCode, Json<OtaCheckResponse>), ApiError> {
    if req.device_serial.is_empty() {
        return Err(ApiError::BadRequest("device_serial required".into()));
    }
    let known_latest = match req.kit_tier.as_str() {
        "bvr-kit" => "1.0.1",
        "bvi-kit" => "1.0.1",
        "legacy-android" => "1.0.0",
        _ => return Err(ApiError::BadRequest(format!("unknown kit_tier: {}", req.kit_tier))),
    };
    let update_available = req.current_version != known_latest;
    Ok((
        StatusCode::OK,
        Json(OtaCheckResponse {
            update_available,
            latest_version: if update_available { Some(known_latest.into()) } else { None },
            download_url: if update_available {
                Some(format!(
                    "https://artifacts.eel.local/ota/{}/{known_latest}.zip",
                    req.kit_tier
                ))
            } else {
                None
            },
            manifest_sig: if update_available {
                Some("stub-ed25519-ota-sig".into())
            } else {
                None
            },
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn ota_check_update_available() {
        let app = router();
        let body = serde_json::json!({
            "device_serial": "BVRD-KE-00001",
            "current_version": "1.0.0",
            "kit_tier": "bvr-kit"
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/ota/check")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
