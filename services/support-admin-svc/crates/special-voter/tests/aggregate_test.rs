//! Integration tests for special-voter domain aggregates (WP-5.5, M25).

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, StoredEvent};
use serde_json::json;
use special_voter::{
    aggregates::facility::SpecialVoterFacility,
    aggregates::idp::IDPReassignment,
    aggregates::voter_event::SpecialVoterEvent,
    events::*,
    guards::{validate_iab_sample_fraction, validate_s44a_dual_signoff, S44AOverrideRequest},
    workflows::{guard_idp_double_cast, validate_kit_open, SpecialVoterCategory, WorkflowError},
};
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

// --- SpecialVoterFacility ---

#[test]
fn facility_opens_on_event() {
    let id = Uuid::new_v4();
    let mut f = SpecialVoterFacility::new(id);
    let ev = stored(
        id,
        1,
        EVENT_SPECIAL_FACILITY_OPENED,
        json!({
            "facility_id": id.to_string(),
            "category": "REMAND_PRISON",
            "county_code": "047",
            "kit_ids": ["KIT-001"],
            "presiding_officer_id": "OFF-001",
            "voting_date": "2027-08-09"
        }),
    );
    f.apply(&ev).unwrap();
    assert_eq!(f.county_code, "047");
    assert_eq!(f.presiding_officer_id, "OFF-001");
    assert_eq!(f.version, 1);
}

#[test]
fn facility_closes_on_form34_transmitted() {
    use special_voter::workflows::FacilitySessionState;
    let id = Uuid::new_v4();
    let mut f = SpecialVoterFacility::new(id);
    f.apply(&stored(
        id,
        1,
        EVENT_SPECIAL_FACILITY_OPENED,
        json!({
            "category": "HOSPITAL", "county_code": "001", "kit_ids": [],
            "presiding_officer_id": "OFF-002", "voting_date": "2027-08-09"
        }),
    ))
    .unwrap();
    f.apply(&stored(
        id,
        2,
        EVENT_SPECIAL_FORM34_TRANSMITTED,
        json!({
            "form_id": Uuid::new_v4().to_string(),
            "facility_id": id.to_string(),
            "form_variant": "34A-H",
            "transmitted_at": "2027-08-09T18:30:00Z",
            "merkle_node_id": "abc123"
        }),
    ))
    .unwrap();
    assert_eq!(f.session_state, FacilitySessionState::Closed);
}

#[test]
fn facility_snapshot_round_trip() {
    let id = Uuid::new_v4();
    let mut f = SpecialVoterFacility::new(id);
    f.apply(&stored(
        id,
        1,
        EVENT_SPECIAL_FACILITY_OPENED,
        json!({
            "category": "BARRACKS", "county_code": "010", "kit_ids": ["K1", "K2"],
            "presiding_officer_id": "OFF-003", "voting_date": "2027-08-09"
        }),
    ))
    .unwrap();
    let snap = f.snapshot();
    let restored: SpecialVoterFacility = serde_json::from_value(snap.state).unwrap();
    assert_eq!(restored.county_code, "010");
    assert_eq!(restored.kit_ids, vec!["K1", "K2"]);
}

// --- IDPReassignment ---

#[test]
fn idp_reassignment_apply() {
    let id = Uuid::new_v4();
    let facility_id = Uuid::new_v4();
    let mut r = IDPReassignment::new(id);
    r.apply(&stored(
        id,
        1,
        EVENT_IDP_REASSIGNMENT_RECORDED,
        json!({
            "voter_id_enc": "enc:abc",
            "original_station_code": "010-0001-001",
            "idp_facility_id": facility_id.to_string(),
            "displacement_source": "NDOC",
            "effective_date": "2027-07-25",
            "reassigned_at": "2027-07-26T09:00:00Z"
        }),
    ))
    .unwrap();
    assert_eq!(r.original_station_code, "010-0001-001");
    assert_eq!(r.idp_facility_id, facility_id);
    assert_eq!(r.displacement_source, "NDOC");
}

// --- SpecialVoterEvent ---

#[test]
fn voter_event_apply() {
    let id = Uuid::new_v4();
    let facility_id = Uuid::new_v4();
    let mut ev = SpecialVoterEvent::new(id);
    ev.apply(&stored(
        id,
        1,
        EVENT_SPECIAL_VOTER_EVENT,
        json!({
            "event_id": id.to_string(),
            "facility_id": facility_id.to_string(),
            "category": "HOSPITAL",
            "event_type": "VOTER_AUTHENTICATED",
            "officer_id": "OFF-010",
            "witness_ids": ["W1", "W2"],
            "reason_code": "ASSISTED_VOTING"
        }),
    ))
    .unwrap();
    assert_eq!(ev.category, "HOSPITAL");
    assert_eq!(ev.witness_ids, vec!["W1", "W2"]);
    assert_eq!(ev.reason_code.as_deref(), Some("ASSISTED_VOTING"));
}

// --- Workflow guards ---

#[test]
fn dual_biometric_required_for_remand() {
    let err = validate_kit_open(&SpecialVoterCategory::RemandPrison, 1, false).unwrap_err();
    assert!(matches!(err, WorkflowError::DualBiometricRequired));
}

#[test]
fn dual_biometric_passes_with_two_officers() {
    validate_kit_open(&SpecialVoterCategory::RemandPrison, 2, false).unwrap();
}

#[test]
fn refugee_voting_disabled_by_default() {
    let err = validate_kit_open(&SpecialVoterCategory::RefugeeCamp, 1, false).unwrap_err();
    assert!(matches!(err, WorkflowError::RefugeeVotingDisabled));
}

#[test]
fn refugee_voting_enabled_by_flag() {
    validate_kit_open(&SpecialVoterCategory::RefugeeCamp, 1, true).unwrap();
}

#[test]
fn idp_double_cast_rejected() {
    let err = guard_idp_double_cast("VOTER-001", true).unwrap_err();
    assert!(matches!(err, WorkflowError::DoubleVoteGuard { .. }));
}

// --- s.44A guardrails ---

#[test]
fn s44a_same_officer_rejected() {
    let req = S44AOverrideRequest {
        facility_id: Uuid::new_v4(),
        reason_code: "MEDICAL_INABILITY".to_string(),
        requesting_officer_id: "OFF-A".to_string(),
        countersigning_officer_id: "OFF-A".to_string(),
    };
    let err = validate_s44a_dual_signoff(&req).unwrap_err();
    assert!(err.to_string().contains("different"));
}

#[test]
fn s44a_distinct_officers_pass() {
    let req = S44AOverrideRequest {
        facility_id: Uuid::new_v4(),
        reason_code: "MEDICAL_INABILITY".to_string(),
        requesting_officer_id: "OFF-A".to_string(),
        countersigning_officer_id: "OFF-B".to_string(),
    };
    validate_s44a_dual_signoff(&req).unwrap();
}

#[test]
fn iab_sample_fraction_floor() {
    // 4 of 100 = 4% — below floor
    let err = validate_iab_sample_fraction(4, 100).unwrap_err();
    assert!(err.to_string().contains("0.04"));
    // 5 of 100 = 5% — at floor
    validate_iab_sample_fraction(5, 100).unwrap();
}
