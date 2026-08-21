//! Tests for twin registry.

use eel_abis_twin_registry::detector::{detect_twin_candidates, face_iris_correlation_check};
use eel_abis_twin_registry::domain::TwinVerificationStatus;
use eel_abis_twin_registry::error::TwinRegistryError;
use eel_abis_twin_registry::registry::{add_member, create_twin_group, verify_from_civil_registry};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn create_group_with_two_members() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let group = create_twin_group(&[a, b]).unwrap();
    assert_eq!(group.members.len(), 2);
    assert_eq!(
        group.verification_status,
        TwinVerificationStatus::Unverified
    );
}

#[test]
fn create_group_with_one_member_fails() {
    let a = Uuid::now_v7();
    assert!(matches!(
        create_twin_group(&[a]),
        Err(TwinRegistryError::InsufficientMembers)
    ));
}

#[test]
fn add_member_increases_count() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let c = Uuid::now_v7();
    let mut group = create_twin_group(&[a, b]).unwrap();
    add_member(&mut group, c).unwrap();
    assert_eq!(group.members.len(), 3);
}

#[test]
fn add_duplicate_member_fails() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let mut group = create_twin_group(&[a, b]).unwrap();
    assert!(matches!(
        add_member(&mut group, a),
        Err(TwinRegistryError::AlreadyInGroup(_))
    ));
}

#[test]
fn civil_registry_verification_sets_status() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let mut group = create_twin_group(&[a, b]).unwrap();
    verify_from_civil_registry(
        &mut group,
        &[(a, "CR001".to_string()), (b, "CR002".to_string())],
    )
    .unwrap();
    assert_eq!(
        group.verification_status,
        TwinVerificationStatus::VerifiedByCivilRegistry
    );
    assert!(group.verified_at.is_some());
}

#[test]
fn civil_registry_partial_sets_pending() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let mut group = create_twin_group(&[a, b]).unwrap();
    verify_from_civil_registry(&mut group, &[(a, "CR001".to_string())]).unwrap();
    assert_eq!(
        group.verification_status,
        TwinVerificationStatus::PendingCivilRegistry
    );
}

#[test]
fn detect_twin_candidates_in_ambiguity_band() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let pairs = vec![(a, b, 0.72)];
    let mut demo = HashMap::new();
    demo.insert((a, b), true);
    let geo = HashMap::new();
    let candidates = detect_twin_candidates(&pairs, &demo, &geo);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].subject_a, a);
}

#[test]
fn detect_excludes_high_score_pairs() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let pairs = vec![(a, b, 0.99)];
    let demo = HashMap::new();
    let geo = HashMap::new();
    let candidates = detect_twin_candidates(&pairs, &demo, &geo);
    assert!(candidates.is_empty());
}

#[test]
fn detect_excludes_low_score_pairs() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let pairs = vec![(a, b, 0.30)];
    let demo = HashMap::new();
    let geo = HashMap::new();
    let candidates = detect_twin_candidates(&pairs, &demo, &geo);
    assert!(candidates.is_empty());
}

#[test]
fn face_iris_correlation_genuine_twin() {
    let result = face_iris_correlation_check(0.85, 0.70, 0.75).unwrap();
    assert!(result);
}

#[test]
fn face_iris_correlation_impostor() {
    let result = face_iris_correlation_check(0.30, 0.28, 0.75).unwrap();
    assert!(!result);
}

#[test]
fn face_iris_correlation_rejects_invalid_scores() {
    assert!(face_iris_correlation_check(1.5, 0.5, 0.75).is_err());
    assert!(face_iris_correlation_check(0.5, -0.1, 0.75).is_err());
}

#[test]
fn civil_registry_unknown_subject_fails() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let mut group = create_twin_group(&[a, b]).unwrap();
    let unknown = Uuid::now_v7();
    assert!(verify_from_civil_registry(&mut group, &[(unknown, "X".to_string())]).is_err());
}
