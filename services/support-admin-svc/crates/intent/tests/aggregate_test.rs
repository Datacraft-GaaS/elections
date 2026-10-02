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

//! Integration tests for intent domain aggregates (WP-5.5, M27).

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, StoredEvent};
use intent::{
    aggregates::adjudication::AdjudicationEvent,
    aggregates::alert::RejectionRateAlert,
    aggregates::premark::PreMarkValidationEvent,
    aggregates::spoilt::SpoiltBallotEvent,
    catalogue::{lookup, validate_decision, AdjudicationDecision},
    events::*,
    model::{assert_image_deleted, assert_model_version, PreMarkModelDescriptor},
};
use serde_json::json;
use uuid::Uuid;

fn stored(
    stream_id: Uuid,
    version: i64,
    event_type: &str,
    payload: serde_json::Value,
) -> StoredEvent {
    StoredEvent {
        stream_id,
        stream_version: version,
        id: Uuid::new_v4(),
        event_type: event_type.to_string(),
        payload,
        prev_hash: None,
        hlc: "0".to_string(),
        recorded_at: Utc::now(),
    }
}

// --- PreMarkValidationEvent ---

#[test]
fn premark_apply_event() {
    let id = Uuid::new_v4();
    let mut e = PreMarkValidationEvent::new(id);
    e.apply(&stored(
        id,
        1,
        EVENT_PREMARK_VALIDATION,
        json!({
            "station_code": "047-0001-001",
            "contest_id": "NAT-PRES-2027",
            "inference_outcome": "LIKELY_VALID",
            "model_version": "v2.1.0",
            "device_id": "KIT-047-0001",
            "image_delete_elapsed_ms": 320
        }),
    ))
    .unwrap();
    assert_eq!(e.station_code, "047-0001-001");
    assert_eq!(e.inference_outcome, "LIKELY_VALID");
    assert_eq!(e.image_delete_elapsed_ms, 320);
}

#[test]
fn premark_snapshot_round_trip() {
    let id = Uuid::new_v4();
    let mut e = PreMarkValidationEvent::new(id);
    e.apply(&stored(
        id,
        1,
        EVENT_PREMARK_VALIDATION,
        json!({
            "station_code": "010-0002-001",
            "contest_id": "CTY-GOV-2027",
            "inference_outcome": "SPOILT",
            "model_version": "v2.1.0",
            "device_id": "KIT-010-0002",
            "image_delete_elapsed_ms": 750
        }),
    ))
    .unwrap();
    let snap = e.snapshot();
    let restored: PreMarkValidationEvent = serde_json::from_value(snap.state).unwrap();
    assert_eq!(restored.station_code, "010-0002-001");
}

// --- SpoiltBallotEvent ---

#[test]
fn spoilt_ballot_apply() {
    let id = Uuid::new_v4();
    let mut s = SpoiltBallotEvent::new(id);
    s.apply(&stored(
        id,
        1,
        EVENT_SPOILT_BALLOT,
        json!({
            "station_code": "047-0003-001",
            "contest_id": "NAT-PRES-2027",
            "reason_code": "TORN",
            "officer_id": "OFF-005",
            "witness_ids": ["W1"],
            "replacement_issued": true
        }),
    ))
    .unwrap();
    assert!(s.replacement_issued);
    assert_eq!(s.reason_code, "TORN");
}

// --- AdjudicationEvent ---

#[test]
fn adjudication_apply_reject() {
    let id = Uuid::new_v4();
    let mut a = AdjudicationEvent::new(id);
    a.apply(&stored(
        id,
        1,
        EVENT_ADJUDICATION,
        json!({
            "ballot_id": "BALLOT-001",
            "station_code": "047-0001-001",
            "contest_id": "NAT-PRES-2027",
            "adjudicator_id": "ADJ-001",
            "reason_code": "A2",
            "decision": "REJECT",
            "photo_evidence_ref": "minio://evidence/BALLOT-001.jpg"
        }),
    ))
    .unwrap();
    assert_eq!(a.reason_code, "A2");
    assert!(matches!(a.decision, Some(AdjudicationDecision::Reject)));
    assert!(a.photo_evidence_ref.is_some());
}

#[test]
fn adjudication_apply_accept() {
    let id = Uuid::new_v4();
    let mut a = AdjudicationEvent::new(id);
    a.apply(&stored(
        id,
        1,
        EVENT_ADJUDICATION,
        json!({
            "ballot_id": "BALLOT-002",
            "station_code": "047-0001-001",
            "contest_id": "NAT-PRES-2027",
            "adjudicator_id": "ADJ-002",
            "reason_code": "V1",
            "decision": "ACCEPT"
        }),
    ))
    .unwrap();
    assert!(matches!(a.decision, Some(AdjudicationDecision::Accept)));
}

// --- RejectionRateAlert ---

#[test]
fn alert_apply_event() {
    let id = Uuid::new_v4();
    let mut al = RejectionRateAlert::new(id);
    al.apply(&stored(
        id,
        1,
        EVENT_REJECTION_RATE_ALERT,
        json!({
            "station_code": "020-0001-001",
            "contest_id": "NAT-PRES-2027",
            "rejection_rate": 0.12,
            "national_mean": 0.008,
            "z_score": 4.2,
            "alerted_at": "2027-08-09T12:00:00Z"
        }),
    ))
    .unwrap();
    assert!(al.z_score > 3.0);
    assert!(RejectionRateAlert::is_outlier(al.z_score));
}

#[test]
fn z_score_computation() {
    let z = RejectionRateAlert::compute_z_score(0.12, 0.008, 0.03);
    assert!(z > 3.0, "z = {z}");
}

#[test]
fn z_score_zero_stddev() {
    let z = RejectionRateAlert::compute_z_score(0.05, 0.01, 0.0);
    assert_eq!(z, 0.0);
}

// --- Catalogue ---

#[test]
fn catalogue_lookup_all_codes() {
    for code in &[
        "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "V1", "V2", "V3",
    ] {
        lookup(code).unwrap_or_else(|_| panic!("missing code {code}"));
    }
}

#[test]
fn catalogue_lookup_unknown_fails() {
    assert!(lookup("Z9").is_err());
}

#[test]
fn catalogue_decision_direction() {
    validate_decision("A1", &AdjudicationDecision::Reject).unwrap();
    validate_decision("V2", &AdjudicationDecision::Accept).unwrap();
}

// --- Model registry ---

#[test]
fn image_delete_within_deadline() {
    assert_image_deleted(999).unwrap();
    assert_image_deleted(1000).unwrap();
}

#[test]
fn image_delete_past_deadline_fails() {
    let err = assert_image_deleted(1001).unwrap_err();
    assert!(err.to_string().contains("1001"));
}

#[test]
fn model_version_match() {
    let desc = PreMarkModelDescriptor {
        model_id: "premark-v2".to_string(),
        version: "v2.1.0".to_string(),
        artefact_sha256: "abc".to_string(),
        registered_at: "2027-01-01T00:00:00Z".to_string(),
        registry_entry_id: "reg-001".to_string(),
    };
    assert_model_version(&desc, "v2.1.0").unwrap();
    assert!(assert_model_version(&desc, "v2.0.0").is_err());
}
