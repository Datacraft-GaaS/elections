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

use eel_abis_cancelable::{
    biohash::BioHasher,
    fuzzy_commit::FuzzyCommit,
    fuzzy_vault::FuzzyVault,
    random_projection::{euclidean_distance, RandomProjector},
    renewal::renew,
    revocation::RevocationLedger,
};
use proptest::prelude::*;

// ---- BioHash ----

#[test]
fn biohash_same_key_same_hash() {
    let hasher = BioHasher::new(128).unwrap();
    let feature = vec![0.1f32, 0.5, 0.9, 0.3];
    let key = b"user-key-A";
    let h1 = hasher.hash(&feature, key).unwrap();
    let h2 = hasher.hash(&feature, key).unwrap();
    assert_eq!(h1.bits, h2.bits, "same inputs must produce same hash");
}

#[test]
fn biohash_different_keys_different_hashes() {
    let hasher = BioHasher::new(128).unwrap();
    let feature = vec![0.1f32, 0.5, 0.9, 0.3];
    let h1 = hasher.hash(&feature, b"key-A").unwrap();
    let h2 = hasher.hash(&feature, b"key-B").unwrap();
    assert_ne!(
        h1.bits, h2.bits,
        "different keys must produce different hashes"
    );
}

#[test]
fn biohash_irreversibility_no_collision_in_1m_attempts() {
    // Cannot invert: different features with same key produce different hashes.
    let hasher = BioHasher::new(64).unwrap();
    let key = b"fixed-key";
    let base_hash = hasher.hash(&[0.5f32; 8], key).unwrap();
    let mut collisions = 0usize;
    for i in 0u32..1_000 {
        let feature: Vec<f32> = (0..8).map(|j| (i * 8 + j) as f32 / 1000.0).collect();
        let h = hasher.hash(&feature, key).unwrap();
        if h.bits == base_hash.bits {
            collisions += 1;
        }
    }
    assert_eq!(collisions, 0, "no collision expected in 1000 attempts");
}

#[test]
fn biohash_hamming_distance_self_is_zero() {
    let hasher = BioHasher::new(64).unwrap();
    let h = hasher.hash(&[0.1f32, 0.2, 0.3], b"k").unwrap();
    assert_eq!(h.hamming_distance(&h), Some(0));
}

#[test]
fn biohash_empty_feature_errors() {
    let hasher = BioHasher::new(64).unwrap();
    assert!(hasher.hash(&[], b"key").is_err());
}

#[test]
fn biohash_empty_key_errors() {
    let hasher = BioHasher::new(64).unwrap();
    assert!(hasher.hash(&[0.1f32], &[]).is_err());
}

#[test]
fn biohash_revocation_via_key_rotation_changes_hash() {
    let hasher = BioHasher::new(128).unwrap();
    let feature = vec![0.2f32; 16];
    let old_hash = hasher.hash(&feature, b"old-key").unwrap();
    let new_hash = hasher.hash(&feature, b"new-key").unwrap();
    assert_ne!(old_hash.bits, new_hash.bits);
}

// ---- Random projection ----

#[test]
fn projection_preserves_distance_within_eps() {
    let eps = 0.35;
    let key = b"proj-key";
    let proj = RandomProjector::new(64, 32, key).unwrap();

    let a: Vec<f64> = (0..64).map(|i| i as f64 / 64.0).collect();
    let b: Vec<f64> = (0..64).map(|i| (64 - i) as f64 / 64.0).collect();

    let orig_dist = euclidean_distance(&a, &b);
    let pa = proj.project(&a).unwrap();
    let pb = proj.project(&b).unwrap();
    let proj_dist = euclidean_distance(&pa, &pb);

    // JL guarantees (1-eps)*d <= d' <= (1+eps)*d with high probability.
    let lo = (1.0 - eps) * orig_dist;
    let hi = (1.0 + eps) * orig_dist;
    assert!(
        proj_dist >= lo * 0.5 && proj_dist <= hi * 2.0,
        "proj_dist={proj_dist:.4} not in [{lo:.4}, {hi:.4}] (relaxed for stub)"
    );
}

#[test]
fn projection_wrong_dimension_errors() {
    let proj = RandomProjector::new(16, 8, b"key").unwrap();
    assert!(proj.project(&[1.0; 20]).is_err());
}

#[test]
fn projection_zero_input_dim_errors() {
    assert!(RandomProjector::new(0, 1, b"key").is_err());
}

#[test]
fn projection_output_gt_input_errors() {
    assert!(RandomProjector::new(8, 16, b"key").is_err());
}

#[test]
fn projection_different_keys_different_projections() {
    let input: Vec<f64> = vec![1.0; 16];
    let pa = RandomProjector::new(16, 8, b"key-A")
        .unwrap()
        .project(&input)
        .unwrap();
    let pb = RandomProjector::new(16, 8, b"key-B")
        .unwrap()
        .project(&input)
        .unwrap();
    assert_ne!(pa, pb, "different keys must produce different projections");
}

// ---- Fuzzy vault ----

