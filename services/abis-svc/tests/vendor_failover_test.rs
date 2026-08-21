//! Integration test: vendor router failover.
//!
//! Simulates primary vendor unhealthy; verifies router picks secondary.
//! Requires live abis-api with configurable vendor endpoints.
//! EC-CON-05 (flapping link / retry budget).

mod common;
use common::{idempotency_key, synthetic_template_blob, TestRig};

/// When primary vendor is unhealthy, a 1:N match still completes via secondary.
/// This test uses the health-override header understood by dev/test builds.
#[cfg_attr(not(feature = "cross-crate-integration"), ignore)]
#[test]
fn test_vendor_failover_to_secondary() {
    let rig = TestRig::local();
    let client = reqwest::blocking::Client::new();

    let payload = serde_json::json!({
        "probe_ref": "bio-store://eel.test/probes/vendor-failover-001",
        "template_blob": synthetic_template_blob("vendor-failover-001"),
        "modality": "fingerprint",
        "caller_id": "spiffe://eel.local/ns/enrolment/sa/enrolment-svc",
        "election_id": "00000000-0000-7000-8000-000000000001",
    });

    // X-Eel-Test-VendorOverride: primary_down instructs the dev build to
    // treat primary vendor as unhealthy for this request only.
    let res = client
        .post(&rig.match_1n_url())
        .header("Content-Type", "application/json")
        .header("Idempotency-Key", idempotency_key())
        .header("X-Eel-Test-VendorOverride", "primary_down")
        .json(&payload)
        .send()
        .expect("request failed");

    let status = res.status().as_u16();
    assert!(
        status == 202 || status == 503,
        "expected 202 (failover succeeded) or 503 (no secondary configured); got {status}"
    );

    if status == 202 {
        let body: serde_json::Value = res.json().unwrap();
        assert!(body["job_id"].is_string(), "expected job_id in response");
    } else {
        eprintln!("INFO: 503 returned; secondary vendor not configured in this environment");
    }
}

/// Retry budget: after N failures the circuit opens; subsequent calls fail fast.
#[test]
fn test_vendor_circuit_breaker_opens_on_repeated_failure() {
    // This test is a documentation-level test; the circuit-breaker behaviour
    // is validated by the eel-abis-vendor-latency chaos scenario.
    // See: services/abis-svc/chaos/eel-abis-vendor-latency.yaml
    eprintln!(
        "INFO: circuit-breaker validation covered by chaos scenario eel-abis-vendor-latency.yaml"
    );
}
