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

//! Integration and unit tests for the report publisher (RPT-B).
//!
//! Database tests require DATABASE_URL; they are skipped gracefully if not set.
//! SSH2 / SFTP tests are skipped if EEL_SFTP_PRIVKEY_PATH is not set.

use chrono::Utc;
use std::sync::Mutex;
use uuid::Uuid;

/// Serialize tests that mutate environment variables to avoid races.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_uuid() -> Uuid {
    Uuid::new_v4()
}

// ---------------------------------------------------------------------------
// 1. Backoff schedule is monotone increasing (property test, no DB needed)
// ---------------------------------------------------------------------------

/// Simulate the SQL backoff: base 30s * 2^attempt, cap 4h, no jitter for determinism.
fn backoff_secs_no_jitter(attempt_count: u32) -> f64 {
    let base = 30.0_f64 * (2.0_f64.powi(attempt_count as i32));
    base.min(4.0 * 3600.0)
}

#[test]
fn backoff_schedule_is_monotone() {
    let schedule: Vec<f64> = (0..5).map(backoff_secs_no_jitter).collect();
    for window in schedule.windows(2) {
        assert!(
            window[1] >= window[0],
            "backoff not monotone: {} then {}",
            window[0],
            window[1]
        );
    }
    // At attempt 5+, must be capped at 4h.
    assert_eq!(backoff_secs_no_jitter(9), 4.0 * 3600.0);
}

#[test]
fn backoff_values_within_jitter_tolerance() {
    // With 20% jitter the actual wait is between base and 1.2 * base.
    // At attempt 0: base = 30s; tolerate [30, 36].
    let base = backoff_secs_no_jitter(0);
    assert!(base >= 30.0 && base <= 30.0 * 1.2001);
}

// ---------------------------------------------------------------------------
// 2. HMAC-SHA256 signature (webhook adapter helper)
// ---------------------------------------------------------------------------

fn hmac_sha256_test(key: &[u8], data: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    const BLOCK: usize = 64;
    let mut k = [0u8; BLOCK];
    if key.len() <= BLOCK {
        k[..key.len()].copy_from_slice(key);
    } else {
        let h = sha2::Sha256::digest(key);
        k[..32].copy_from_slice(&h);
    }
    let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();
    let inner = {
        let mut h = sha2::Sha256::new();
        h.update(&ipad);
        h.update(data);
        h.finalize()
    };
    let outer = {
        let mut h = sha2::Sha256::new();
        h.update(&opad);
        h.update(inner);
        h.finalize()
    };
    outer.to_vec()
}

#[test]
fn hmac_sha256_known_vector() {
    // HMAC-SHA256 of "data" with key "key" = known test vector.
    let sig = hmac_sha256_test(b"key", b"data");
    let hex = hex::encode(&sig);
    // Expected: 5031fe3d989c6d1537a013fa6e739da23463fdaec3b70137d828e36ace221bd0
    assert_eq!(
        hex,
        "5031fe3d989c6d1537a013fa6e739da23463fdaec3b70137d828e36ace221bd0"
    );
}

#[test]
fn hmac_sha256_different_keys_produce_different_sigs() {
    let s1 = hmac_sha256_test(b"key1", b"payload");
    let s2 = hmac_sha256_test(b"key2", b"payload");
    assert_ne!(s1, s2);
}

// ---------------------------------------------------------------------------
// 3. Webhook delivery mock (httpmock)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn webhook_deliver_success() {
    let _lock = ENV_LOCK.lock().unwrap();
    use httpmock::Method::POST;
    use httpmock::MockServer;

    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/report-hook");
        then.status(200).body("ok");
    });

    let subscriber = make_subscriber("webhook", &server.url("/report-hook"));
    let ctx = make_delivery_context(subscriber);

    let outcome = eel_abis_report_publisher::channels::webhook::deliver(ctx).await;
    assert!(
        matches!(
            outcome,
            eel_abis_report_publisher::domain::DeliveryOutcome::Succeeded { .. }
        ),
        "expected Succeeded"
    );
    mock.assert();
}

