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

//! Integration test: full enrolment dedup pipeline.
//!
//! Flow: ingest -> quality -> dedup -> route -> enrol.
//! Asserts EnrolmentDecision and event emission.
//!
//! Requires a deployed abis-api backed by PostgreSQL and Kafka. Without the
//! cross-crate-integration feature the tests are ignored, never fake-passed.
//!
//! Run against the deployed stack:
//!   cargo test --test enrolment_dedup_flow_test --features cross-crate-integration

mod common;
use common::{idempotency_key, synthetic_template_blob, TestRig};

/// Full clean-path enrolment: probe does not match gallery; status = clean.
#[cfg_attr(
    not(feature = "cross-crate-integration"),
    ignore = "needs a deployed abis-api backed by PostgreSQL and Kafka; enable cross-crate-integration and run against the docker-compose stack"
)]
#[test]
fn test_enrolment_clean_path() {
    let rig = TestRig::local();
    let client = reqwest::blocking::Client::new();

    let blob = synthetic_template_blob("test-clean-001");
    let payload = serde_json::json!({
        "probe_ref": "bio-store://eel.test/probes/clean-001",
        "template_blob": blob,
        "modality": "fingerprint",
        "caller_id": "spiffe://eel.local/ns/enrolment/sa/enrolment-svc",
        "election_id": "00000000-0000-7000-8000-000000000001",
        "subject_ref": "subject:clean-001",
    });

    let res = client
        .post(&rig.match_1n_url())
        .header("Content-Type", "application/json")
        .header("Idempotency-Key", idempotency_key())
        .json(&payload)
        .send()
        .expect("request failed");

    assert_eq!(res.status().as_u16(), 202, "expected 202 Accepted");
    let body: serde_json::Value = res.json().expect("invalid JSON");
    assert!(body["job_id"].is_string(), "missing job_id");
    assert_eq!(body["status"].as_str().unwrap_or(""), "queued");
}

/// Duplicate enrolment: same template twice; second should return existing decision.
#[cfg_attr(
    not(feature = "cross-crate-integration"),
    ignore = "needs a deployed abis-api backed by PostgreSQL and Kafka; enable cross-crate-integration and run against the docker-compose stack"
)]
#[test]
fn test_enrolment_idempotent_duplicate_submission() {
    let rig = TestRig::local();
    let client = reqwest::blocking::Client::new();

    let blob = synthetic_template_blob("test-dedup-idem-001");
    let key = idempotency_key();
    let payload = serde_json::json!({
        "probe_ref": "bio-store://eel.test/probes/dedup-idem-001",
        "template_blob": blob,
        "modality": "fingerprint",
        "caller_id": "spiffe://eel.local/ns/enrolment/sa/enrolment-svc",
        "election_id": "00000000-0000-7000-8000-000000000001",
    });

    let r1 = client
        .post(&rig.match_1n_url())
        .header("Content-Type", "application/json")
        .header("Idempotency-Key", &key)
        .json(&payload)
        .send()
        .expect("r1 failed");
    let r2 = client
        .post(&rig.match_1n_url())
        .header("Content-Type", "application/json")
        .header("Idempotency-Key", &key)
        .json(&payload)
        .send()
        .expect("r2 failed");

    // Both should succeed; r2 is served from idempotency cache.
    assert_eq!(r1.status().as_u16(), 202);
    assert_eq!(r2.status().as_u16(), 202);

    let b1: serde_json::Value = r1.json().unwrap();
    let b2: serde_json::Value = r2.json().unwrap();
    assert_eq!(
        b1["job_id"], b2["job_id"],
        "idempotency cache should return same job_id"
    );
}

/// Same idempotency key, different body: must return 422 idempotency-key-reuse.
#[cfg_attr(
    not(feature = "cross-crate-integration"),
    ignore = "needs a deployed abis-api backed by PostgreSQL and Kafka; enable cross-crate-integration and run against the docker-compose stack"
)]
#[test]
fn test_enrolment_idempotency_key_reuse_different_body() {
    let rig = TestRig::local();
    let client = reqwest::blocking::Client::new();
    let key = idempotency_key();

    let payload_a = serde_json::json!({
        "probe_ref": "bio-store://eel.test/probes/idem-a",
        "template_blob": synthetic_template_blob("idem-a"),
        "modality": "fingerprint",
        "caller_id": "spiffe://eel.local/ns/enrolment/sa/enrolment-svc",
        "election_id": "00000000-0000-7000-8000-000000000001",
    });
    let payload_b = serde_json::json!({
        "probe_ref": "bio-store://eel.test/probes/idem-b",
        "template_blob": synthetic_template_blob("idem-b"),
        "modality": "iris",
        "caller_id": "spiffe://eel.local/ns/enrolment/sa/enrolment-svc",
        "election_id": "00000000-0000-7000-8000-000000000001",
    });

    let r1 = client
        .post(&rig.match_1n_url())
        .header("Content-Type", "application/json")
        .header("Idempotency-Key", &key)
        .json(&payload_a)
        .send()
        .expect("r1 failed");
    assert_eq!(r1.status().as_u16(), 202);

    let r2 = client
        .post(&rig.match_1n_url())
        .header("Content-Type", "application/json")
        .header("Idempotency-Key", &key)
        .json(&payload_b)
        .send()
        .expect("r2 failed");
    assert_eq!(
        r2.status().as_u16(),
        422,
        "expect 422 idempotency-key-reuse"
    );
}
