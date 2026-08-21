use eel_abis_fhe_match::{
    batch_1n::BatchMatcher,
    context::{FheContext, FheScheme, SerializedContext},
    match_bfv::{BfvCiphertext, BfvMatcher},
    match_ckks::{CkksCiphertext, CkksMatchTrait, CkksMatcher},
    policy::FhePolicy,
};
use proptest::prelude::*;

// ---- Context ----

#[test]
fn context_ckks_preset_valid() {
    FheContext::ckks_face_preset().unwrap();
}

#[test]
fn context_bfv_preset_valid() {
    FheContext::bfv_iris_preset().unwrap();
}

#[test]
fn context_invalid_poly_degree_errors() {
    assert!(FheContext::new(FheScheme::Ckks, 1000, 0, 128, 300, 20).is_err());
}

#[test]
fn context_insufficient_security_errors() {
    assert!(FheContext::new(FheScheme::Bfv, 4096, 65537, 79, 200, 10).is_err());
}

#[test]
fn context_serialise_round_trip() {
    let ctx = FheContext::ckks_face_preset().unwrap();
    let sc = ctx.serialise().unwrap();
    let ctx2 = FheContext::deserialise(&sc).unwrap();
    assert_eq!(ctx.security_level_bits, ctx2.security_level_bits);
    assert_eq!(ctx.polynomial_degree, ctx2.polynomial_degree);
}

#[test]
fn context_invalid_json_errors() {
    let sc = SerializedContext {
        json: "not-json".into(),
    };
    assert!(FheContext::deserialise(&sc).is_err());
}

// ---- CKKS ----

#[test]
fn ckks_identical_vectors_score_near_one() {
    let scale = 1000.0;
    let v: Vec<f64> = (0..32).map(|i| i as f64 / 32.0).collect();
    let probe = CkksCiphertext::encrypt(&v, scale).unwrap();
    let gallery = CkksCiphertext::encrypt(&v, scale).unwrap();
    let matcher = CkksMatcher::new(scale);
    let score = matcher.match_1v1(&probe, &gallery, 0.0).unwrap();
    assert!(
        (score - 1.0).abs() < 0.01,
        "self-match score should be ~1.0, got {score}"
    );
}

#[test]
fn ckks_orthogonal_vectors_low_score() {
    let scale = 1000.0;
    let a: Vec<f64> = (0..32)
        .map(|i| if i % 2 == 0 { 1.0 } else { 0.0 })
        .collect();
    let b: Vec<f64> = (0..32)
        .map(|i| if i % 2 == 1 { 1.0 } else { 0.0 })
        .collect();
    let pa = CkksCiphertext::encrypt(&a, scale).unwrap();
    let pb = CkksCiphertext::encrypt(&b, scale).unwrap();
    let matcher = CkksMatcher::new(scale);
    let result = matcher.match_1v1(&pa, &pb, 0.5);
    assert!(result.is_err(), "orthogonal vectors should not match");
}

#[test]
fn ckks_dim_mismatch_errors() {
    let scale = 100.0;
    let p = CkksCiphertext::encrypt(&[1.0; 16], scale).unwrap();
    let g = CkksCiphertext::encrypt(&[1.0; 8], scale).unwrap();
    let matcher = CkksMatcher::new(scale);
    assert!(matcher.match_1v1(&p, &g, 0.0).is_err());
}

#[test]
fn ckks_empty_ciphertext_errors() {
    assert!(CkksCiphertext::encrypt(&[], 100.0).is_err());
}

#[test]
fn ckks_encrypt_decrypt_roundtrip() {
    let scale = 10000.0;
    let v: Vec<f64> = vec![0.1, 0.5, 0.9, -0.3];
    let ct = CkksCiphertext::encrypt(&v, scale).unwrap();
    let dec = ct.decrypt();
    for (orig, dec_v) in v.iter().zip(dec.iter()) {
        assert!(
            (orig - dec_v).abs() < 0.001,
            "decrypt mismatch: {orig} vs {dec_v}"
        );
    }
}

// ---- BFV ----

#[test]
fn bfv_identical_bitstrings_zero_distance() {
    let bits: Vec<u8> = (0..128).map(|i: u8| i % 2).collect();
    let probe = BfvCiphertext::encrypt(&bits).unwrap();
    let gallery = BfvCiphertext::encrypt(&bits).unwrap();
    let dist = BfvMatcher::match_1v1(&probe, &gallery, 100).unwrap();
    assert_eq!(dist, 0);
}

