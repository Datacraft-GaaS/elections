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

//! Contract tests for edge-api stub routes (MRZ, OTA, config, distress).
//!
//! Pins status codes + response shapes. MRZ/OTA/config handlers currently
//! return structured stub values (hardcoded WANJIKU fields, stub signatures)
//! pending integrations/mrz + bvi-svc proxy wiring; these tests lock the
//! shapes so that wiring cannot silently change the contract.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use edge_api::app;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn post_json(uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .uri(uri)
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn get(uri: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .uri(uri)
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp = app().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn healthz_ok() {
    let (status, body) = get("/healthz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn mrz_parse_shape() {
    let (status, body) = post_json(
        "/api/v1/edge/mrz/parse",
        json!({
            "mrz_raw": "P<KENWANJIKU<<GRACE<<<<<<<<<<<<<<<<<<<<\n12345678<8KEN9006154F2906153<<<<<<<<<<<<<<04",
            "device_serial": "dev-001"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    for key in ["surname", "national_id", "check_digits_valid"] {
        assert!(body.get(key).is_some(), "missing {key}");
    }
}

#[tokio::test]
async fn mrz_empty_rejected() {
    let (status, _) = post_json(
        "/api/v1/edge/mrz/parse",
        json!({ "mrz_raw": "", "device_serial": "dev-001" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ota_check_shape() {
    let (status, body) = post_json(
        "/api/v1/edge/ota/check",
        json!({ "device_serial": "dev-001", "current_version": "1.0.0", "kit_tier": "bvi-kit" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.get("manifest_sig").is_some() || body.get("update_available").is_some());
}

#[tokio::test]
async fn config_is_not_served_as_a_stub_signed_bundle() {
    // This asserted 200 with a `manifest_sig` field present, which is what pinned
    // the fabrication: the value in that field was a literal placeholder, and it
    // exists to carry an Ed25519 manifest signature a kit trusts.
    let (status, body) = get("/api/v1/biometric-edge/config").await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert!(body.get("manifest_sig").is_none());
    assert!(body.get("config").is_none());
}

#[tokio::test]
async fn distress_accepted() {
    let (status, _) = post_json(
        "/api/v1/edge/distress",
        json!({
            "device_serial": "dev-001",
            "station_code": "KE-NBI-001",
            "timestamp": "2026-09-18T00:00:00Z"
        }),
    )
    .await;
    // Handler MUST return 202 regardless of ECC reachability (EC-IDN-08):
    // any other code could surface to the UI and alert a coercer.
    assert_eq!(status, StatusCode::ACCEPTED);
}
