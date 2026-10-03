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

//! Integration / property tests for eel-field-mobile-telemetry-ingest.
//!
//! Target: >= 35 tests. No real DB required; all tests exercise pure logic.

use chrono::{Duration, Utc};
use eel_field_mobile_telemetry_ingest::{
    alert_engine::{evaluate, AlertCandidate},
    batch::{ingest_batch, BatchItemResult},
    domain::{
        AlertClass, ExitReason, MotionClass, Severity, SignalClass, TelemetrySession,
        TelemetrySignal,
    },
    dwell_computer::{compute_dwells, LocationPing},
    error::TelemetryError,
    events::{
        telemetry_alert_data, telemetry_session_closed_data, telemetry_session_opened_data,
        telemetry_signal_data, TELEMETRY_ALERT_V1, TELEMETRY_SESSION_CLOSED_V1,
        TELEMETRY_SESSION_OPENED_V1, TELEMETRY_SIGNAL_V1,
    },
    frames::{classify_motion, downsample, MovementSample},
    hash_chain::{compute_this_hash, verify_chain},
    hmac_verify::{compute_hmac, verify_hmac},
    ingest::{ingest_signal_pure, IngestRequest},
    merkle_snapshot::{compute_merkle_root, take_snapshot, TlogClient},
    replay_defence::{InMemoryNonceStore, NonceStore, REPLAY_WINDOW},
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_secret() -> Vec<u8> {
    b"test-hmac-secret-32-bytes-padding".to_vec()
}

fn make_server_key() -> Vec<u8> {
    b"server-signing-key-32-bytes-here".to_vec()
}

fn make_payload() -> serde_json::Value {
    serde_json::json!({"event": "test", "value": 42})
}

fn make_signal(
    device_id: Uuid,
    class: SignalClass,
    counter: i64,
    prev_hash: Vec<u8>,
) -> TelemetrySignal {
    let payload = make_payload();
    let now = Utc::now();
    let mut sig = TelemetrySignal {
        signal_id: Uuid::now_v7(),
        device_id,
        session_id: None,
        class,
        payload,
        occurred_at: now,
        ingested_at: now,
        monotonic_counter: counter,
        prev_hash: if prev_hash.is_empty() {
            None
        } else {
            Some(prev_hash.clone())
        },
        this_hash: vec![],
        hmac_signature: vec![0u8; 32],
        server_signature: None,
        trace_id: None,
        span_id: None,
        correlation_id: None,
        causation_id: None,
    };
    let hash = compute_this_hash(prev_hash.as_slice(), &sig);
    sig.this_hash = hash.to_vec();
    sig
}

fn make_ingest_request(
    device_id: Uuid,
    class: SignalClass,
    counter: i64,
    last_counter: i64,
    last_hash: Vec<u8>,
    secret: &[u8],
) -> IngestRequest {
    let payload = make_payload();
    let hmac_sig = compute_hmac(secret, &payload).unwrap();
    let nonce = vec![counter as u8, 0x01, 0x02, 0x03];

    IngestRequest {
        device_id,
        session_id: None,
        class,
        payload,
        occurred_at: Utc::now(),
        monotonic_counter: counter,
        prev_hash: if last_hash.is_empty() {
            None
        } else {
            Some(last_hash.clone())
        },
        hmac_signature: hmac_sig,
        nonce,
        trace_id: None,
        span_id: None,
        correlation_id: None,
        causation_id: None,
        actor_svid: "spiffe://eel/device/test".to_owned(),
        device_secret: secret.to_vec(),
        last_known_counter: last_counter,
        last_known_hash: last_hash,
    }
}

// ---------------------------------------------------------------------------
// Hash chain tests
// ---------------------------------------------------------------------------

#[test]
fn test_hash_chain_empty_ok() {
    assert!(verify_chain(&[]).is_ok());
}

#[test]
fn test_hash_chain_single_signal_ok() {
    let device_id = Uuid::now_v7();
    let s = make_signal(device_id, SignalClass::Location, 1, vec![]);
    assert!(verify_chain(&[s]).is_ok());
}

#[test]
fn test_hash_chain_multi_signal_ok() {
    let device_id = Uuid::now_v7();
    let s1 = make_signal(device_id, SignalClass::Location, 1, vec![]);
    let s2 = make_signal(device_id, SignalClass::Battery, 2, s1.this_hash.clone());
    let s3 = make_signal(device_id, SignalClass::Drop, 3, s2.this_hash.clone());
    assert!(verify_chain(&[s1, s2, s3]).is_ok());
}

#[test]
fn test_hash_chain_tampered_entry_rejected() {
    let device_id = Uuid::now_v7();
    let s1 = make_signal(device_id, SignalClass::Location, 1, vec![]);
    let s2 = make_signal(device_id, SignalClass::Battery, 2, s1.this_hash.clone());
    let mut s2_tampered = s2.clone();
    s2_tampered.monotonic_counter = 999; // tamper without recomputing hash
    let err = verify_chain(&[s1, s2_tampered]);
    assert!(matches!(err, Err(TelemetryError::ChainBroken { .. })));
}

#[test]
fn test_hash_chain_wrong_prev_hash_rejected() {
    let device_id = Uuid::now_v7();
    let s1 = make_signal(device_id, SignalClass::Location, 1, vec![]);
    // Build s2 with a wrong prev_hash.
    let s2 = make_signal(device_id, SignalClass::Battery, 2, vec![0xFF; 32]);
    // s2.prev_hash != s1.this_hash, so chain verification will fail.
    let err = verify_chain(&[s1, s2]);
    assert!(matches!(err, Err(TelemetryError::ChainBroken { .. })));
}

// ---------------------------------------------------------------------------
// HMAC tests
// ---------------------------------------------------------------------------

#[test]
fn test_hmac_verify_ok() {
    let secret = make_secret();
    let payload = make_payload();
    let sig = compute_hmac(&secret, &payload).unwrap();
    assert!(verify_hmac(&secret, &payload, &sig, "dev-1").is_ok());
}

#[test]
fn test_hmac_verify_wrong_signature_rejected() {
    let secret = make_secret();
    let payload = make_payload();
    let wrong_sig = vec![0xAB; 32];
    let err = verify_hmac(&secret, &payload, &wrong_sig, "dev-1");
    assert!(matches!(err, Err(TelemetryError::HmacFailure { .. })));
}

#[test]
fn test_hmac_verify_wrong_payload_rejected() {
    let secret = make_secret();
    let payload = make_payload();
    let sig = compute_hmac(&secret, &payload).unwrap();
    let wrong_payload = serde_json::json!({"event": "different"});
    let err = verify_hmac(&secret, &wrong_payload, &sig, "dev-1");
    assert!(matches!(err, Err(TelemetryError::HmacFailure { .. })));
}

// ---------------------------------------------------------------------------
// Replay defence tests
// ---------------------------------------------------------------------------

#[test]
fn test_replay_fresh_nonce_accepted() {
    let mut store = InMemoryNonceStore::new();
    let now = Utc::now();
    assert!(store.check_and_record("dev-1", b"nonce-001", now).is_ok());
}

#[test]
fn test_replay_duplicate_nonce_rejected() {
    let mut store = InMemoryNonceStore::new();
    let now = Utc::now();
    store.check_and_record("dev-1", b"nonce-dup", now).unwrap();
    let err = store.check_and_record("dev-1", b"nonce-dup", now);
    assert!(matches!(err, Err(TelemetryError::ReplayDetected { .. })));
}

#[test]
fn test_replay_expired_nonce_accepted() {
    let mut store = InMemoryNonceStore::new();
    let past = Utc::now() - REPLAY_WINDOW - Duration::seconds(10);
    store.check_and_record("dev-1", b"old-nonce", past).unwrap();
    // Same nonce is now beyond the window; should be accepted.
    assert!(store
        .check_and_record("dev-1", b"old-nonce", Utc::now())
        .is_ok());
}

#[test]
fn test_replay_different_device_same_nonce_accepted() {
    let mut store = InMemoryNonceStore::new();
    let now = Utc::now();
    store
        .check_and_record("dev-1", b"shared-nonce", now)
        .unwrap();
    // Same nonce for a different device must not be rejected.
    assert!(store
        .check_and_record("dev-2", b"shared-nonce", now)
        .is_ok());
}

// ---------------------------------------------------------------------------
// Signal ingest pure-logic tests
// ---------------------------------------------------------------------------

#[test]
fn test_ingest_happy_path() {
    let secret = make_secret();
    let server_key = make_server_key();
    let device_id = Uuid::now_v7();
    let req = make_ingest_request(device_id, SignalClass::Location, 1, -1, vec![], &secret);
    let mut store = InMemoryNonceStore::new();
    let outcome = ingest_signal_pure(&req, &mut store, &server_key).unwrap();
    assert_eq!(outcome.signal.monotonic_counter, 1);
    assert!(!outcome.signal.this_hash.is_empty());
    assert!(outcome.signal.server_signature.is_some());
}

#[test]
fn test_ingest_hmac_failure_rejected() {
    let secret = make_secret();
    let server_key = make_server_key();
    let device_id = Uuid::now_v7();
    let mut req = make_ingest_request(device_id, SignalClass::Location, 1, -1, vec![], &secret);
    req.hmac_signature = vec![0xFF; 32]; // corrupt
    let mut store = InMemoryNonceStore::new();
    let err = ingest_signal_pure(&req, &mut store, &server_key);
    assert!(matches!(err, Err(TelemetryError::HmacFailure { .. })));
}

#[test]
fn test_ingest_replay_nonce_rejected() {
    let secret = make_secret();
    let server_key = make_server_key();
    let device_id = Uuid::now_v7();
    let req = make_ingest_request(device_id, SignalClass::Location, 1, -1, vec![], &secret);
    let mut store = InMemoryNonceStore::new();
    ingest_signal_pure(&req, &mut store, &server_key).unwrap();
    // Same nonce again: should be rejected.
    let err = ingest_signal_pure(&req, &mut store, &server_key);
    assert!(matches!(err, Err(TelemetryError::ReplayDetected { .. })));
}

#[test]
fn test_ingest_monotonic_counter_out_of_order_rejected() {
    let secret = make_secret();
    let server_key = make_server_key();
    let device_id = Uuid::now_v7();
    // last_known_counter = 5, but we send counter = 3.
    let req = make_ingest_request(
        device_id,
        SignalClass::Location,
        3,
        5,
        vec![0xAA; 32],
        &secret,
    );
    let mut store = InMemoryNonceStore::new();
    let err = ingest_signal_pure(&req, &mut store, &server_key);
    assert!(matches!(err, Err(TelemetryError::CounterOutOfOrder { .. })));
}

#[test]
fn test_ingest_empty_svid_rejected() {
    let secret = make_secret();
    let server_key = make_server_key();
    let device_id = Uuid::now_v7();
    let mut req = make_ingest_request(device_id, SignalClass::Location, 1, -1, vec![], &secret);
    req.actor_svid = String::new();
    let mut store = InMemoryNonceStore::new();
    let err = ingest_signal_pure(&req, &mut store, &server_key);
    assert!(matches!(err, Err(TelemetryError::SvidRejected(_))));
}

#[test]
fn test_ingest_counter_equal_to_last_rejected() {
    let secret = make_secret();
    let server_key = make_server_key();
    let device_id = Uuid::now_v7();
    // counter == last_known_counter (not strictly greater).
    let req = make_ingest_request(
        device_id,
        SignalClass::Activity,
        5,
        5,
        vec![0xBB; 32],
        &secret,
    );
    let mut store = InMemoryNonceStore::new();
    let err = ingest_signal_pure(&req, &mut store, &server_key);
    assert!(matches!(err, Err(TelemetryError::CounterOutOfOrder { .. })));
}

// ---------------------------------------------------------------------------
// Batch ingest tests
// ---------------------------------------------------------------------------

#[test]
fn test_batch_over_limit_rejected() {
    let requests: Vec<_> = (0..1001)
        .map(|i| {
            let secret = make_secret();
            make_ingest_request(
                Uuid::now_v7(),
                SignalClass::Battery,
                i as i64 + 1,
                -1,
                vec![],
                &secret,
            )
        })
        .collect();
    let mut store = InMemoryNonceStore::new();
    let err = ingest_batch(requests, &mut store, &make_server_key());
    assert!(matches!(
        err,
        Err(TelemetryError::BatchTooLarge { count: 1001 })
    ));
}

#[test]
fn test_batch_partial_success() {
    let secret = make_secret();
    let server_key = make_server_key();
    let device_id = Uuid::now_v7();

    // First request: valid.
    let req_ok = make_ingest_request(device_id, SignalClass::Location, 1, -1, vec![], &secret);

    // Second request: corrupt HMAC -> will fail.
    let mut req_bad = make_ingest_request(
        device_id,
        SignalClass::Battery,
        2,
        1,
        vec![0xCC; 32],
        &secret,
    );
    req_bad.hmac_signature = vec![0xFF; 32];
    req_bad.nonce = vec![0xAB; 4]; // fresh nonce to avoid replay rejection first

    let mut store = InMemoryNonceStore::new();
    let result = ingest_batch(vec![req_ok, req_bad], &mut store, &server_key).unwrap();

    assert_eq!(result.accepted, 1);
    assert_eq!(result.rejected, 1);
}

#[test]
fn test_batch_all_accepted() {
    let secret = make_secret();
    let server_key = make_server_key();

    // 3 different devices, counter 1 each.
    let requests: Vec<_> = (0..3)
        .map(|_| {
            make_ingest_request(
                Uuid::now_v7(),
                SignalClass::Activity,
                1,
                -1,
                vec![],
                &secret,
            )
        })
        .collect();

    let mut store = InMemoryNonceStore::new();
    let result = ingest_batch(requests, &mut store, &server_key).unwrap();
    assert_eq!(result.accepted, 3);
    assert_eq!(result.rejected, 0);
}

// ---------------------------------------------------------------------------
// Activity frame downsampler tests
// ---------------------------------------------------------------------------

/// Fixed base instant for the movement-frame tests.
///
/// `downsample` buckets samples by truncated second, so a test that derives each
/// sample from its own `Utc::now()` straddles a second boundary whenever the
/// loop happens to cross one, and then produces two frames instead of one. The
/// failure is a property of the wall clock, not of the downsampler. Anchoring to
/// a fixed instant makes the bucket assertions deterministic.
fn frame_base() -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339("2026-08-01T10:00:00Z")
        .expect("static base instant")
        .with_timezone(&Utc)
}

