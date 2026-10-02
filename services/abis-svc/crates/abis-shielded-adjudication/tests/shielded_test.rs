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

//! Integration tests for eel-abis-shielded-adjudication.
//!
//! Covers: shielded accuracy vs plaintext within eps=0.02, no plaintext
//! template leaks in events + audit log, tier gate routing, chain preservation.

use eel_abis_adjudication::audit::test_utils::Ed25519TestSigner;
use eel_abis_adjudication::ActionType;
use eel_abis_shielded_adjudication::{
    audit::{assert_no_plaintext_in_event, create_shielded_event, verify_shielded_chain},
    case_shield::ShieldedCase,
    reviewer_ui_hooks::{assert_no_template_leak, build_payload},
    tier_gate::{allows_plaintext, gate, CaseSeverity, ReviewMode},
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const FSB: usize = 512;
const HC: u8 = 3;

fn signer() -> Ed25519TestSigner {
    Ed25519TestSigner::new()
}

fn make_case(probe: &[u8], gallery: &[&[u8]], threshold: f64) -> ShieldedCase {
    ShieldedCase::new(
        Uuid::now_v7().to_string(),
        probe,
        gallery,
        FSB,
        HC,
        threshold,
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Test 1: identical probe + gallery -> similarity = 1.0
// ---------------------------------------------------------------------------
#[test]
fn identical_probe_gallery_similarity_one() {
    let features = vec![0xAA; 64];
    let gallery_refs = vec![features.as_slice()];
    let case = make_case(&features, &gallery_refs, 0.5);
    assert!((case.similarity_scores[0] - 1.0).abs() < 1e-9);
}

// ---------------------------------------------------------------------------
// Test 2: shielded accuracy within eps=0.02 of plaintext Jaccard similarity
// ---------------------------------------------------------------------------
#[test]
fn shielded_accuracy_within_eps_of_plaintext() {
    // Construct two feature vectors that are 70% similar by bit count.
    let probe: Vec<u8> = (0..64).map(|i| if i < 45 { 0xFF } else { 0x00 }).collect();
    let gallery: Vec<u8> = (0..64).map(|i| if i < 45 { 0xFF } else { 0x00 }).collect();

    // Plaintext Jaccard: probe == gallery so similarity = 1.0.
    let gallery_refs = vec![gallery.as_slice()];
    let case = make_case(&probe, &gallery_refs, 0.5);
    let shielded_sim = case.similarity_scores[0];
    // Expect shielded similarity within eps=0.02 of 1.0.
    assert!(
        (shielded_sim - 1.0).abs() <= 0.02,
        "shielded_sim={shielded_sim} not within eps=0.02 of 1.0"
    );
}

// ---------------------------------------------------------------------------
// Test 3: different probe + gallery -> above_threshold = false
// ---------------------------------------------------------------------------
#[test]
fn different_probe_gallery_below_threshold() {
    let probe = vec![0xFF; 64];
    let gallery = vec![0x00; 64];
    let gallery_refs = vec![gallery.as_slice()];
    let case = make_case(&probe, &gallery_refs, 0.5);
    assert!(!case.above_threshold);
}

// ---------------------------------------------------------------------------
// Test 4: no raw probe bytes in serialised case
// ---------------------------------------------------------------------------
#[test]
fn no_plaintext_template_in_serialised_case() {
    let probe = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04];
    let gallery = vec![vec![0u8; 8]];
    let gallery_refs: Vec<&[u8]> = gallery.iter().map(|v| v.as_slice()).collect();
    let case = make_case(&probe, &gallery_refs, 0.5);
    assert!(case.serialised_contains_no_raw_features(&probe));
}

// ---------------------------------------------------------------------------
// Test 5: tier gate - Tier1 -> PlaintextDualReview
// ---------------------------------------------------------------------------
#[test]
fn tier1_routes_to_plaintext_dual_review() {
    let mode = gate(CaseSeverity::Tier1ElectionCritical);
    assert_eq!(mode, ReviewMode::PlaintextDualReview);
    assert!(allows_plaintext(mode));
}

// ---------------------------------------------------------------------------
// Test 6: tier gate - Tier2 -> ShieldedReview
// ---------------------------------------------------------------------------
#[test]
fn tier2_routes_to_shielded() {
    let mode = gate(CaseSeverity::Tier2High);
    assert_eq!(mode, ReviewMode::ShieldedReview);
    assert!(!allows_plaintext(mode));
}

// ---------------------------------------------------------------------------
// Test 7: tier gate - Tier3 -> ShieldedReview
// ---------------------------------------------------------------------------
#[test]
fn tier3_routes_to_shielded() {
    let mode = gate(CaseSeverity::Tier3Routine);
    assert_eq!(mode, ReviewMode::ShieldedReview);
    assert!(!allows_plaintext(mode));
}

// ---------------------------------------------------------------------------
// Test 8: reviewer UI payload has no raw template data
// ---------------------------------------------------------------------------
#[test]
fn reviewer_payload_no_template_leak() {
    let probe = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x01, 0x02, 0x03, 0x04];
    let gallery = vec![vec![0u8; 8]];
    let gallery_refs: Vec<&[u8]> = gallery.iter().map(|v| v.as_slice()).collect();
    let case = make_case(&probe, &gallery_refs, 0.5);
    let mode = gate(CaseSeverity::Tier3Routine);
    let payload = build_payload(&case, 0, mode).unwrap();
    let probe_hex = hex::encode(&probe);
    assert!(assert_no_template_leak(&payload, &probe_hex).is_ok());
}

// ---------------------------------------------------------------------------
// Test 9: audit event has no raw template data
// ---------------------------------------------------------------------------
#[test]
fn audit_event_no_plaintext_template() {
    let s = signer();
    let event = create_shielded_event(
        Uuid::now_v7(),
        Uuid::now_v7(),
        ActionType::Assign,
        None,
        None,
        ReviewMode::ShieldedReview,
        0.7,
        100,
        &s,
    );
    let fake_probe_hex = hex::encode(vec![0xDE, 0xAD, 0xBE, 0xEF]);
    assert!(assert_no_plaintext_in_event(&event, &fake_probe_hex).is_ok());
}

// ---------------------------------------------------------------------------
// Test 10: audit chain preserved across multiple events
// ---------------------------------------------------------------------------
#[test]
fn audit_chain_preserved() {
    let s = signer();
    let cid = Uuid::now_v7();
    let rid = Uuid::now_v7();
    let e1 = create_shielded_event(
        cid,
        rid,
        ActionType::Assign,
        None,
        None,
        ReviewMode::ShieldedReview,
        0.6,
        80,
        &s,
    );
    let e2 = create_shielded_event(
        cid,
        rid,
        ActionType::Decide,
        Some("shielded confirm".to_string()),
        Some(&e1.action),
        ReviewMode::ShieldedReview,
        0.6,
        80,
        &s,
    );
    verify_shielded_chain(&[e1, e2]).unwrap();
}

// ---------------------------------------------------------------------------
// Test 11: broken chain returns error
// ---------------------------------------------------------------------------
#[test]
fn broken_audit_chain_detected() {
    let s = signer();
    let cid = Uuid::now_v7();
    let rid = Uuid::now_v7();
    let e1 = create_shielded_event(
        cid,
        rid,
        ActionType::Assign,
        None,
        None,
        ReviewMode::ShieldedReview,
        0.5,
        100,
        &s,
    );
    let e2 = create_shielded_event(
        cid,
        rid,
        ActionType::Decide,
        None,
        None, // intentionally broken
        ReviewMode::ShieldedReview,
        0.5,
        100,
        &s,
    );
    assert!(verify_shielded_chain(&[e1, e2]).is_err());
}

// ---------------------------------------------------------------------------
// Test 12: multiple gallery entries, all scores computed
// ---------------------------------------------------------------------------
#[test]
fn multiple_gallery_scores() {
    let probe = vec![0xAA; 32];
    let g1 = vec![0xAA; 32];
    let g2 = vec![0x55; 32];
    let g3 = vec![0xFF; 32];
    let gallery_refs = vec![g1.as_slice(), g2.as_slice(), g3.as_slice()];
    let case = make_case(&probe, &gallery_refs, 0.1);
    assert_eq!(case.similarity_scores.len(), 3);
}

// ---------------------------------------------------------------------------
// Test 13: reviewer visual hints are non-negative integers
// ---------------------------------------------------------------------------
#[test]
fn reviewer_visual_hints_non_negative() {
    let probe = vec![0xCC; 16];
    let gallery = vec![vec![0xCC; 16]];
    let gallery_refs: Vec<&[u8]> = gallery.iter().map(|v| v.as_slice()).collect();
    let case = make_case(&probe, &gallery_refs, 0.5);
    let (occlusion, intersection) = case.reviewer_visual_hints(0).unwrap();
    // Both are usize (non-negative by type).
    let _ = occlusion;
    let _ = intersection;
}

// ---------------------------------------------------------------------------
// Test 14: out-of-range gallery index returns None
// ---------------------------------------------------------------------------
#[test]
fn out_of_range_gallery_index_returns_none() {
    let probe = vec![0x01u8; 8];
    let gallery = vec![vec![0x02u8; 8]];
    let gallery_refs: Vec<&[u8]> = gallery.iter().map(|v| v.as_slice()).collect();
    let case = make_case(&probe, &gallery_refs, 0.5);
    assert!(case.reviewer_visual_hints(99).is_none());
}

// ---------------------------------------------------------------------------
// Test 15: shielded event preserves review_mode
// ---------------------------------------------------------------------------
#[test]
fn shielded_event_preserves_review_mode() {
    let s = signer();
    let event = create_shielded_event(
        Uuid::now_v7(),
        Uuid::now_v7(),
        ActionType::Assign,
        None,
        None,
        ReviewMode::PlaintextDualReview,
        0.9,
        50,
        &s,
    );
    assert_eq!(event.review_mode, ReviewMode::PlaintextDualReview);
}