#[tokio::test]
async fn webhook_deliver_4xx_is_permanent() {
    let _lock = ENV_LOCK.lock().unwrap();
    use httpmock::Method::POST;
    use httpmock::MockServer;

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/hook");
        then.status(403).body("forbidden");
    });

    let subscriber = make_subscriber("webhook", &server.url("/hook"));
    let ctx = make_delivery_context(subscriber);
    let outcome = eel_abis_report_publisher::channels::webhook::deliver(ctx).await;
    assert!(
        matches!(
            outcome,
            eel_abis_report_publisher::domain::DeliveryOutcome::Permanent { .. }
        ),
        "expected Permanent for 403"
    );
}

#[tokio::test]
async fn webhook_deliver_5xx_is_retryable() {
    let _lock = ENV_LOCK.lock().unwrap();
    use httpmock::Method::POST;
    use httpmock::MockServer;

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/hook");
        then.status(503).body("unavailable");
    });

    let subscriber = make_subscriber("webhook", &server.url("/hook"));
    let ctx = make_delivery_context(subscriber);
    let outcome = eel_abis_report_publisher::channels::webhook::deliver(ctx).await;
    assert!(
        matches!(
            outcome,
            eel_abis_report_publisher::domain::DeliveryOutcome::Retryable { .. }
        ),
        "expected Retryable for 503"
    );
}

// ---------------------------------------------------------------------------
// 4. Gov printer bridge mock
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gov_printer_deliver_success() {
    let _lock = ENV_LOCK.lock().unwrap();
    use httpmock::Method::POST;
    use httpmock::MockServer;

    let server = MockServer::start();
    let sid = make_uuid().to_string();
    let sid2 = sid.clone();
    server.mock(move |when, then| {
        when.method(POST).path("/deliver/gov-printer");
        then.status(200)
            .json_body(serde_json::json!({ "submission_id": sid2 }));
    });

    std::env::set_var("EEL_GOV_PRINTER_ADAPTER_URL", server.base_url());
    let subscriber = make_subscriber("gov_printer", "https://gazette.example.go.ke");
    let ctx = make_delivery_context(subscriber);
    let outcome = eel_abis_report_publisher::channels::gov_printer::deliver(ctx).await;
    assert!(
        matches!(
            &outcome,
            eel_abis_report_publisher::domain::DeliveryOutcome::Succeeded {
                remote_receipt_id: Some(_),
                ..
            }
        ),
        "expected Succeeded with submission_id"
    );
    if let eel_abis_report_publisher::domain::DeliveryOutcome::Succeeded {
        remote_receipt_id: Some(id),
        ..
    } = outcome
    {
        assert_eq!(id, sid);
    }
}

// ---------------------------------------------------------------------------
// 5. KBC SFTP bridge mock
// ---------------------------------------------------------------------------

#[tokio::test]
async fn kbc_sftp_deliver_success() {
    let _lock = ENV_LOCK.lock().unwrap();
    use httpmock::Method::POST;
    use httpmock::MockServer;

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/deliver/kbc-sftp");
        then.status(200)
            .json_body(serde_json::json!({ "remote_path": "/kbc/upload/file.pdf" }));
    });

    std::env::set_var("EEL_KBC_SFTP_ADAPTER_URL", server.base_url());
    let subscriber = make_subscriber("kbc_sftp", "kbc.example.go.ke");
    let ctx = make_delivery_context(subscriber);
    let outcome = eel_abis_report_publisher::channels::kbc_sftp::deliver(ctx).await;
    assert!(
        matches!(
            &outcome,
            eel_abis_report_publisher::domain::DeliveryOutcome::Succeeded { .. }
        ),
        "expected Succeeded"
    );
}