fn make_sample_at(
    base: chrono::DateTime<Utc>,
    device_id: Uuid,
    offset_ms: i64,
    accel_x: f32,
    accel_y: f32,
    accel_z: f32,
) -> MovementSample {
    MovementSample {
        device_id,
        sampled_at: base + Duration::milliseconds(offset_ms),
        accel_x,
        accel_y,
        accel_z,
        gyro_x: 0.0,
        gyro_y: 0.0,
        gyro_z: 0.0,
        battery_pct: 80,
        battery_temp_c: 30.0,
        network_class: Some("LTE".to_owned()),
        lat_coarse: None,
        lon_coarse: None,
    }
}

fn make_sample(
    device_id: Uuid,
    offset_ms: i64,
    accel_x: f32,
    accel_y: f32,
    accel_z: f32,
) -> MovementSample {
    make_sample_at(
        frame_base(),
        device_id,
        offset_ms,
        accel_x,
        accel_y,
        accel_z,
    )
}

#[test]
fn test_downsample_single_second_produces_one_frame() {
    let device_id = Uuid::now_v7();
    // 5 samples all within the same second (< 1000ms apart), anchored to a
    // fixed instant so the result does not depend on the wall clock.
    let base = frame_base();
    let samples: Vec<_> = (0..5)
        .map(|i| make_sample_at(base, device_id, i * 100, 0.1, 0.0, 9.81))
        .collect();
    let frames = downsample(samples);
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].device_id, device_id);
}