#[test]
fn bfv_high_hamming_above_threshold_errors() {
    let a: Vec<u8> = vec![0u8; 128];
    let b: Vec<u8> = vec![1u8; 128];
    let pa = BfvCiphertext::encrypt(&a).unwrap();
    let pb = BfvCiphertext::encrypt(&b).unwrap();
    assert!(BfvMatcher::match_1v1(&pa, &pb, 10).is_err());
}

#[test]
fn bfv_length_mismatch_errors() {
    let pa = BfvCiphertext::encrypt(&[0u8; 64]).unwrap();
    let pb = BfvCiphertext::encrypt(&[0u8; 32]).unwrap();
    assert!(BfvMatcher::match_1v1(&pa, &pb, 100).is_err());
}

#[test]
fn bfv_empty_bitstring_errors() {
    assert!(BfvCiphertext::encrypt(&[]).is_err());
}

// ---- Batch 1:N ----

#[test]
fn batch_1n_finds_best_match() {
    let scale = 1000.0;
    let probe_plain: Vec<f64> = vec![1.0; 32];
    let probe = CkksCiphertext::encrypt(&probe_plain, scale).unwrap();
    let mut gallery = Vec::new();
    // Add probe-identical entry at index 2.
    for i in 0..5usize {
        let v: Vec<f64> = (0..32)
            .map(|j| if i == 2 { 1.0 } else { j as f64 / 100.0 })
            .collect();
        gallery.push(CkksCiphertext::encrypt(&v, scale).unwrap());
    }
    let matcher = BatchMatcher::new(scale);
    let result = matcher.search_1n(&probe, &gallery, 0.5).unwrap();
    assert_eq!(result.best_idx, 2);
}

#[test]
fn batch_1n_empty_gallery_errors() {
    let scale = 100.0;
    let probe = CkksCiphertext::encrypt(&[1.0; 8], scale).unwrap();
    let matcher = BatchMatcher::new(scale);
    assert!(matcher.search_1n(&probe, &[], 0.5).is_err());
}

// ---- Policy ----

#[test]
fn policy_128bit_approves_valid_context() {
    let ctx = FheContext::ckks_face_preset().unwrap();
    let policy = FhePolicy::default_128bit();
    let outcome = policy.check(&ctx).unwrap();
    assert!(outcome.approved, "violations: {:?}", outcome.violations);
}

#[test]
fn policy_rejects_below_128_bit_security() {
    let ctx = FheContext::new(FheScheme::Ckks, 4096, 0, 80, 300, 20).unwrap();
    let policy = FhePolicy::default_128bit();
    let outcome = policy.check(&ctx).unwrap();
    assert!(!outcome.approved);
    assert!(outcome
        .violations
        .iter()
        .any(|v| v.contains("security_level_bits")));
}

#[test]
fn policy_rejects_excessive_circuit_depth() {
    let ctx = FheContext::new(FheScheme::Ckks, 8192, 0, 128, 300, 200).unwrap();
    let policy = FhePolicy::new(128, 50, 50);
    let outcome = policy.check(&ctx).unwrap();
    assert!(!outcome.approved);
    assert!(outcome
        .violations
        .iter()
        .any(|v| v.contains("circuit_depth_max")));
}

// ---- Proptest ----

proptest! {
    #[test]
    fn prop_ckks_self_match_score_near_one(
        dim in 4usize..32usize,
    ) {
        let scale = 10000.0;
        let v: Vec<f64> = (0..dim).map(|i| (i + 1) as f64 / dim as f64).collect();
        let ct = CkksCiphertext::encrypt(&v, scale).unwrap();
        let matcher = CkksMatcher::new(scale);
        let score = matcher.match_1v1(&ct, &ct, 0.0).unwrap();
        prop_assert!((score - 1.0).abs() < 0.01, "self match {score} should be ~1.0");
    }

    #[test]
    fn prop_bfv_zero_distance_on_identical(
        bits in proptest::collection::vec(0u8..2u8, 16..128),
    ) {
        let probe = BfvCiphertext::encrypt(&bits).unwrap();
        let gallery = BfvCiphertext::encrypt(&bits).unwrap();
        let dist = BfvMatcher::match_1v1(&probe, &gallery, 200).unwrap();
        prop_assert_eq!(dist, 0);
    }
}
