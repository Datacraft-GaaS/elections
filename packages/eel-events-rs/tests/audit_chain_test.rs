//! Tests for the cross-service audit-shadow prev_hash chain (HARD-21). 20+ cases.

use chrono::Utc;
use eel_events_rs::audit_chain::{
    AuditEntry, ChainError, ChainVerifier, ChainedAudit, InMemoryChain,
};
use serde_json::json;

const SVC: &str = "eel.rts-svc";
const RES: &str = "station:47-001";

fn make_entry_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

fn append_one(chain: &mut InMemoryChain, action: &str, resource: &str) -> AuditEntry {
    chain
        .append(
            make_entry_id(),
            SVC.to_owned(),
            action.to_owned(),
            "spiffe://eel.local/svc/rts".to_owned(),
            resource.to_owned(),
            Utc::now(),
            json!({"action": action}),
        )
        .expect("append should succeed")
}

// ---- append round-trip ----

#[test]
fn first_entry_has_zero_prev_hash() {
    let mut chain = InMemoryChain::new();
    let e = append_one(&mut chain, "station_opened", RES);
    assert_eq!(e.prev_hash, [0u8; 32]);
}

#[test]
fn second_entry_prev_hash_equals_first_this_hash() {
    let mut chain = InMemoryChain::new();
    let e1 = append_one(&mut chain, "station_opened", RES);
    let e2 = append_one(&mut chain, "form34a_submitted", RES);
    assert_eq!(e2.prev_hash, e1.this_hash);
}

#[test]
fn this_hash_matches_recomputed_value() {
    let mut chain = InMemoryChain::new();
    let e = append_one(&mut chain, "station_opened", RES);
    let expected = AuditEntry::compute_hash(&e.prev_hash, &e.payload, &e.entry_id);
    assert_eq!(e.this_hash, expected);
}

#[test]
fn single_entry_chain_verifies_ok() {
    let mut chain = InMemoryChain::new();
    append_one(&mut chain, "station_opened", RES);
    ChainVerifier::verify_all(chain.entries()).expect("single entry should verify");
}

#[test]
fn multi_entry_chain_verifies_ok() {
    let mut chain = InMemoryChain::new();
    for i in 0..5 {
        append_one(&mut chain, &format!("action_{i}"), RES);
    }
    ChainVerifier::verify_all(chain.entries()).expect("5-entry chain should verify");
}

#[test]
fn is_self_consistent_returns_true_for_valid_entry() {
    let mut chain = InMemoryChain::new();
    let e = append_one(&mut chain, "form34a_submitted", RES);
    assert!(e.is_self_consistent());
}

// ---- tampered entry breaks chain ----

#[test]
fn tampered_payload_breaks_self_consistency() {
    let mut chain = InMemoryChain::new();
    let mut e = append_one(&mut chain, "form34a_submitted", RES);
    e.payload = json!({"tampered": true});
    assert!(!e.is_self_consistent());
}

#[test]
fn tampered_payload_breaks_verify_all() {
    let mut chain = InMemoryChain::new();
    append_one(&mut chain, "station_opened", RES);
    append_one(&mut chain, "form34a_submitted", RES);
    let mut entries = chain.entries().to_vec();
    entries[0].payload = json!({"rigged": true});
    let err = ChainVerifier::verify_all(&entries).unwrap_err();
    assert!(matches!(err, ChainError::HashMismatch { .. }));
}

#[test]
fn tampered_this_hash_breaks_verify_all() {
    let mut chain = InMemoryChain::new();
    append_one(&mut chain, "station_opened", RES);
    append_one(&mut chain, "form34a_submitted", RES);
    let mut entries = chain.entries().to_vec();
    entries[0].this_hash = [0xff; 32];
    let err = ChainVerifier::verify_all(&entries).unwrap_err();
    assert!(matches!(err, ChainError::HashMismatch { .. }));
}

#[test]
fn tampered_prev_hash_breaks_chain() {
    let mut chain = InMemoryChain::new();
    append_one(&mut chain, "station_opened", RES);
    append_one(&mut chain, "form34a_submitted", RES);
    let mut entries = chain.entries().to_vec();
    entries[1].prev_hash = [0xaa; 32];
    let err = ChainVerifier::verify_all(&entries).unwrap_err();
    assert!(matches!(err, ChainError::HashMismatch { .. }));
}

// ---- missing entry breaks chain ----

#[test]
fn removing_middle_entry_breaks_chain() {
    let mut chain = InMemoryChain::new();
    for i in 0..4 {
        append_one(&mut chain, &format!("action_{i}"), RES);
    }
    let mut entries = chain.entries().to_vec();
    entries.remove(1); // remove second entry
    let err = ChainVerifier::verify_all(&entries).unwrap_err();
    assert!(matches!(err, ChainError::HashMismatch { .. }));
}