#[test]
fn test_downsample_samples_straddling_a_second_produce_two_frames() {
    // The complement of the test above, and the behaviour the previous version
    // was accidentally sensitive to: samples either side of a second boundary
    // belong to different buckets.
    let device_id = Uuid::now_v7();
    let base = frame_base();
    let samples: Vec<_> = (0..5)
        .map(|i| make_sample_at(base, device_id, 900 + i * 100, 0.1, 0.0, 9.81))
        .collect();
    let frames = downsample(samples);
    assert_eq!(frames.len(), 2, "900ms..1300ms spans two second buckets");
}

#[test]
fn test_downsample_averages_correctly() {
    let device_id = Uuid::now_v7();
    // 2 samples in the same second with known values.
    let s1 = make_sample(device_id, 0, 1.0, 2.0, 3.0);
    let s2 = make_sample(device_id, 100, 3.0, 4.0, 5.0);
    let frames = downsample(vec![s1, s2]);
    assert_eq!(frames.len(), 1);
    let f = &frames[0];
    assert!(
        (f.accel_x_avg - 2.0).abs() < 0.01,
        "accel_x_avg should be 2.0, got {}",
        f.accel_x_avg
    );
    assert!((f.accel_y_avg - 3.0).abs() < 0.01);
    assert!((f.accel_z_avg - 4.0).abs() < 0.01);
}

