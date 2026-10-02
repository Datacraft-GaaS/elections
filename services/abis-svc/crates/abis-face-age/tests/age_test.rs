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

use eel_abis_face_age::{
    AgeError, AgeEstimator, AgeInvariantMatcher, AgeProgressionStub, HashAgeEstimator, EMBED_DIM,
};

fn unit_emb(i: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; EMBED_DIM];
    v[i % EMBED_DIM] = 1.0;
    v
}

fn hash_emb(seed: u8) -> Vec<f32> {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update([seed]);
    let d: [u8; 32] = h.finalize().into();
    let mut v: Vec<f32> = d.iter().map(|&b| b as f32 - 128.0).collect();
    v.resize(EMBED_DIM, 0.0);
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    v.iter_mut().for_each(|x| *x /= norm);
    v
}

// --- HashAgeEstimator tests ---

#[test]
fn age_estimate_bounded_0_100() {
    let est = HashAgeEstimator::new();
    let emb = unit_emb(0);
    let res = est.estimate(&emb).unwrap();
    assert!(res.age <= 100, "age={}", res.age);
    assert!((0.0..=1.0).contains(&res.confidence));
}

#[test]
fn age_estimate_deterministic() {
    let est = HashAgeEstimator::new();
    let emb = unit_emb(3);
    let a = est.estimate(&emb).unwrap();
    let b = est.estimate(&emb).unwrap();
    assert_eq!(a.age, b.age);
}

#[test]
fn age_estimate_empty_errors() {
    let est = HashAgeEstimator::new();
    assert!(matches!(est.estimate(&[]), Err(AgeError::EmptyInput)));
}

#[test]
fn age_estimate_returns_plausible_age() {
    let est = HashAgeEstimator::new();
    let emb = hash_emb(42);
    let res = est.estimate(&emb).unwrap();
    assert!(res.age >= 15 && res.age < 80, "age={}", res.age);
}

// --- AgeInvariantMatcher tests ---

#[test]
fn age_invariant_same_embedding_high_similarity() {
    let matcher = AgeInvariantMatcher::new();
    let emb = hash_emb(7);
    let sim = matcher.similarity(&emb, &emb).unwrap();
    assert!((sim - 1.0).abs() < 0.01, "sim={}", sim);
}

#[test]
fn age_invariant_identity_pair_across_age_gap() {
    let matcher = AgeInvariantMatcher::new();
    let prog = AgeProgressionStub::new();
    let emb = hash_emb(5);
    let aged = prog.progress(&emb, 15).unwrap();
    let sim = matcher.similarity(&emb, &aged).unwrap();
    // Should remain reasonably high (> 0.8) for a small perturbation.
    assert!(sim > 0.8, "sim={}", sim);
}

#[test]
fn age_invariant_different_embeddings_lower_similarity() {
    let matcher = AgeInvariantMatcher::new();
    let a = hash_emb(1);
    let b = hash_emb(2);
    let sim = matcher.similarity(&a, &b).unwrap();
    // Different identities should score low.
    assert!(sim < 0.5, "sim={}", sim);
}

#[test]
fn age_invariant_wrong_dimension_errors() {
    let matcher = AgeInvariantMatcher::new();
    let short = vec![1.0f32; 10];
    let emb = hash_emb(0);
    assert!(matches!(
        matcher.similarity(&short, &emb),
        Err(AgeError::DimensionMismatch { .. })
    ));
}

// --- AgeProgressionStub tests ---

#[test]
fn progression_returns_same_length() {
    let prog = AgeProgressionStub::new();
    let emb = hash_emb(9);
    let out = prog.progress(&emb, 5).unwrap();
    assert_eq!(out.len(), EMBED_DIM);
}

#[test]
fn progression_is_l2_normalised() {
    let prog = AgeProgressionStub::new();
    let emb = hash_emb(9);
    let out = prog.progress(&emb, 10).unwrap();
    let norm: f32 = out.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "norm={}", norm);
}

#[test]
fn progression_different_years_different_output() {
    let prog = AgeProgressionStub::new();
    let emb = hash_emb(3);
    let a = prog.progress(&emb, 5).unwrap();
    let b = prog.progress(&emb, 15).unwrap();
    assert_ne!(a, b);
}

#[test]
fn progression_preserves_identity_cosine() {
    let prog = AgeProgressionStub::new();
    let emb = hash_emb(11);
    let aged = prog.progress(&emb, 15).unwrap();
    let sim: f32 = emb.iter().zip(aged.iter()).map(|(a, b)| a * b).sum();
    assert!(sim > 0.9, "cosine to original={}", sim);
}

#[test]
fn progression_wrong_dimension_errors() {
    let prog = AgeProgressionStub::new();
    assert!(matches!(
        prog.progress(&[1.0; 10], 5),
        Err(AgeError::DimensionMismatch { .. })
    ));
}
