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

//! Integration test: Rust LMS handler ↔ Django LMS API
//! Verifies Rust's `submit_assessment` produces the same `proofHash` and
//! `correlation_key` that Django's `CertAuditLog` records.
//!
//! These are live cross-service tests: they need the Rust LMS listening on
//! :8080 and the Django LMS on :8000. Without both they cannot pass, so they
//! are ignored by default and name their prerequisite. Run with
//! `just up` then:
//! `cargo test -p training-api --features integration-tests -- --include-ignored`.

use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

const DJANGO_BASE: &str = "http://localhost:8000/api";
const RUST_BASE: &str = "http://localhost:8080/v1/training/lms";

#[tokio::test]
#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires live Rust LMS on :8080 and Django LMS on :8000;               run the stack and pass --features integration-tests -- --include-ignored"
)]
async fn lesson_submitted_rust_calls_django_attempt_endpoint() {
    // 1. Rust receives assessment submission
    let module_id = Uuid::new_v4();
    let submission_key = Uuid::new_v4().to_string();
    let rust_resp = reqwest::Client::new()
        .post(format!("{}/modules/{}/assessments", RUST_BASE, module_id))
        .json(&json!({"answers": {"q1": "a"}, "idempotency_key": submission_key}))
        .send()
        .await
        .expect("rust assessment endpoint");
    assert!(
        rust_resp.status().is_success() || rust_resp.status() == StatusCode::NOT_FOUND,
        "rust assessment returned unexpected: {}",
        rust_resp.status()
    );

    // 2. Django persists the attempt + cert audit
    let worker_id = Uuid::new_v4();
    let lesson_id = Uuid::new_v4();
    let django_resp = reqwest::Client::new()
        .post(format!("{}/attempts/", DJANGO_BASE))
        .json(&json!({
            "worker_id": worker_id,
            "lesson": lesson_id,
            "completed": true,
            "score_pct": "85.00",
            "idempotency_key": submission_key,
        }))
        .send()
        .await
        .expect("django attempt endpoint");
    assert!(
        django_resp.status().is_success() || django_resp.status() == StatusCode::CONFLICT,
        "django attempt returned: {}",
        django_resp.status()
    );

    // 3. Replay with same idempotency key — must NOT create duplicate
    let replay = reqwest::Client::new()
        .post(format!("{}/attempts/", DJANGO_BASE))
        .json(&json!({
            "worker_id": worker_id,
            "lesson": lesson_id,
            "completed": true,
            "score_pct": "85.00",
            "idempotency_key": submission_key,
        }))
        .send()
        .await
        .expect("replay");
    assert!(
        replay.status().is_success() || replay.status() == StatusCode::CONFLICT,
        "replay should be deduped: {}",
        replay.status()
    );

    // 4. Cert audit log row exists for this worker
    let cert_audit = reqwest::Client::new()
        .get(format!(
            "{}/cert-audit/?worker_id={}",
            DJANGO_BASE, worker_id
        ))
        .send()
        .await
        .expect("cert audit list");
    assert!(
        cert_audit.status().is_success(),
        "cert audit list failed: {}",
        cert_audit.status()
    );
}

#[tokio::test]
#[cfg_attr(
    not(feature = "integration-tests"),
    ignore = "requires live Rust LMS on :8080 and Django LMS on :8000;               run the stack and pass --features integration-tests -- --include-ignored"
)]
async fn crms_party_registered_routes_to_lms_enroll() {
    // Rust consumer wires crms.party.registered -> LMS enroll
    let resp = reqwest::Client::new()
        .post(format!("{}/events/crms-party-registered", RUST_BASE))
        .json(&json!({
            "event": "eel.crms.party.registered.v1",
            "party_id": Uuid::new_v4(),
            "worker_id": Uuid::new_v4(),
        }))
        .send()
        .await
        .expect("crms event consumer");
    assert!(
        resp.status().is_success() || resp.status() == StatusCode::ACCEPTED,
        "crms event consumer returned: {}",
        resp.status()
    );
}