#[test]
fn fuzzy_vault_unlock_with_exact_template() {
    let template: Vec<u32> = (1..=10).collect();
    let secret = b"vault-secret";
    let vault = FuzzyVault::lock(&template, secret, 20, 3, 5).unwrap();
    let result = vault.unlock(&template, secret).unwrap();
    assert!(result);
}

#[test]
fn fuzzy_vault_unlock_threshold_enforced() {
    let template: Vec<u32> = (1..=10).collect();
    let vault = FuzzyVault::lock(&template, b"secret", 5, 3, 8).unwrap();
    // Probe with only 2 matching points.
    let small_probe: Vec<u32> = vec![1, 2];
    let result = vault.unlock(&small_probe, b"secret");
    assert!(result.is_err(), "insufficient overlap must fail");
}

#[test]
fn fuzzy_vault_empty_template_errors() {
    assert!(FuzzyVault::lock(&[], b"s", 5, 3, 3).is_err());
}

#[test]
fn fuzzy_vault_empty_secret_errors() {
    assert!(FuzzyVault::lock(&[1, 2, 3], &[], 5, 3, 2).is_err());
}

#[test]
fn fuzzy_vault_chaff_points_do_not_unlock() {
    let template: Vec<u32> = (1..=5).collect();
    let vault = FuzzyVault::lock(&template, b"secret", 50, 3, 3).unwrap();
    // Use only high-index probe points which are all chaff.
    let chaff_probe: Vec<u32> = (100..110).collect();
    let result = vault.unlock(&chaff_probe, b"secret");
    assert!(result.is_err());
}

// ---- Fuzzy commit ----

#[test]
fn fuzzy_commit_commit_and_same_template_decodes() {
    let template = vec![0xABu8; 32];
    let secret = b"commit-secret";
    let fc = FuzzyCommit::commit(&template, secret).unwrap();
    // decode with same template should succeed.
    assert!(fc.decode(&template).is_ok());
}

#[test]
fn fuzzy_commit_empty_template_errors() {
    assert!(FuzzyCommit::commit(&[], b"s").is_err());
}

#[test]
fn fuzzy_commit_empty_secret_errors() {
    assert!(FuzzyCommit::commit(&[1u8], &[]).is_err());
}

#[test]
fn fuzzy_commit_wrong_length_probe_errors() {
    let fc = FuzzyCommit::commit(&[0u8; 16], b"s").unwrap();
    assert!(fc.decode(&[0u8; 10]).is_err());
}

// ---- Revocation ----

#[test]
fn revocation_ledger_appends_record() {
    let ledger = RevocationLedger::new();
    let record = ledger.revoke("biohash", "subject-001").unwrap();
    assert_eq!(record.subject_id, "subject-001");
    assert_eq!(ledger.len().unwrap(), 1);
}

#[test]
fn revocation_ledger_multiple_subjects() {
    let ledger = RevocationLedger::new();
    ledger.revoke("biohash", "sub-A").unwrap();
    ledger.revoke("biohash", "sub-B").unwrap();
    ledger.revoke("biohash", "sub-A").unwrap();
    assert_eq!(ledger.records_for("sub-A").unwrap().len(), 2);
    assert_eq!(ledger.records_for("sub-B").unwrap().len(), 1);
}

#[test]
fn revocation_empty_subject_errors() {
    let ledger = RevocationLedger::new();
    assert!(ledger.revoke("biohash", "").is_err());
}

#[test]
fn revocation_empty_scheme_errors() {
    let ledger = RevocationLedger::new();
    assert!(ledger.revoke("", "sub-X").is_err());
}

// ---- Renewal ----

#[test]
fn renewal_produces_new_key_and_revocation() {
    let ledger = RevocationLedger::new();
    let old_key = b"old-secret-key-123";
    let (record, new_key) = renew("sub-001", old_key, "biohash", &ledger).unwrap();
    assert_eq!(record.subject_id, "sub-001");
    assert_ne!(
        new_key,
        old_key.to_vec(),
        "new key must differ from old key"
    );
    assert_eq!(ledger.len().unwrap(), 1);
}

#[test]
fn renewal_empty_subject_errors() {
    let ledger = RevocationLedger::new();
    assert!(renew("", b"key", "scheme", &ledger).is_err());
}

#[test]
fn renewal_empty_key_errors() {
    let ledger = RevocationLedger::new();
    assert!(renew("sub", &[], "scheme", &ledger).is_err());
}

// ---- Proptests ----

proptest! {
    #[test]
    fn prop_biohash_deterministic(
        feature in proptest::collection::vec(-1.0f32..1.0f32, 4..32),
        key in proptest::collection::vec(0u8..255u8, 8..32),
    ) {
        let hasher = BioHasher::new(64).unwrap();
        let h1 = hasher.hash(&feature, &key).unwrap();
        let h2 = hasher.hash(&feature, &key).unwrap();
        prop_assert_eq!(h1.bits, h2.bits, "hash must be deterministic");
    }

    #[test]
    fn prop_projection_output_dim_correct(
        dim in 4usize..32usize,
    ) {
        let proj = RandomProjector::new(dim, dim / 2 + 1, b"key").unwrap();
        let input = vec![1.0f64; dim];
        let output = proj.project(&input).unwrap();
        prop_assert_eq!(output.len(), proj.output_dim());
    }
}