#[test]
fn test_motion_class_stationary() {
    let cls = classify_motion(0.01, 0.01, 0.01, 0.02);
    assert_eq!(cls, MotionClass::Stationary);
}

#[test]
fn test_motion_class_walking() {
    let cls = classify_motion(0.3, 0.3, 0.0, 0.5);
    assert_eq!(cls, MotionClass::Walking);
}

#[test]
fn test_motion_class_running_via_max_g() {
    let cls = classify_motion(0.5, 0.5, 0.0, 3.5);
    assert_eq!(cls, MotionClass::Running);
}

// ---------------------------------------------------------------------------
// Alert rule engine tests
// ---------------------------------------------------------------------------

fn make_signal_for_alert(
    device_id: Uuid,
    class: SignalClass,
    payload: serde_json::Value,
) -> TelemetrySignal {
    let now = Utc::now();
    TelemetrySignal {
        signal_id: Uuid::now_v7(),
        device_id,
        session_id: None,
        class,
        payload,
        occurred_at: now,
        ingested_at: now,
        monotonic_counter: 1,
        prev_hash: None,
        this_hash: vec![0u8; 32],
        hmac_signature: vec![0u8; 32],
        server_signature: None,
        trace_id: None,
        span_id: None,
        correlation_id: None,
        causation_id: None,
    }
}