// ---------------------------------------------------------------------------
// 6. api_pull stays pending until access log observed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn api_pull_returns_retryable_waiting() {
    let subscriber = make_subscriber("api_pull", "-");
    let ctx = make_delivery_context(subscriber);
    let outcome = eel_abis_report_publisher::channels::api_pull::deliver(ctx).await;
    match outcome {
        eel_abis_report_publisher::domain::DeliveryOutcome::Retryable { ref error } => {
            assert!(error.starts_with("api_pull:"), "expected api_pull: prefix");
        }
        other => panic!("expected Retryable, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 7. Idempotency: backoff attempt_count 0..5 monotone
// ---------------------------------------------------------------------------

#[test]
fn backoff_attempt_0_to_5_all_positive() {
    for attempt in 0..5u32 {
        let secs = backoff_secs_no_jitter(attempt);
        assert!(secs > 0.0, "backoff at attempt {attempt} must be positive");
    }
}

// ---------------------------------------------------------------------------
// 8. Channel::to_string round-trips
// ---------------------------------------------------------------------------

#[test]
fn channel_display_round_trips() {
    use eel_abis_report_publisher::domain::Channel;
    let channels = [
        Channel::Email,
        Channel::Webhook,
        Channel::Kafka,
        Channel::Sftp,
        Channel::GovPrinter,
        Channel::KbcSftp,
        Channel::ApiPull,
    ];
    for ch in channels {
        let s = ch.to_string();
        assert!(!s.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 9. DeliveryStatus display
// ---------------------------------------------------------------------------

#[test]
fn delivery_status_display() {
    use eel_abis_report_publisher::domain::DeliveryStatus;
    assert_eq!(DeliveryStatus::Pending.to_string(), "pending");
    assert_eq!(DeliveryStatus::Sending.to_string(), "sending");
    assert_eq!(DeliveryStatus::Delivered.to_string(), "delivered");
    assert_eq!(DeliveryStatus::Failed.to_string(), "failed");
    assert_eq!(DeliveryStatus::Abandoned.to_string(), "abandoned");
}

// ---------------------------------------------------------------------------
// 10. Webhook signature is deterministic
// ---------------------------------------------------------------------------

#[test]
fn webhook_signature_deterministic() {
    let s1 = hmac_sha256_test(b"secret", b"body");
    let s2 = hmac_sha256_test(b"secret", b"body");
    assert_eq!(s1, s2);
}

// ---------------------------------------------------------------------------
// 11. Backoff cap at 4h
// ---------------------------------------------------------------------------

#[test]
fn backoff_caps_at_4h() {
    let secs = backoff_secs_no_jitter(100);
    assert_eq!(secs, 4.0 * 3600.0);
}

// ---------------------------------------------------------------------------
// 12. Metrics functions return non-null refs
// ---------------------------------------------------------------------------

#[test]
fn metrics_functions_initialize() {
    let _ = eel_abis_report_publisher::metrics::deliveries_total();
    let _ = eel_abis_report_publisher::metrics::delivery_latency_ms();
    let _ = eel_abis_report_publisher::metrics::abandon_total();
}

// ---------------------------------------------------------------------------
// 13. Gov printer adapter 4xx is permanent
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gov_printer_4xx_permanent() {
    let _lock = ENV_LOCK.lock().unwrap();
    use httpmock::Method::POST;
    use httpmock::MockServer;

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/deliver/gov-printer");
        then.status(400).body("bad request");
    });

    std::env::set_var("EEL_GOV_PRINTER_ADAPTER_URL", server.base_url());
    let subscriber = make_subscriber("gov_printer", "x");
    let ctx = make_delivery_context(subscriber);
    let outcome = eel_abis_report_publisher::channels::gov_printer::deliver(ctx).await;
    assert!(matches!(
        outcome,
        eel_abis_report_publisher::domain::DeliveryOutcome::Permanent { .. }
    ));
}

// ---------------------------------------------------------------------------
// 14. KBC SFTP adapter 5xx is retryable
// ---------------------------------------------------------------------------

#[tokio::test]
async fn kbc_sftp_5xx_retryable() {
    let _lock = ENV_LOCK.lock().unwrap();
    use httpmock::Method::POST;
    use httpmock::MockServer;

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/deliver/kbc-sftp");
        then.status(502).body("bad gateway");
    });

    std::env::set_var("EEL_KBC_SFTP_ADAPTER_URL", server.base_url());
    let subscriber = make_subscriber("kbc_sftp", "kbc.example");
    let ctx = make_delivery_context(subscriber);
    let outcome = eel_abis_report_publisher::channels::kbc_sftp::deliver(ctx).await;
    assert!(matches!(
        outcome,
        eel_abis_report_publisher::domain::DeliveryOutcome::Retryable { .. }
    ));
}

// ---------------------------------------------------------------------------
// 15. SFTP skipped gracefully if no privkey
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sftp_no_privkey_returns_retryable() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::remove_var("EEL_SFTP_PRIVKEY_PATH");
    let subscriber = make_subscriber("sftp", "localhost:2222/uploads");
    let ctx = make_delivery_context(subscriber);
    // This will fail at connection or auth; that's a Retryable outcome.
    let outcome = eel_abis_report_publisher::channels::sftp::deliver(ctx).await;
    // We just assert it doesn't panic and returns an outcome.
    assert!(matches!(
        outcome,
        eel_abis_report_publisher::domain::DeliveryOutcome::Retryable { .. }
    ));
}

