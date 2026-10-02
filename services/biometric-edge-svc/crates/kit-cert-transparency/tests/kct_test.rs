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

//! Integration tests for eel-kit-cert-transparency.
//!
//! Covers: append round-trip, inclusion proof verify, tampered cert rejected,
//! consistency proof between STHs, empty log STH.

use eel_kit_cert_transparency::{
    entry::KitCertEntry,
    issuance_hook::{self, IssuanceRequest},
    log::KitCertTransparencyLog,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn req(n: u8) -> IssuanceRequest {
    IssuanceRequest {
        kit_id: format!("kit-{n:03}"),
        cert_der: vec![0x30, 0x82, n],
        issuer_ca: "eel-step-ca".to_string(),
        spiffe_id: format!("spiffe://eel.internal/kit/{n:03}"),
        revocation_ref: None,
    }
}

fn append_n(log: &KitCertTransparencyLog, n: u8) -> (KitCertEntry, usize) {
    let r = req(n);
    let receipt = issuance_hook::intercept(log, r).unwrap();
    // Retrieve the exact entry stored in the log (issued_at is set by the hook).
    let entry = log.entry_at(receipt.leaf_index).unwrap();
    (entry, receipt.leaf_index)
}

// ---------------------------------------------------------------------------
// Test 1: empty log returns a valid (non-empty-string) STH
// ---------------------------------------------------------------------------
#[test]
fn empty_log_sth_valid() {
    let log = KitCertTransparencyLog::new();
    let sth = log.current_sth();
    assert_eq!(sth.tree_size, 0);
    assert!(!sth.sha256_root_hash.is_empty());
    // Root is SHA256("") with 0x00 prefix -- 64 hex chars.
    assert_eq!(sth.sha256_root_hash.len(), 64);
}

// ---------------------------------------------------------------------------
// Test 2: append and tree size increases
// ---------------------------------------------------------------------------
#[test]
fn append_increases_tree_size() {
    let log = KitCertTransparencyLog::new();
    assert_eq!(log.tree_size(), 0);
    issuance_hook::intercept(&log, req(1)).unwrap();
    assert_eq!(log.tree_size(), 1);
    issuance_hook::intercept(&log, req(2)).unwrap();
    assert_eq!(log.tree_size(), 2);
}

// ---------------------------------------------------------------------------
// Test 3: leaf index is monotonically increasing
// ---------------------------------------------------------------------------
#[test]
fn leaf_indices_monotonic() {
    let log = KitCertTransparencyLog::new();
    for i in 0u8..5 {
        let receipt = issuance_hook::intercept(&log, req(i)).unwrap();
        assert_eq!(receipt.leaf_index, i as usize);
    }
}

// ---------------------------------------------------------------------------
// Test 4: inclusion proof for first entry has empty audit path (single leaf)
// ---------------------------------------------------------------------------
#[test]
fn single_leaf_inclusion_proof_empty_path() {
    let log = KitCertTransparencyLog::new();
    issuance_hook::intercept(&log, req(1)).unwrap();
    let proof = log.inclusion_proof(0).unwrap();
    assert_eq!(proof.leaf_index, 0);
    assert_eq!(proof.tree_size, 1);
    assert!(proof.audit_path.is_empty());
}

// ---------------------------------------------------------------------------
// Test 5: inclusion proof round-trip (4 leaves, verify all)
// ---------------------------------------------------------------------------
#[test]
fn inclusion_proof_roundtrip_four_leaves() {
    let log = KitCertTransparencyLog::new();
    let mut entries = Vec::new();
    for i in 0u8..4 {
        let (entry, _) = append_n(&log, i);
        entries.push(entry);
    }
    for (i, entry) in entries.iter().enumerate() {
        let proof = log.inclusion_proof(i).unwrap();
        log.verify_inclusion(i, entry, &proof).unwrap();
    }
}

// ---------------------------------------------------------------------------
// Test 6: tampered cert fails inclusion proof verification
// ---------------------------------------------------------------------------
#[test]
fn tampered_cert_rejected() {
    let log = KitCertTransparencyLog::new();
    let (mut entry, idx) = append_n(&log, 10);
    let proof = log.inclusion_proof(idx).unwrap();
    // Tamper with the cert DER.
    entry.cert_der[0] ^= 0xFF;
    assert!(log.verify_inclusion(idx, &entry, &proof).is_err());
}

// ---------------------------------------------------------------------------
// Test 7: out-of-range inclusion proof returns error
// ---------------------------------------------------------------------------
#[test]
fn inclusion_proof_out_of_range() {
    let log = KitCertTransparencyLog::new();
    issuance_hook::intercept(&log, req(1)).unwrap();
    assert!(log.inclusion_proof(5).is_err());
}

// ---------------------------------------------------------------------------
// Test 8: STH root changes after append
// ---------------------------------------------------------------------------
#[test]
fn sth_root_changes_after_append() {
    let log = KitCertTransparencyLog::new();
    let sth0 = log.current_sth();
    issuance_hook::intercept(&log, req(1)).unwrap();
    let sth1 = log.current_sth();
    assert_ne!(sth0.sha256_root_hash, sth1.sha256_root_hash);
}

// ---------------------------------------------------------------------------
// Test 9: two identical certs get distinct leaf indices
// ---------------------------------------------------------------------------
#[test]
fn duplicate_certs_distinct_indices() {
    let log = KitCertTransparencyLog::new();
    let r1 = issuance_hook::intercept(&log, req(5)).unwrap();
    let r2 = issuance_hook::intercept(&log, req(5)).unwrap();
    assert_ne!(r1.leaf_index, r2.leaf_index);
}

// ---------------------------------------------------------------------------
// Test 10: consistency proof between tree sizes 2 and 4
// ---------------------------------------------------------------------------
#[test]
fn consistency_proof_2_to_4() {
    let log = KitCertTransparencyLog::new();
    for i in 0u8..4 {
        issuance_hook::intercept(&log, req(i)).unwrap();
    }
    let path = log.consistency_proof(2, 4).unwrap();
    assert!(!path.is_empty());
}

// ---------------------------------------------------------------------------
// Test 11: consistency proof same size is empty
// ---------------------------------------------------------------------------
#[test]
fn consistency_proof_same_size_empty() {
    let log = KitCertTransparencyLog::new();
    for i in 0u8..3 {
        issuance_hook::intercept(&log, req(i)).unwrap();
    }
    let path = log.consistency_proof(3, 3).unwrap();
    assert!(path.is_empty());
}

// ---------------------------------------------------------------------------
// Test 12: issuance hook rejects empty kit_id
// ---------------------------------------------------------------------------
#[test]
fn issuance_hook_rejects_empty_kit_id() {
    let log = KitCertTransparencyLog::new();
    let mut r = req(1);
    r.kit_id = String::new();
    assert!(issuance_hook::intercept(&log, r).is_err());
}

// ---------------------------------------------------------------------------
// Test 13: issuance hook rejects empty cert
// ---------------------------------------------------------------------------
#[test]
fn issuance_hook_rejects_empty_cert() {
    let log = KitCertTransparencyLog::new();
    let mut r = req(1);
    r.cert_der = vec![];
    assert!(issuance_hook::intercept(&log, r).is_err());
}

// ---------------------------------------------------------------------------
// Test 14: root_at consistency
// ---------------------------------------------------------------------------
#[test]
fn root_at_consistent_with_sth() {
    let log = KitCertTransparencyLog::new();
    for i in 0u8..3 {
        issuance_hook::intercept(&log, req(i)).unwrap();
    }
    let root3 = log.root_at(3).unwrap();
    let sth = log.current_sth();
    assert_eq!(root3, sth.sha256_root_hash);
}

// ---------------------------------------------------------------------------
// Test 15: root_at(0) matches empty-log STH root
// ---------------------------------------------------------------------------
#[test]
fn root_at_zero_matches_empty_sth() {
    let log_empty = KitCertTransparencyLog::new();
    let empty_root = log_empty.current_sth().sha256_root_hash;

    let log = KitCertTransparencyLog::new();
    issuance_hook::intercept(&log, req(1)).unwrap();
    let root0 = log.root_at(0).unwrap();
    assert_eq!(root0, empty_root);
}

// ---------------------------------------------------------------------------
// Test 16: inclusion proof verifies with revocation_ref set
// ---------------------------------------------------------------------------
#[test]
fn inclusion_proof_with_revocation_ref() {
    let log = KitCertTransparencyLog::new();
    let r = IssuanceRequest {
        kit_id: "kit-rev".to_string(),
        cert_der: vec![0xDE, 0xAD],
        issuer_ca: "eel-step-ca".to_string(),
        spiffe_id: "spiffe://eel.internal/kit/rev".to_string(),
        revocation_ref: Some("crl://eel.internal/revoke/rev".to_string()),
    };
    let receipt = issuance_hook::intercept(&log, r).unwrap();
    let proof = log.inclusion_proof(receipt.leaf_index).unwrap();
    // Check proof structure is correct (single-leaf tree: empty audit path).
    assert_eq!(proof.tree_size, 1);
    assert_eq!(proof.leaf_index, 0);
    // Also verify inclusion using the exact stored entry.
    let stored_entry = log.entry_at(receipt.leaf_index).unwrap();
    log.verify_inclusion(receipt.leaf_index, &stored_entry, &proof)
        .unwrap();
}
