//! Integration test: kit simulator submits synthetic enrolments through edge-api,
//! bvr-svc receives, verifier-rs verifies signature.
//!
//! These tests require the full service stack; they run via docker-compose in CI.
//! Set EDGE_API_URL + BVR_SVC_URL env vars before running.

#[cfg(test)]
mod kit_simulator {
    /// Submits a synthetic enrolment through edge-api and asserts 202 response.
    ///
    /// In CI this hits a live edge-api instance. Locally it can be run against
    /// a docker-compose stack with `cargo test -p edge-api --test kit_simulator`.
    #[tokio::test]
    #[ignore = "requires live edge-api stack; run with --include-ignored in CI"]
    async fn synthetic_enrolment_accepted() {
        let url = std::env::var("EDGE_API_URL")
            .unwrap_or_else(|_| "http://localhost:8090".into());

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
    #[ignore = "requires live edge-api stack"]
    async fn attestation_verify_returns_verdict() {
        let url = std::env::var("EDGE_API_URL")
            .unwrap_or_else(|_| "http://localhost:8090".into());

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