#[test]
fn test_alert_drop_raised() {
    let device_id = Uuid::now_v7();
    let sig = make_signal_for_alert(device_id, SignalClass::Drop, serde_json::json!({}));
    let alerts = evaluate(&sig);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].class, AlertClass::Drop);
    assert_eq!(alerts[0].severity, Severity::Critical);
}

#[test]
fn test_alert_battery_low_raised() {
    let device_id = Uuid::now_v7();
    let sig = make_signal_for_alert(
        device_id,
        SignalClass::Battery,
        serde_json::json!({"battery_pct": 5}),
    );
    let alerts = evaluate(&sig);
    assert!(alerts.iter().any(|a| a.class == AlertClass::BatteryLow));
}

#[test]
fn test_alert_battery_ok_no_alert() {
    let device_id = Uuid::now_v7();
    let sig = make_signal_for_alert(
        device_id,
        SignalClass::Battery,
        serde_json::json!({"battery_pct": 80}),
    );
    let alerts = evaluate(&sig);
    assert!(alerts.is_empty());
}

#[test]
fn test_alert_geo_fence_breach_raised() {
    let device_id = Uuid::now_v7();
    let sig = make_signal_for_alert(
        device_id,
        SignalClass::Location,
        serde_json::json!({"out_of_bounds": true, "lat": -1.28, "lon": 36.82}),
    );
    let alerts = evaluate(&sig);
    assert!(alerts.iter().any(|a| a.class == AlertClass::GeoFenceBreach));
}

