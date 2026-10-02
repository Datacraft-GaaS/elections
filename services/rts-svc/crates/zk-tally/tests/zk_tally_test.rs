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

//! Integration tests for the zk-tally crate.
//!
//! These tests use a small synthetic corpus (100 stations) so they run in CI
//! within a few seconds. Full 46,229-station circuits are exercised by the
//! benchmarks and the nightly CI job.

use zk_tally::{
    circuit::{PublishedTally, TallyLeaf},
    serialise,
    setup::generate_params,
    TallyProver, TallyVerifier,
};

fn make_leaves(counts: &[u64]) -> Vec<TallyLeaf> {
    counts
        .iter()
        .enumerate()
        .map(|(i, &v)| TallyLeaf {
            station_id: format!("KE-01-{i:04}"),
            votes: v,
        })
        .collect()
}

fn make_published(leaves: &[TallyLeaf]) -> PublishedTally {
    PublishedTally {
        total_votes: leaves.iter().map(|l| l.votes).sum(),
    }
}

/// Generate params + prove + verify on a synthetic 100-station corpus.
#[test]
fn proof_round_trip_100_stations() {
    let counts: Vec<u64> = (1u64..=100).map(|i| i * 137 + 500).collect();
    let leaves = make_leaves(&counts);
    let published = make_published(&leaves);
    let params = generate_params(100).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    assert!(TallyVerifier::verify(&proof, &params.vk(), &published).expect("verify"));
}

/// Verifier accepts a valid proof.
#[test]
fn verifier_accepts_valid_proof() {
    let leaves = make_leaves(&[1000, 2000, 3000, 4000]);
    let published = make_published(&leaves);
    let params = generate_params(4).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    let result = TallyVerifier::verify(&proof, &params.vk(), &published).expect("verify");
    assert!(result);
}

/// Verifier rejects a proof when the published tally is wrong.
#[test]
fn verifier_rejects_mismatched_published_tally() {
    let leaves = make_leaves(&[1000, 2000]);
    let correct = make_published(&leaves);
    let params = generate_params(2).expect("setup");
    let proof = TallyProver::prove(&leaves, &correct, &params).expect("prove");
    let wrong = PublishedTally { total_votes: 9999 };
    let result = TallyVerifier::verify(&proof, &params.vk(), &wrong).expect("verify");
    assert!(!result);
}

/// Prover returns TallyMismatch error when leaves do not sum to published total.
#[test]
fn prover_rejects_mismatched_leaves() {
    let leaves = make_leaves(&[1000, 2000]);
    let wrong_published = PublishedTally { total_votes: 9999 };
    let params = generate_params(2).expect("setup");
    let result = TallyProver::prove(&leaves, &wrong_published, &params);
    assert!(matches!(
        result,
        Err(zk_tally::ZkError::TallyMismatch { .. })
    ));
}

/// Proof serialises to a compact byte representation.
#[test]
fn proof_serialises_to_bytes() {
    let leaves = make_leaves(&[100, 200]);
    let published = make_published(&leaves);
    let params = generate_params(2).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    let bytes = serialise::compress(&proof).expect("compress");
    assert!(!bytes.is_empty());
}

/// Serialised proof size is below 512 bytes.
#[test]
fn proof_size_below_512_bytes() {
    let leaves = make_leaves(&[500, 600, 700]);
    let published = make_published(&leaves);
    let params = generate_params(3).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    let bytes = serialise::compress(&proof).expect("compress");
    assert!(
        bytes.len() < 512,
        "proof is {} bytes, expected < 512",
        bytes.len()
    );
}

/// Proof round-trips through serialise::compress / decompress.
#[test]
fn proof_compress_decompress_round_trip() {
    let leaves = make_leaves(&[111, 222]);
    let published = make_published(&leaves);
    let params = generate_params(2).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    let bytes = serialise::compress(&proof).expect("compress");
    let proof2 = serialise::decompress(&bytes).expect("decompress");
    assert!(TallyVerifier::verify(&proof2, &params.vk(), &published).expect("verify"));
}

/// Proof round-trips through hex encoding.
#[test]
fn proof_hex_round_trip() {
    let leaves = make_leaves(&[42, 58]);
    let published = make_published(&leaves);
    let params = generate_params(2).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    let hex = serialise::to_hex(&proof).expect("to_hex");
    let proof2 = serialise::from_hex(&hex).expect("from_hex");
    assert!(TallyVerifier::verify(&proof2, &params.vk(), &published).expect("verify"));
}

/// Verifier rejects a proof with a flipped bit.
#[test]
fn verifier_rejects_bitflipped_proof() {
    let leaves = make_leaves(&[1000, 2000]);
    let published = make_published(&leaves);
    let params = generate_params(2).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    let mut bytes = serialise::compress(&proof).expect("compress");
    bytes[0] ^= 0xFF;
    // Bitflipped bytes may fail to deserialise or verify.
    match serialise::decompress(&bytes) {
        Err(_) => {} // expected: corrupt proof rejected at deserialise
        Ok(bad_proof) => {
            let ok = TallyVerifier::verify(&bad_proof, &params.vk(), &published).unwrap_or(false);
            assert!(!ok, "bitflipped proof must not verify");
        }
    }
}

