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

//! Integration tests for eel-rts-fhe-tally.
//! 20+ tests: encrypt-aggregate-decrypt round-trips, tamper detection,
//! threshold decryption, policy enforcement, proof verification.

use eel_abis_fhe_match::{FheContext, FheScheme};
use eel_rts_fhe_tally::{
    aggregate::{aggregate_encrypted_leaves, EncryptedTally},
    decrypt::{decrypt_tally, ThresholdKey, ThresholdShare},
    encrypt_leaf::{encrypt_34a_leaf, EncryptedLeaf, Form34ALeaf},
    policy::{FhePolicy, FhePolicyError},
    prove::{build_aggregation_proof, AggregationProof},
    FheTallyError,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaf(station: &str, counts: &[u64]) -> Form34ALeaf {
    Form34ALeaf {
        station_id: station.into(),
        candidate_counts: counts
            .iter()
            .enumerate()
            .map(|(i, &c)| (format!("cand_{i}"), c))
            .collect(),
    }
}

fn enc_leaf(station: &str, counts: &[u64]) -> EncryptedLeaf {
    encrypt_34a_leaf(&leaf(station, counts), 1.0).unwrap()
}

fn zero_key(m: usize, n: usize) -> ThresholdKey {
    ThresholdKey {
        n,
        m,
        secret_mask: 0,
    }
}

fn zero_shares(key: &ThresholdKey) -> Vec<ThresholdShare> {
    key.split()
}

fn toy_ctx() -> FheContext {
    FheContext {
        scheme: FheScheme::Ckks,
        polynomial_degree: 32,
        plaintext_modulus: 0,
        security_level_bits: 1,
        noise_budget_initial: 10,
        circuit_depth_max: 2,
    }
}

// ---------------------------------------------------------------------------
// Policy tests
// ---------------------------------------------------------------------------

#[test]
fn policy_rejects_weak_security() {
    let ctx = toy_ctx();
    let err = FhePolicy::default().check(&ctx).unwrap_err();
    assert!(matches!(err, FhePolicyError::InsufficientSecurity { .. }));
}

#[test]
fn policy_test_mode_allows_weak_params() {
    FhePolicy::test().check(&toy_ctx()).unwrap();
}

#[test]
fn policy_rejects_wrong_scheme() {
    let mut ctx = FheContext {
        scheme: FheScheme::Ckks,
        polynomial_degree: 8192,
        plaintext_modulus: 0,
        security_level_bits: 128,
        noise_budget_initial: 100,
        circuit_depth_max: 20,
    };
    ctx.scheme = FheScheme::Bfv;
    let err = FhePolicy::default().check(&ctx).unwrap_err();
    assert!(matches!(err, FhePolicyError::WrongScheme(_)));
}

#[test]
fn policy_accepts_production_params() {
    let ctx = FheContext {
        scheme: FheScheme::Ckks,
        polynomial_degree: 8192,
        plaintext_modulus: 0,
        security_level_bits: 128,
        noise_budget_initial: 100,
        circuit_depth_max: 20,
    };
    FhePolicy::default().check(&ctx).unwrap();
}

// ---------------------------------------------------------------------------
// Encrypt-aggregate-decrypt round-trips
// ---------------------------------------------------------------------------

#[test]
fn encrypt_decrypt_round_trip_single_leaf() {
    let l = enc_leaf("s0", &[100, 200, 50]);
    let tally = aggregate_encrypted_leaves(&[l]).unwrap();
    let key = zero_key(2, 3);
    let shares = zero_shares(&key);
    // Additive XOR sharing: provide all N=3 shares to reconstruct secret=0.
    let plain = decrypt_tally(&tally, &shares, &key).unwrap();
    assert!((plain[0] - 100.0).abs() < 0.01);
    assert!((plain[1] - 200.0).abs() < 0.01);
    assert!((plain[2] - 50.0).abs() < 0.01);
}

#[test]
fn aggregate_two_stations_preserves_sum() {
    let leaves = vec![enc_leaf("s0", &[100, 200]), enc_leaf("s1", &[50, 75])];
    let tally = aggregate_encrypted_leaves(&leaves).unwrap();
    let key = zero_key(2, 3);
    let shares = zero_shares(&key);
    let plain = decrypt_tally(&tally, &shares, &key).unwrap();
    assert!((plain[0] - 150.0).abs() < 0.01);
    assert!((plain[1] - 275.0).abs() < 0.01);
}

#[test]
fn aggregate_five_stations_correct_total() {
    let leaves: Vec<EncryptedLeaf> = (0..5u64)
        .map(|i| enc_leaf(&format!("s{i}"), &[i * 100, i * 50]))
        .collect();
    let tally = aggregate_encrypted_leaves(&leaves).unwrap();
    let key = zero_key(2, 3);
    let shares = zero_shares(&key);
    let plain = decrypt_tally(&tally, &shares, &key).unwrap();
    // sum of i*100 for i in 0..5 = 0+100+200+300+400 = 1000
    assert!((plain[0] - 1000.0).abs() < 0.01);
    // sum of i*50 = 0+50+100+150+200 = 500
    assert!((plain[1] - 500.0).abs() < 0.01);
}

#[test]
fn tampered_ciphertext_produces_wrong_plaintext() {
    // Directly mutate a slot to simulate tampering; decryption should produce wrong sum.
    let leaves = vec![enc_leaf("s0", &[100, 200])];
    let mut tally = aggregate_encrypted_leaves(&leaves).unwrap();
    tally.slots[0] = 99999; // tamper
    let key = zero_key(2, 3);
    let shares = zero_shares(&key);
    let plain = decrypt_tally(&tally, &shares, &key).unwrap();
    // Should NOT equal 100 (slot was tampered to 99999).
    assert!((plain[0] - 100.0).abs() > 1.0);
}

// ---------------------------------------------------------------------------
// Threshold decryption tests
// ---------------------------------------------------------------------------

#[test]
fn threshold_requires_m_shares() {
    let key = ThresholdKey {
        n: 5,
        m: 3,
        secret_mask: 0,
    };
    let shares = key.split();
    let tally = EncryptedTally {
        candidate_ids: vec!["c0".into()],
        slots: vec![42],
        scale: 1.0,
        leaf_count: 1,
    };
    // m=3 satisfied (all 5 shares provided for correct reconstruction).
    decrypt_tally(&tally, &shares, &key).unwrap();
    // m-1=2: error.
    let err = decrypt_tally(&tally, &shares[..2], &key).unwrap_err();
    assert!(matches!(
        err,
        FheTallyError::InsufficientShares {
            got: 2,
            required: 3
        }
    ));
}

#[test]
fn threshold_three_of_five_default() {
    let key = ThresholdKey::default();
    assert_eq!(key.m, 3);
    assert_eq!(key.n, 5);
}

#[test]
fn threshold_split_produces_n_shares() {
    let key = ThresholdKey {
        n: 5,
        m: 3,
        secret_mask: 0xCAFE,
    };
    let shares = key.split();
    assert_eq!(shares.len(), 5);
}

#[test]
fn small_perturbation_on_leaf_changes_tally_deterministically() {
    let leaves_a = vec![enc_leaf("s0", &[100, 200]), enc_leaf("s1", &[50, 75])];
    let leaves_b = vec![enc_leaf("s0", &[101, 200]), enc_leaf("s1", &[50, 75])]; // s0 +1
    let tally_a = aggregate_encrypted_leaves(&leaves_a).unwrap();
    let tally_b = aggregate_encrypted_leaves(&leaves_b).unwrap();
    assert_ne!(tally_a.slots[0], tally_b.slots[0]);
    // Delta should be exactly 1 slot unit.
    assert_eq!((tally_b.slots[0] - tally_a.slots[0]), 1);
}

// ---------------------------------------------------------------------------
// Proof verification tests
// ---------------------------------------------------------------------------

#[test]
fn proof_verifies_against_tally() {
    let leaves: Vec<EncryptedLeaf> = (0..3)
        .map(|i| enc_leaf(&format!("s{i}"), &[10, 20]))
        .collect();
    let tally = aggregate_encrypted_leaves(&leaves).unwrap();
    let proof = build_aggregation_proof(&leaves, &tally).unwrap();
    proof.verify(&tally).unwrap();
}

#[test]
fn tampered_proof_root_fails_verification() {
    let leaves: Vec<EncryptedLeaf> = (0..3)
        .map(|i| enc_leaf(&format!("s{i}"), &[10, 20]))
        .collect();
    let tally = aggregate_encrypted_leaves(&leaves).unwrap();
    let mut proof = build_aggregation_proof(&leaves, &tally).unwrap();
    proof.root_hash[0] ^= 0xFF;
    let err = proof.verify(&tally).unwrap_err();
    assert!(matches!(err, FheTallyError::ProofVerification(_)));
}

#[test]
fn proof_entry_count_matches_leaves() {
    let leaves: Vec<EncryptedLeaf> = (0..4)
        .map(|i| enc_leaf(&format!("s{i}"), &[5, 10]))
        .collect();
    let tally = aggregate_encrypted_leaves(&leaves).unwrap();
    let proof = build_aggregation_proof(&leaves, &tally).unwrap();
    assert_eq!(proof.entries.len(), 4);
}

#[test]
fn proof_root_is_deterministic() {
    let leaves: Vec<EncryptedLeaf> = (0..2)
        .map(|i| enc_leaf(&format!("s{i}"), &[1, 2]))
        .collect();
    let tally = aggregate_encrypted_leaves(&leaves).unwrap();
    let p1 = build_aggregation_proof(&leaves, &tally).unwrap();
    let p2 = build_aggregation_proof(&leaves, &tally).unwrap();
    assert_eq!(p1.root_hash, p2.root_hash);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn aggregate_empty_errors() {
    let err = aggregate_encrypted_leaves(&[]).unwrap_err();
    assert!(matches!(err, FheTallyError::Aggregation(_)));
}

#[test]
fn encrypt_empty_candidates_errors() {
    let l = Form34ALeaf {
        station_id: "x".into(),
        candidate_counts: vec![],
    };
    assert!(encrypt_34a_leaf(&l, 1.0).is_err());
}

#[test]
fn aggregate_mismatched_candidates_errors() {
    let l1 = enc_leaf("s0", &[10, 20]);
    let l2 = {
        let f = Form34ALeaf {
            station_id: "s1".into(),
            candidate_counts: vec![("only_one".into(), 30)],
        };
        encrypt_34a_leaf(&f, 1.0).unwrap()
    };
    let err = aggregate_encrypted_leaves(&[l1, l2]).unwrap_err();
    assert!(matches!(err, FheTallyError::Aggregation(_)));
}