#[test]
fn test_alert_login_time_anomaly_midnight() {
    let device_id = Uuid::now_v7();
    let now = Utc::now();
    // Force occurred_at to midnight UTC (hour 0).
    let midnight = now.date_naive().and_hms_opt(0, 30, 0).unwrap().and_utc();
    let mut sig = make_signal_for_alert(device_id, SignalClass::LoginLogout, serde_json::json!({}));
    sig.occurred_at = midnight;
    let alerts = evaluate(&sig);
    assert!(alerts
        .iter()
        .any(|a| a.class == AlertClass::LoginTimeAnomaly));
}

#[test]
fn test_alert_read_heavy_raised() {
    let device_id = Uuid::now_v7();
    let sig = make_signal_for_alert(
        device_id,
        SignalClass::Read,
        serde_json::json!({"read_count": 501}),
    );
    let alerts = evaluate(&sig);
    assert!(alerts.iter().any(|a| a.class == AlertClass::ReadHeavy));
}

// ---------------------------------------------------------------------------
// Merkle snapshot tests
// ---------------------------------------------------------------------------

#[test]
fn test_merkle_root_single_leaf() {
    let root = compute_merkle_root(&[b"hello".to_vec()]).unwrap();
    assert_eq!(root.len(), 32);
}

#[test]
fn test_merkle_root_two_leaves_deterministic() {
    let a = compute_merkle_root(&[b"leaf1".to_vec(), b"leaf2".to_vec()]).unwrap();
    let b = compute_merkle_root(&[b"leaf1".to_vec(), b"leaf2".to_vec()]).unwrap();
    assert_eq!(a, b);
}

#[test]
fn test_merkle_root_different_leaves_differ() {
    let a = compute_merkle_root(&[b"leaf1".to_vec()]).unwrap();
    let b = compute_merkle_root(&[b"leaf2".to_vec()]).unwrap();
    assert_ne!(a, b);
}

#[test]
fn test_merkle_root_empty_is_error() {
    let err = compute_merkle_root(&[]);
    assert!(err.is_err());
}

#[tokio::test]
async fn test_merkle_snapshot_round_trip_stub() {
    let device_id = Uuid::now_v7();
    let date = chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
    let leaves = vec![b"hash1".to_vec(), b"hash2".to_vec(), b"hash3".to_vec()];
    let tlog = TlogClient::new_stub();
    let snap = take_snapshot(device_id, date, leaves, &tlog).await.unwrap();
    assert_eq!(snap.snapshot_date, date);
    assert_eq!(snap.device_id, Some(device_id));
    assert!(snap.device_merkle_root.is_some());
    assert_eq!(snap.tlog_leaf_index, Some(42));
    assert!(snap.inclusion_proof.is_some());
}

// ---------------------------------------------------------------------------
// Geo-dwell computation tests
// ---------------------------------------------------------------------------

fn make_ping(device_id: Uuid, polygon_ref: &str, offset_secs: i64) -> LocationPing {
    LocationPing {
        device_id,
        occurred_at: frame_base() + Duration::seconds(offset_secs),
        polygon_ref: polygon_ref.to_owned(),
    }
}