/// Stations with zero votes are valid (padded leaves).
#[test]
fn zero_vote_stations_are_valid() {
    let leaves = make_leaves(&[0, 0, 500]);
    let published = make_published(&leaves);
    let params = generate_params(3).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    assert!(TallyVerifier::verify(&proof, &params.vk(), &published).expect("verify"));
}

/// A single station is a valid degenerate case.
#[test]
fn single_station_proof() {
    let leaves = make_leaves(&[99_999]);
    let published = make_published(&leaves);
    let params = generate_params(1).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    assert!(TallyVerifier::verify(&proof, &params.vk(), &published).expect("verify"));
}

/// Proving key vk() returns a consistent verifying key.
#[test]
fn vk_is_consistent() {
    let leaves = make_leaves(&[100, 200]);
    let published = make_published(&leaves);
    let params = generate_params(2).expect("setup");
    let vk1 = params.vk();
    let vk2 = params.vk();
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    assert!(TallyVerifier::verify(&proof, &vk1, &published).expect("v1"));
    assert!(TallyVerifier::verify(&proof, &vk2, &published).expect("v2"));
}

/// TallyCircuit::new rejects too many leaves.
#[test]
fn circuit_rejects_too_many_leaves() {
    use ark_bls12_381::Fr;
    use zk_tally::circuit::TallyCircuit;
    let leaves: Vec<TallyLeaf> = (0..5)
        .map(|i| TallyLeaf {
            station_id: format!("s{i}"),
            votes: 1,
        })
        .collect();
    let published = PublishedTally { total_votes: 5 };
    let result = TallyCircuit::<Fr>::new(&leaves, &published, 3);
    assert!(matches!(
        result,
        Err(zk_tally::ZkError::TooManyLeaves(5, 3))
    ));
}

/// Proof with all-zero leaves and zero published total.
#[test]
fn all_zero_tally_proof() {
    let leaves = make_leaves(&[0; 10]);
    let published = PublishedTally { total_votes: 0 };
    let params = generate_params(10).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    assert!(TallyVerifier::verify(&proof, &params.vk(), &published).expect("verify"));
}

/// Serialised proof size is exactly 192 bytes for BLS12-381 Groth16.
#[test]
fn proof_size_is_192_bytes() {
    let leaves = make_leaves(&[1, 2, 3]);
    let published = make_published(&leaves);
    let params = generate_params(3).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    let bytes = serialise::compress(&proof).expect("compress");
    // Groth16 over BLS12-381: 2 * 48 (G1) + 96 (G2) = 192 bytes.
    assert_eq!(
        bytes.len(),
        192,
        "expected 192-byte Groth16 proof, got {}",
        bytes.len()
    );
}

/// Verifier rejects proof produced by a different proving key.
#[test]
fn verifier_rejects_wrong_vk() {
    let leaves = make_leaves(&[100, 200]);
    let published = make_published(&leaves);
    let params1 = generate_params(2).expect("setup 1");
    let params2 = generate_params(2).expect("setup 2");
    let proof = TallyProver::prove(&leaves, &published, &params1).expect("prove");
    // Proof from params1 should not verify under params2's vk.
    let result = TallyVerifier::verify(&proof, &params2.vk(), &published).expect("verify");
    assert!(!result);
}

/// Empty input (no leaves, zero total) is valid.
#[test]
fn empty_station_list() {
    let leaves: Vec<TallyLeaf> = vec![];
    let published = PublishedTally { total_votes: 0 };
    let params = generate_params(0).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    assert!(TallyVerifier::verify(&proof, &params.vk(), &published).expect("verify"));
}

/// Verification time is under 1 second on a local machine.
///
/// This test measures wall time and fails if verify takes > 1,000 ms.
/// In practice Groth16 over BLS12-381 verifies in < 10 ms.
#[test]
fn verify_completes_under_one_second() {
    use std::time::Instant;
    let leaves = make_leaves(&[1, 2, 3, 4, 5]);
    let published = make_published(&leaves);
    let params = generate_params(5).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    let start = Instant::now();
    let ok = TallyVerifier::verify(&proof, &params.vk(), &published).expect("verify");
    let elapsed = start.elapsed();
    assert!(ok);
    assert!(
        elapsed.as_millis() < 1_000,
        "verify took {}ms, expected < 1000ms",
        elapsed.as_millis()
    );
}

/// Proof with large vote counts (realistic Kenya scale).
#[test]
fn realistic_vote_counts() {
    // Kenya 2022: ~14.2 million valid votes across ~46k stations
    // Average ~307 votes per station; use 10 stations scaled for speed.
    let counts: Vec<u64> = (0..10).map(|i| 300_000 + i * 7_777).collect();
    let leaves = make_leaves(&counts);
    let published = make_published(&leaves);
    let params = generate_params(10).expect("setup");
    let proof = TallyProver::prove(&leaves, &published, &params).expect("prove");
    assert!(TallyVerifier::verify(&proof, &params.vk(), &published).expect("verify"));
}

/// Circuit TooManyLeaves error for production MAX_STATIONS limit.
#[test]
fn circuit_too_many_leaves_error_message() {
    let err = zk_tally::ZkError::TooManyLeaves(50_000, 46_229);
    let msg = err.to_string();
    assert!(msg.contains("50000"));
    assert!(msg.contains("46229"));
}
