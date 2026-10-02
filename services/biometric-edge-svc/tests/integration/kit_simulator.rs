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

//! Integration test: the kit simulator submits synthetic enrolments through
//! edge-api, and verifier-side attestation verdicts come back from the same
//! router (WP-3.8).
//!
//! Hermetic by default: the full edge-api axum router is booted in process on
//! an ephemeral loopback port; no docker-compose stack is required. Set
//! EDGE_API_URL to run against a deployed instance instead.

use std::net::SocketAddr;

/// Boot the edge-api router on an ephemeral loopback port and return its base
/// URL. Honors a pre-set EDGE_API_URL for deployed-stack runs.
async fn edge_api_base_url() -> String {
    if let Ok(url) = std::env::var("EDGE_API_URL") {
        return url;
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("must bind an ephemeral loopback port");
    let addr: SocketAddr = listener.local_addr().expect("listener address");
    tokio::spawn(async move {
        axum::serve(listener, edge_api::app())
            .await
            .expect("edge-api server task must not crash");
    });
    format!("http://{addr}")
}

mod kit_simulator {
    use super::edge_api_base_url;

    /// Submits a synthetic enrolment through edge-api and asserts 202 response.
    #[tokio::test]
    async fn synthetic_enrolment_accepted() {
        let url = edge_api_base_url().await;

        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "idempotency_key": "kit-sim-001",
            "device_serial": "BVRD-KE-SIM-001",
            "capture_timestamp": chrono::Utc::now().to_rfc3339(),
            "biometric_ref": "v1:sha256:sim-fingerprint-ref",
            "modality": "fingerprint",
            "quality_score": 90,
            "device_sig": "sim-device-sig",
            "station_code": "001-001-001",
            "national_id_hash": "sha256:sim-national-id-hash"
        });

        let resp = client
            .post(format!("{url}/api/v1/edge/enrolment/upload"))
            .json(&body)
            .send()
            .await
            .expect("edge-api must be reachable");

        assert_eq!(resp.status(), 202, "expected 202 Accepted from edge-api");
        let json: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(json["status"], "accepted");
        assert!(json["enrolment_id"].as_str().is_some());
    }

    #[tokio::test]
    async fn attestation_verify_returns_verdict() {
        let url = edge_api_base_url().await;

        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "kind": "legacy_software",
            "token": "sim-hmac-valid-token",
            "device_serial": "BVRD-KE-SIM-001",
            "nonce": null
        });

        let resp = client
            .post(format!("{url}/api/v1/edge/attestation/verify"))
            .json(&body)
            .send()
            .await
            .expect("edge-api must be reachable");

        assert_eq!(resp.status(), 200);
        let json: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(json["integrity_level"], "MeetsBasic");
        assert_eq!(json["legacy_warning"], true);
    }
}