#[test]
fn test_geo_dwell_single_polygon() {
    let device_id = Uuid::now_v7();
    let pings = vec![
        make_ping(device_id, "zone-A", 0),
        make_ping(device_id, "zone-A", 60),
        make_ping(device_id, "zone-A", 120),
    ];
    let dwells = compute_dwells(&pings);
    assert_eq!(dwells.len(), 1);
    assert_eq!(dwells[0].polygon_ref, "zone-A");
    assert!(dwells[0].exited_at.is_none()); // still inside
}

#[test]
fn test_geo_dwell_transition_creates_two_dwells() {
    let device_id = Uuid::now_v7();
    let pings = vec![
        make_ping(device_id, "zone-A", 0),
        make_ping(device_id, "zone-A", 30),
        make_ping(device_id, "zone-B", 60),
        make_ping(device_id, "zone-B", 90),
    ];
    let dwells = compute_dwells(&pings);
    assert_eq!(dwells.len(), 2);
    assert_eq!(dwells[0].polygon_ref, "zone-A");
    assert!(dwells[0].exited_at.is_some());
    assert_eq!(dwells[1].polygon_ref, "zone-B");
    assert!(dwells[1].exited_at.is_none());
}

#[test]
fn test_geo_dwell_empty_input() {
    let dwells = compute_dwells(&[]);
    assert!(dwells.is_empty());
}

// ---------------------------------------------------------------------------
// CloudEvent builder tests
// ---------------------------------------------------------------------------

#[test]
fn test_event_signal_type() {
    let data = telemetry_signal_data(Uuid::now_v7(), Uuid::now_v7(), "location");
    assert_eq!(data["type"], TELEMETRY_SIGNAL_V1);
}

#[test]
fn test_event_session_opened_type() {
    let data = telemetry_session_opened_data(Uuid::now_v7(), Uuid::now_v7(), "spiffe://eel/device");
    assert_eq!(data["type"], TELEMETRY_SESSION_OPENED_V1);
}

#[test]
fn test_event_session_closed_type() {
    let data = telemetry_session_closed_data(Uuid::now_v7(), Uuid::now_v7(), "normal");
    assert_eq!(data["type"], TELEMETRY_SESSION_CLOSED_V1);
}

#[test]
fn test_event_alert_type() {
    let data = telemetry_alert_data(Uuid::now_v7(), Uuid::now_v7(), "drop", "critical");
    assert_eq!(data["type"], TELEMETRY_ALERT_V1);
}

// ---------------------------------------------------------------------------
// Session lifecycle tests
// ---------------------------------------------------------------------------

#[test]
fn test_exit_reason_roundtrip() {
    for (s, expected) in [
        ("normal", ExitReason::Normal),
        ("timeout", ExitReason::Timeout),
        ("revoked", ExitReason::Revoked),
        ("duress", ExitReason::Duress),
        ("crash", ExitReason::Crash),
    ] {
        let parsed = ExitReason::from_str(s);
        assert_eq!(parsed, Some(expected.clone()), "failed for '{}'", s);
        assert_eq!(expected.as_str(), s);
    }
}

#[test]
fn test_session_duress_exit_reason() {
    let reason = ExitReason::from_str("duress");
    assert_eq!(reason, Some(ExitReason::Duress));
}

#[test]
fn test_session_unknown_exit_reason_is_none() {
    assert!(ExitReason::from_str("panic").is_none());
}

// ---------------------------------------------------------------------------
// Signal class roundtrip tests
// ---------------------------------------------------------------------------

#[test]
fn test_signal_class_roundtrip() {
    let classes = [
        "location",
        "time",
        "activity",
        "proximal_rfid",
        "movement",
        "login_logout",
        "read",
        "write",
        "battery",
        "lock_unlock",
        "drop",
        "network",
        "attestation",
    ];
    for class_str in classes {
        let cls = SignalClass::from_str(class_str)
            .unwrap_or_else(|| panic!("unknown class: {}", class_str));
        assert_eq!(cls.as_str(), class_str);
    }
}