#[test]
fn removing_first_entry_breaks_chain() {
    let mut chain = InMemoryChain::new();
    for i in 0..3 {
        append_one(&mut chain, &format!("action_{i}"), RES);
    }
    let mut entries = chain.entries().to_vec();
    entries.remove(0);
    // Second entry's prev_hash no longer matches [0u8;32].
    let err = ChainVerifier::verify_all(&entries).unwrap_err();
    assert!(matches!(err, ChainError::HashMismatch { .. }));
}

#[test]
fn empty_chain_returns_empty_chain_error() {
    let err = ChainVerifier::verify_all(&[]).unwrap_err();
    assert!(matches!(err, ChainError::EmptyChain));
}

// ---- cross-service chain composes correctly ----

#[test]
fn cross_service_separate_resources_independent_chains() {
    let mut chain = InMemoryChain::new();
    let res_a = "station:47-001";
    let res_b = "station:47-002";
    let e_a1 = append_one(&mut chain, "opened", res_a);
    let e_b1 = append_one(&mut chain, "opened", res_b);
    // Each resource starts from zero.
    assert_eq!(e_a1.prev_hash, [0u8; 32]);
    assert_eq!(e_b1.prev_hash, [0u8; 32]);

    let e_a2 = append_one(&mut chain, "form34a", res_a);
    assert_eq!(e_a2.prev_hash, e_a1.this_hash);

    let e_b2 = append_one(&mut chain, "form34a", res_b);
    assert_eq!(e_b2.prev_hash, e_b1.this_hash);
}

#[test]
fn cross_service_chain_each_resource_verifies_independently() {
    let mut chain = InMemoryChain::new();
    let res_a = "station:47-001";
    let res_b = "station:47-002";
    for _ in 0..3 {
        append_one(&mut chain, "action", res_a);
    }
    for _ in 0..3 {
        append_one(&mut chain, "action", res_b);
    }
    let a_entries: Vec<_> = chain
        .entries()
        .iter()
        .filter(|e| e.resource == res_a)
        .cloned()
        .collect();
    let b_entries: Vec<_> = chain
        .entries()
        .iter()
        .filter(|e| e.resource == res_b)
        .cloned()
        .collect();
    ChainVerifier::verify_all(&a_entries).expect("resource-a chain ok");
    ChainVerifier::verify_all(&b_entries).expect("resource-b chain ok");
}

// ---- determinism and canonical JSON ----

#[test]
fn compute_hash_is_deterministic() {
    let prev = [0u8; 32];
    let payload = json!({"b": 2, "a": 1});
    let id = "test-id";
    let h1 = AuditEntry::compute_hash(&prev, &payload, id);
    let h2 = AuditEntry::compute_hash(&prev, &payload, id);
    assert_eq!(h1, h2);
}

#[test]
fn compute_hash_different_for_different_payloads() {
    let prev = [0u8; 32];
    let id = "test-id";
    let h1 = AuditEntry::compute_hash(&prev, &json!({"v": 1}), id);
    let h2 = AuditEntry::compute_hash(&prev, &json!({"v": 2}), id);
    assert_ne!(h1, h2);
}

#[test]
fn compute_hash_different_for_different_entry_ids() {
    let prev = [0u8; 32];
    let payload = json!({"v": 1});
    let h1 = AuditEntry::compute_hash(&prev, &payload, "id-1");
    let h2 = AuditEntry::compute_hash(&prev, &payload, "id-2");
    assert_ne!(h1, h2);
}

#[test]
fn canonical_json_key_order_does_not_affect_hash() {
    let prev = [0u8; 32];
    let id = "canon-test";
    // Same logical object, different insertion order.
    let p1 = json!({"a": 1, "b": 2});
    let p2 = json!({"b": 2, "a": 1});
    let h1 = AuditEntry::compute_hash(&prev, &p1, id);
    let h2 = AuditEntry::compute_hash(&prev, &p2, id);
    assert_eq!(h1, h2, "canonical JSON must normalise key order");
}

// ---- tip_hash advances correctly ----

#[test]
fn tip_hash_zero_before_first_append() {
    let chain = InMemoryChain::new();
    let tip = chain.tip_hash(RES).unwrap();
    assert_eq!(tip, [0u8; 32]);
}

#[test]
fn tip_hash_matches_last_entry_this_hash() {
    let mut chain = InMemoryChain::new();
    let e = append_one(&mut chain, "last", RES);
    let tip = chain.tip_hash(RES).unwrap();
    assert_eq!(tip, e.this_hash);
}