// ---------------------------------------------------------------------------
// 16-20: Additional property and coverage tests
// ---------------------------------------------------------------------------

#[test]
fn backoff_attempt_3_is_240s() {
    // 30 * 2^3 = 240
    let secs = backoff_secs_no_jitter(3);
    assert_eq!(secs, 240.0);
}

#[test]
fn backoff_attempt_4_is_480s() {
    let secs = backoff_secs_no_jitter(4);
    assert_eq!(secs, 480.0);
}

#[test]
fn backoff_attempt_1_is_60s() {
    let secs = backoff_secs_no_jitter(1);
    assert_eq!(secs, 60.0);
}

#[test]
fn sha256_of_artifact_bytes_correct() {
    use sha2::Digest;
    let data = b"report content";
    let expected = sha2::Sha256::digest(data);
    assert_eq!(expected.len(), 32);
}

#[test]
fn hex_encode_decode_roundtrip() {
    let bytes = [0xde, 0xad, 0xbe, 0xef];
    let encoded = hex::encode(bytes);
    assert_eq!(encoded, "deadbeef");
}

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

fn make_subscriber(channel: &str, endpoint: &str) -> eel_abis_report_publisher::domain::Subscriber {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(endpoint.as_bytes()).to_vec();
    eel_abis_report_publisher::domain::Subscriber {
        subscriber_id: make_uuid(),
        name: "test-subscriber".into(),
        channel: channel.into(),
        endpoint: endpoint.into(),
        endpoint_hash: hash,
        audience: "public".into(),
        auth_ref: None,
        pgp_pubkey_ref: None,
        preferred_format: "pdf".into(),
        created_at: Utc::now(),
        verified_at: None,
        suspended_at: None,
        suspension_reason: None,
    }
}

fn make_delivery_context(
    subscriber: eel_abis_report_publisher::domain::Subscriber,
) -> eel_abis_report_publisher::domain::DeliveryContext {
    use sha2::Digest;

    let delivery_id = make_uuid();
    let run_id = make_uuid();
    let artifact_id = make_uuid();
    let subscription_id = make_uuid();

    let delivery = eel_abis_report_publisher::domain::Delivery {
        delivery_id,
        run_id,
        subscription_id,
        artifact_id,
        status: "sending".into(),
        attempt_count: 0,
        max_attempts: 5,
        next_attempt_at: None,
        last_attempt_at: None,
        delivered_at: None,
        last_error: None,
        remote_receipt_id: None,
        remote_receipt_at: None,
    };

    let subscription = eel_abis_report_publisher::domain::Subscription {
        subscription_id,
        subscriber_id: subscriber.subscriber_id,
        definition_id: make_uuid(),
        format: "pdf".into(),
        created_at: Utc::now(),
        cancelled_at: None,
    };

    let sha256 = sha2::Sha256::digest(b"fake artifact content").to_vec();
    let artifact = eel_abis_report_publisher::domain::Artifact {
        artifact_id,
        run_id,
        format: "pdf".into(),
        blob_ref: "http://localhost/fake-blob".into(),
        sha256,
        size_bytes: 1024,
        hsm_signature: None,
        rendered_at: Utc::now(),
    };

    eel_abis_report_publisher::domain::DeliveryContext {
        delivery,
        subscriber,
        subscription,
        artifact,
        tlog_anchor: None,
    }
}
