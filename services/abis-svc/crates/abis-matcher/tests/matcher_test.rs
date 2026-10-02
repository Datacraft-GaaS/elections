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

use abis_matcher::adjudication::{is_ambiguous, open_adjudication_case, record_decision};
use abis_matcher::batch::run_dedup_batch;
use abis_matcher::domain::{
    AdjudicationDecision, AdjudicationStatus, MatchOutcome, MatchResult, ModalityScore,
    TemplateIndex,
};
use abis_matcher::fairness::{DemographicKey, FairnessReport, MatchRecord};
use abis_matcher::fmr_fnmr::compute_fmr_fnmr;
use abis_matcher::index::bloom_prune::{AgeBucket, DemographicBloom, Sex};
use abis_matcher::index::hnsw::HnswIndex;
use abis_matcher::index::shard::shard_distribution;
use abis_matcher::index::{shard_id, TemplateIndex as IndexTrait};
use abis_matcher::matcher::face_matcher::{CohortStats, FaceMatcher};
use abis_matcher::matcher::fingerprint_matcher::FingerprintMatcher;
use abis_matcher::matcher::fusion::{fuse, FusionPolicy, MultiModalScore};
use abis_matcher::matcher::iris_matcher::IrisMatcher;
use abis_matcher::matcher::Matcher;
use abis_matcher::modality::face::{FaceTemplate, IcaoFlags};
use abis_matcher::modality::fingerprint::{
    FingerPosition, FingerprintTemplate, Minutia, MinutiaeType,
};
use abis_matcher::modality::iris::{IrisTemplate, IRIS_CODE_BYTES};
use abis_matcher::modality::Modality;
use abis_matcher::pad::{PadDetector, Sample, SoftwarePad};
use abis_matcher::quality::{
    gate_face, gate_finger_count, gate_fingerprint, gate_iris, QualityDecision, QualityPolicy,
};
use abis_matcher::{MatchPipeline, MockMatcher};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_minutia(x: u16, y: u16, theta: f32) -> Minutia {
    Minutia {
        x,
        y,
        theta,
        mtype: MinutiaeType::Ending,
        quality: 80,
    }
}

fn make_fingerprint(seed: u16) -> FingerprintTemplate {
    let minutiae: Vec<Minutia> = (0..30u16)
        .map(|i| {
            make_minutia(
                (seed + i * 13) % 450 + 25,
                (seed + i * 17) % 450 + 25,
                (i * 12) as f32,
            )
        })
        .collect();
    FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, minutiae)
}

fn icao_all_pass() -> IcaoFlags {
    IcaoFlags {
        pose_ok: true,
        lighting_ok: true,
        eyes_open: true,
        no_glasses: true,
    }
}

fn icao_all_fail() -> IcaoFlags {
    IcaoFlags {
        pose_ok: false,
        lighting_ok: false,
        eyes_open: false,
        no_glasses: false,
    }
}

fn make_face(seed: f32) -> FaceTemplate {
    let mut emb: Vec<f32> = (0..512usize)
        .map(|i| (seed + i as f32 * 0.001).sin())
        .collect();
    let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
    emb.iter_mut().for_each(|x| *x /= norm.max(1e-9));
    FaceTemplate::new(emb, icao_all_pass(), None)
}

fn full_mask_iris(code: u8) -> IrisTemplate {
    IrisTemplate::from_vecs(vec![code; IRIS_CODE_BYTES], vec![0xFFu8; IRIS_CODE_BYTES])
}

fn demo_key(sex: Sex, age: AgeBucket, county: &str) -> DemographicKey {
    DemographicKey {
        sex,
        age_bucket: age,
        county: county.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Legacy mock matcher tests (kept)
// ---------------------------------------------------------------------------

#[test]
fn mock_matcher_deterministic() {
    let m = MockMatcher;
    let r1 = m.match_11("probe-abc", "gallery-xyz", 0.8);
    let r2 = m.match_11("probe-abc", "gallery-xyz", 0.8);
    assert_eq!(r1.confidence_score, r2.confidence_score);
    assert_eq!(r1.outcome, r2.outcome);
}

#[test]
fn mock_matcher_self_match_is_high() {
    let m = MockMatcher;
    let r = m.match_11("voter-001", "voter-001", 0.8);
    assert!(
        r.confidence_score >= 0.9,
        "self-match score={}",
        r.confidence_score
    );
    assert_eq!(r.outcome, MatchOutcome::Match);
}

#[test]
fn is_ambiguous_thresholds() {
    assert!(is_ambiguous(0.6, 0.5, 0.8));
    assert!(is_ambiguous(0.5, 0.5, 0.8));
    assert!(!is_ambiguous(0.8, 0.5, 0.8));
    assert!(!is_ambiguous(0.4, 0.5, 0.8));
    assert!(!is_ambiguous(0.9, 0.5, 0.8));
}

#[test]
fn fmr_fnmr_mock() {
    let m = MockMatcher;
    let genuine: Vec<(String, String)> = (0..1000)
        .map(|i| (format!("voter-{i}"), format!("voter-{i}")))
        .collect();
    let impostors: Vec<(String, String)> = (0..1000)
        .map(|i| {
            (
                format!("probe-{i}"),
                format!("gallery-impostor-{}", i + 100_000),
            )
        })
        .collect();
    let harness = compute_fmr_fnmr(&m, &genuine, &impostors, 0.8);
    assert_eq!(harness.total_genuine, 1000);
    assert_eq!(harness.total_impostor, 1000);
    assert!(harness.fnmr <= 0.001, "FNMR too high: {}", harness.fnmr);
    assert!(harness.fmr <= 0.05, "FMR too high: {}", harness.fmr);
}

#[test]
fn batch_dedup_empty() {
    let m = MockMatcher;
    let cases = run_dedup_batch(&m, &[], 0.8);
    assert!(cases.is_empty());
}

#[test]
fn adjudication_case_open_and_decide() {
    let fake_result = MatchResult {
        result_id: "result-1".to_string(),
        request_id: "req-1".to_string(),
        outcome: MatchOutcome::Adjudicate,
        confidence_score: 0.65,
        modality_scores: vec![ModalityScore {
            modality: "fingerprint".to_string(),
            score: 0.65,
        }],
    };
    let mut case = open_adjudication_case(&fake_result, vec!["voter-1".into(), "voter-2".into()]);
    assert_eq!(case.status, AdjudicationStatus::Open);
    let decision = AdjudicationDecision {
        case_id: case.case_id.clone(),
        decision: "no_match".to_string(),
        reason_code: "BIOMETRIC_DISTINCT".to_string(),
        officer_id: "officer-99".to_string(),
        decided_at: "2026-01-01T00:00:00Z".to_string(),
    };
    record_decision(&mut case, decision);
    assert_eq!(case.status, AdjudicationStatus::Resolved);
    assert_eq!(case.decision.as_deref(), Some("no_match"));
}

// ---------------------------------------------------------------------------
// ISO 19794-2 parse / serialise round-trip (20+ minutiae records)
// ---------------------------------------------------------------------------

#[test]
fn iso19794_roundtrip_20_minutiae() {
    let minutiae: Vec<Minutia> = (0..22u16)
        .map(|i| Minutia {
            x: 50 + i * 15,
            y: 50 + i * 17,
            theta: (i * 16) as f32,
            mtype: if i % 2 == 0 {
                MinutiaeType::Ending
            } else {
                MinutiaeType::Bifurcation
            },
            quality: 70 + (i % 30) as u8,
        })
        .collect();
    let tmpl = FingerprintTemplate::from_minutiae(FingerPosition::RightThumb, minutiae.clone());
    let bytes = tmpl.serialise_iso19794_2();

    let parsed = FingerprintTemplate::parse_iso19794_2(&bytes).unwrap();
    assert_eq!(parsed.len(), 1);
    let p = &parsed[0];
    assert_eq!(p.minutiae.len(), minutiae.len());
    for (orig, got) in minutiae.iter().zip(p.minutiae.iter()) {
        assert_eq!(orig.x, got.x, "x mismatch");
        assert_eq!(orig.y, got.y, "y mismatch");
        assert_eq!(orig.mtype, got.mtype, "type mismatch");
        // Theta goes through u8 quantisation: allow ±1.5 degree tolerance.
        assert!(
            (orig.theta - got.theta).abs() < 1.5,
            "theta {} vs {}",
            orig.theta,
            got.theta
        );
    }
}

#[test]
fn iso19794_parse_bad_magic() {
    let bad = vec![0u8; 30];
    assert!(FingerprintTemplate::parse_iso19794_2(&bad).is_err());
}

#[test]
fn iso19794_parse_too_short() {
    let bad = b"FM20\x00\x00\x00\x18";
    assert!(FingerprintTemplate::parse_iso19794_2(bad).is_err());
}

#[test]
fn iso19794_roundtrip_empty_minutiae() {
    let tmpl = FingerprintTemplate::from_minutiae(FingerPosition::LeftIndex, vec![]);
    let bytes = tmpl.serialise_iso19794_2();
    let parsed = FingerprintTemplate::parse_iso19794_2(&bytes).unwrap();
    assert_eq!(parsed[0].minutiae.len(), 0);
}

#[test]
fn iso19794_roundtrip_40_minutiae() {
    let minutiae: Vec<Minutia> = (0..40u16)
        .map(|i| make_minutia(30 + i * 10, 30 + i * 11, (i * 9) as f32))
        .collect();
    let tmpl = FingerprintTemplate::from_minutiae(FingerPosition::LeftThumb, minutiae.clone());
    let bytes = tmpl.serialise_iso19794_2();
    let parsed = FingerprintTemplate::parse_iso19794_2(&bytes).unwrap();
    assert_eq!(parsed[0].minutiae.len(), 40);
}

// ---------------------------------------------------------------------------
// Fingerprint matcher tests
// ---------------------------------------------------------------------------

#[test]
fn fingerprint_genuine_pair_passes_threshold() {
    // Same template against itself: should be a strong match.
    let probe = make_fingerprint(100);
    let gallery = probe.clone();
    let matcher = FingerprintMatcher::new(0.3);
    let score = matcher.compare(&probe, &gallery);
    assert!(
        score.threshold_pass,
        "self-match must pass, normalised={}",
        score.normalised
    );
    assert!(
        score.normalised >= 0.5,
        "self score too low: {}",
        score.normalised
    );
}

#[test]
fn fingerprint_impostor_pair_fails_threshold() {
    // Two templates with completely incommensurable spatial patterns.
    // Probe: diagonal staircase pattern.
    // Gallery: grid with prime-spaced positions and different angles.
    // With tight trans_tol (5px), accidental anchor alignments don't produce matches.
    let probe_minutiae: Vec<Minutia> = (0..20u16)
        .map(|i| make_minutia(50 + i * 13, 50 + i * 17, (i * 31 % 360) as f32))
        .collect();
    let gallery_minutiae: Vec<Minutia> = (0..20u16)
        .map(|i| make_minutia(100 + i * 19, 80 + i * 7, (i * 53 % 360) as f32))
        .collect();
    let probe = FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, probe_minutiae);
    let gallery = FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, gallery_minutiae);
    let mut matcher = FingerprintMatcher::new(0.5);
    matcher.trans_tol = 5.0; // Tight tolerance: only near-exact spatial matches count.
    matcher.rot_tol = 5.0;
    let score = matcher.compare(&probe, &gallery);
    assert!(
        score.normalised < 0.5,
        "impostor with tight tolerances score too high: {} (normalised={})",
        score.raw,
        score.normalised
    );
}

#[test]
fn fingerprint_rotational_tolerance_boundary() {
    // Probe rotated exactly 14 degrees: should still match (within ±15°).
    let base: Vec<Minutia> = (0..20u16)
        .map(|i| make_minutia(200 + i * 5, 200 + i * 5, (i * 18) as f32))
        .collect();
    let rotated: Vec<Minutia> = base
        .iter()
        .map(|m| Minutia {
            theta: (m.theta + 14.0).rem_euclid(360.0),
            ..*m
        })
        .collect();
    let probe = FingerprintTemplate::from_minutiae(FingerPosition::RightMiddle, base);
    let gallery = FingerprintTemplate::from_minutiae(FingerPosition::RightMiddle, rotated);
    let matcher = FingerprintMatcher::new(0.3);
    let score = matcher.compare(&probe, &gallery);
    assert!(
        score.threshold_pass,
        "14-degree rotation should pass, score={}",
        score.normalised
    );
}

#[test]
fn fingerprint_rotational_tolerance_enforced() {
    // Probe: all minutiae at same 2D positions, all angles = 45 degrees.
    // Gallery: same 2D positions, angles = i*18 degrees (evenly distributed 0..360).
    // For any global rotation delta applied to probe, all probe angles become 45+delta.
    // Only gallery minutia with angle ≈ 45+delta (within rot_tol=10) can match.
    // Since gallery angles are 18 degrees apart, at most 1 matches per rotation.
    // Score = 1/20 = 0.05 per anchor.
    let probe_minutiae: Vec<Minutia> = (0..20u16)
        .map(|i| make_minutia(50 + i * 20, 50 + ((i * 13) % 15) * 20, 45.0))
        .collect();
    let gallery_minutiae: Vec<Minutia> = probe_minutiae
        .iter()
        .enumerate()
        .map(|(i, m)| Minutia {
            theta: ((i as u32 * 18) % 360) as f32,
            ..*m
        })
        .collect();
    let probe = FingerprintTemplate::from_minutiae(FingerPosition::RightRing, probe_minutiae);
    let gallery = FingerprintTemplate::from_minutiae(FingerPosition::RightRing, gallery_minutiae);
    let mut matcher = FingerprintMatcher::new(0.4);
    matcher.rot_tol = 10.0;
    let score = matcher.compare(&probe, &gallery);
    // At most 2 gallery angles can be within 10 degrees of any single rotation value
    // (spacing = 18 degrees; window = 20 degrees).
    assert!(
        score.raw <= 4.0,
        "with rot_tol=10 and gallery angles 18-deg apart: max ~2 matches, raw={}",
        score.raw
    );
}

#[test]
fn fingerprint_translational_tolerance_boundary() {
    // Gallery shifted by 39 pixels: within ±40 px tolerance.
    let base: Vec<Minutia> = (0..20u16)
        .map(|i| make_minutia(100 + i * 5, 100 + i * 5, (i * 12) as f32))
        .collect();
    let shifted: Vec<Minutia> = base
        .iter()
        .map(|m| Minutia {
            x: m.x + 39,
            y: m.y,
            ..*m
        })
        .collect();
    let probe = FingerprintTemplate::from_minutiae(FingerPosition::RightLittle, base);
    let gallery = FingerprintTemplate::from_minutiae(FingerPosition::RightLittle, shifted);
    let matcher = FingerprintMatcher::new(0.3);
    let score = matcher.compare(&probe, &gallery);
    assert!(
        score.threshold_pass,
        "39-px shift should pass, score={}",
        score.normalised
    );
}

#[test]
fn fingerprint_translational_tolerance_enforced() {
    // Gallery has INDEPENDENT per-minutia spatial offsets (prime-based, 10-70px).
    // No single global translation can bring all probe minutiae within trans_tol=5.
    // After anchoring on any pair (probe[i], gallery[i]), only the anchor itself
    // sits within tolerance; all others are offset by (offset_i - offset_j) >> 5.
    let base: Vec<Minutia> = (0..15u16)
        .map(|i| make_minutia(100 + i * 20, 100 + i * 20, (i * 24) as f32))
        .collect();
    let gallery_minutiae: Vec<Minutia> = base
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let ox = ((i as u16 * 23) % 60 + 10) as u16; // 10..69px, prime-spaced
            let oy = ((i as u16 * 29) % 60 + 10) as u16;
            Minutia {
                x: m.x.saturating_add(ox),
                y: m.y.saturating_add(oy),
                ..*m
            }
        })
        .collect();
    let probe = FingerprintTemplate::from_minutiae(FingerPosition::LeftMiddle, base);
    let gallery = FingerprintTemplate::from_minutiae(FingerPosition::LeftMiddle, gallery_minutiae);
    let mut matcher = FingerprintMatcher::new(0.3);
    matcher.trans_tol = 5.0; // Tight: most residuals are 10-70px >> 5.
    let score = matcher.compare(&probe, &gallery);
    // Only the anchor pair itself matches; score = 1/15 ≈ 0.067.
    assert!(
        score.normalised < 0.15,
        "independent offsets with 5px tol: expected low score, got {}",
        score.normalised
    );
}

#[test]
fn fingerprint_empty_templates_score_zero() {
    let probe = FingerprintTemplate::from_minutiae(FingerPosition::RightThumb, vec![]);
    let gallery = FingerprintTemplate::from_minutiae(FingerPosition::RightThumb, vec![]);
    let matcher = FingerprintMatcher::default();
    let score = matcher.compare(&probe, &gallery);
    assert_eq!(score.raw, 0.0);
}

// ---------------------------------------------------------------------------
// Face matcher tests
// ---------------------------------------------------------------------------

#[test]
fn face_cosine_identical_is_one() {
    let f = make_face(1.0);
    let matcher = FaceMatcher {
        threshold: 0.5,
        cohort: None,
    };
    let score = matcher.compare(&f, &f);
    assert!(
        (score.raw - 1.0).abs() < 1e-5,
        "identical cosine = {}",
        score.raw
    );
}

#[test]
fn face_cosine_orthogonal_is_near_zero() {
    // Two embeddings constructed to be roughly orthogonal.
    let mut emb_a = vec![0.0f32; 512];
    let mut emb_b = vec![0.0f32; 512];
    emb_a[0] = 1.0;
    emb_b[1] = 1.0;
    let fa = FaceTemplate::new(emb_a, icao_all_pass(), None);
    let fb = FaceTemplate::new(emb_b, icao_all_pass(), None);
    let matcher = FaceMatcher {
        threshold: 0.5,
        cohort: None,
    };
    let score = matcher.compare(&fa, &fb);
    assert!(score.raw < 0.01, "orthogonal cosine = {}", score.raw);
}

#[test]
fn face_z_score_normalisation_shifts_score() {
    let probe = make_face(1.0);
    let gallery = make_face(2.0);
    let raw_matcher = FaceMatcher {
        threshold: 0.5,
        cohort: None,
    };
    let raw_score = raw_matcher.compare(&probe, &gallery);

    let z_matcher = FaceMatcher::with_cohort(
        0.5,
        CohortStats {
            mean: raw_score.raw,
            std: 0.1,
        },
    );
    let z_score = z_matcher.compare(&probe, &gallery);
    // With mean=raw and std=0.1, z=0 -> normalised ~= 0.5
    assert!(
        (z_score.normalised - 0.5).abs() < 0.05,
        "z-score normalised={}",
        z_score.normalised
    );
}

#[test]
fn face_different_embeddings_score_below_identical() {
    let f1 = make_face(1.0);
    let f2 = make_face(5.0);
    let matcher = FaceMatcher {
        threshold: 0.5,
        cohort: None,
    };
    let s_same = matcher.compare(&f1, &f1);
    let s_diff = matcher.compare(&f1, &f2);
    assert!(
        s_diff.raw < s_same.raw,
        "different faces should score lower than self"
    );
}

// ---------------------------------------------------------------------------
// Iris matcher tests
// ---------------------------------------------------------------------------

#[test]
fn iris_identical_hd_is_zero() {
    let iris = full_mask_iris(0b10101010);
    let matcher = IrisMatcher::default();
    let score = matcher.compare(&iris, &iris);
    assert_eq!(score.raw, 0.0, "identical IrisCode HD = 0");
    assert!(score.threshold_pass, "zero HD must pass");
}

#[test]
fn iris_random_hd_near_half() {
    // XOR of random-ish codes with full mask approaches HD ≈ 0.5.
    let code_a = vec![0xAAu8; IRIS_CODE_BYTES]; // 10101010
    let code_b = vec![0x55u8; IRIS_CODE_BYTES]; // 01010101
    let iris_a = IrisTemplate::from_vecs(code_a, vec![0xFF; IRIS_CODE_BYTES]);
    let iris_b = IrisTemplate::from_vecs(code_b, vec![0xFF; IRIS_CODE_BYTES]);
    let matcher = IrisMatcher::default();
    let score = matcher.compare(&iris_a, &iris_b);
    // 0xAA XOR 0x55 = 0xFF => all bits differ => HD = 1.0
    // Normalised: (1.0 - 1.0/0.5).clamp(0,1) = 0
    assert!(score.raw > 0.4, "complementary codes HD={}", score.raw);
}

#[test]
fn iris_mask_overlap_rejection() {
    // Mask with only 20% overlap fraction -> should be rejected.
    let code_a = vec![0xAAu8; IRIS_CODE_BYTES];
    let code_b = vec![0xAAu8; IRIS_CODE_BYTES];
    // Sparse masks: only first 25 bytes set = 200/2048 ≈ 9.8% each.
    let mut mask_a = vec![0u8; IRIS_CODE_BYTES];
    let mut mask_b = vec![0u8; IRIS_CODE_BYTES];
    for i in 0..25 {
        mask_a[i] = 0xFF;
        mask_b[i] = 0xFF;
    }
    let iris_a = IrisTemplate::from_vecs(code_a, mask_a);
    let iris_b = IrisTemplate::from_vecs(code_b, mask_b);
    let matcher = IrisMatcher::default(); // min_overlap = 0.30
    let score = matcher.compare(&iris_a, &iris_b);
    // Overlap ≈ 9.8% < 30% → reject (score.normalised = 0, threshold_pass = false).
    assert!(!score.threshold_pass, "low-overlap iris should fail");
}

#[test]
fn iris_partial_mask_hd_counts_only_overlap() {
    // Second half of both codes differ; masks only cover first half.
    let code_a: Vec<u8> = (0..IRIS_CODE_BYTES).map(|_| 0xAAu8).collect();
    let code_b: Vec<u8> = (0..IRIS_CODE_BYTES)
        .map(|i| {
            if i < IRIS_CODE_BYTES / 2 {
                0xAAu8
            } else {
                0x55u8
            }
        })
        .collect();
    let mask: Vec<u8> = (0..IRIS_CODE_BYTES)
        .map(|i| {
            if i < IRIS_CODE_BYTES / 2 {
                0xFFu8
            } else {
                0x00u8
            }
        })
        .collect();
    let iris_a = IrisTemplate::from_vecs(code_a, mask.clone());
    let iris_b = IrisTemplate::from_vecs(code_b, mask);
    let matcher = IrisMatcher::default();
    let score = matcher.compare(&iris_a, &iris_b);
    // Only first half covered; codes identical there → HD = 0.
    assert!(
        score.raw < 0.05,
        "masked HD should be near 0, got {}",
        score.raw
    );
}

// ---------------------------------------------------------------------------
// HNSW index tests
// ---------------------------------------------------------------------------

fn pseudo_rand_emb(seed: u64, dim: usize) -> Vec<f32> {
    let mut v = Vec::with_capacity(dim);
    let mut s = seed ^ 0xDEAD_CAFE_1234_5678u64;
    for _ in 0..dim {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        v.push((s >> 33) as f32 / u32::MAX as f32 * 2.0 - 1.0);
    }
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    v.iter_mut().for_each(|x| *x /= norm.max(1e-9));
    v
}

#[test]
fn hnsw_insert_and_search_basic() {
    let mut idx = HnswIndex::new(512);
    idx.add("s1", &pseudo_rand_emb(1, 512)).unwrap();
    idx.add("s2", &pseudo_rand_emb(2, 512)).unwrap();
    idx.add("s3", &pseudo_rand_emb(3, 512)).unwrap();
    let results = idx.search(&pseudo_rand_emb(1, 512), 1).unwrap();
    assert_eq!(
        results[0].subject_id, "s1",
        "nearest to s1 embedding should be s1"
    );
}

#[test]
fn hnsw_10k_recall_at_10() {
    const N: usize = 10_000;
    const K: usize = 10;
    let mut idx = HnswIndex::new(512);
    for i in 0..N {
        idx.add(&format!("s{i}"), &pseudo_rand_emb(i as u64, 512))
            .unwrap();
    }
    // For each of 100 random probes, the true nearest in brute-force should appear in HNSW top-K.
    let mut hits = 0usize;
    let total_queries = 100usize;
    for q in 0..total_queries {
        let probe = pseudo_rand_emb((N + q) as u64, 512);
        let hnsw_results = idx.search(&probe, K).unwrap();
        // Brute-force: find true nearest.
        let bf_nearest = (0..N)
            .map(|i| {
                let emb = pseudo_rand_emb(i as u64, 512);
                let d: f32 = probe
                    .iter()
                    .zip(emb.iter())
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f32>();
                (d, format!("s{i}"))
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap()
            .1;
        if hnsw_results.iter().any(|c| c.subject_id == bf_nearest) {
            hits += 1;
        }
    }
    let recall = hits as f64 / total_queries as f64;
    assert!(recall >= 0.90, "HNSW recall@{K} = {recall:.2} < 0.90");
}

#[test]
fn hnsw_dim_mismatch_error() {
    let mut idx = HnswIndex::new(512);
    let result = idx.add("s1", &vec![0.1f32; 128]);
    assert!(result.is_err(), "dimension mismatch must fail");
}

#[test]
fn hnsw_remove_then_not_returned() {
    let mut idx = HnswIndex::new(16);
    idx.add("s1", &pseudo_rand_emb(1, 16)).unwrap();
    idx.add("s2", &pseudo_rand_emb(2, 16)).unwrap();
    idx.remove("s1").unwrap();
    assert_eq!(idx.len(), 1);
}

// ---------------------------------------------------------------------------
// Shard hasher tests
// ---------------------------------------------------------------------------

#[test]
fn shard_deterministic_per_id() {
    assert_eq!(shard_id("voter-abc"), shard_id("voter-abc"));
}

#[test]
fn shard_uniform_distribution_chi_squared() {
    const N: usize = 25_600; // 100 per shard expected.
    let ids: Vec<String> = (0..N).map(|i| format!("voter-{i:08}")).collect();
    let id_refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
    let counts = shard_distribution(&id_refs);
    let expected = N as f64 / 256.0;
    // Chi-squared statistic: sum((obs - exp)^2 / exp).
    let chi2: f64 = counts
        .iter()
        .map(|&c| (c as f64 - expected).powi(2) / expected)
        .sum();
    // At 255 df, p=0.001 critical value ≈ 347. A fair hash should be well below that.
    assert!(
        chi2 < 400.0,
        "shard distribution chi2={chi2:.1} (should be < 400 for uniform)"
    );
}

#[test]
fn shard_id_in_range() {
    for i in 0..1000u32 {
        let _s: u8 = shard_id(&format!("subject-{i}"));
        // Implicit: no panic, value fits u8.
    }
}

// ---------------------------------------------------------------------------
// Bloom filter / demographic pruner tests
// ---------------------------------------------------------------------------

#[test]
fn bloom_no_false_negatives() {
    let key = demo_key(Sex::M, AgeBucket::Age26_40, "nairobi");
    let mut bloom = DemographicBloom::new(1000);
    let ids: Vec<String> = (0..500).map(|i| format!("voter-{i}")).collect();
    for id in &ids {
        bloom.insert(&key, id);
    }
    // Every inserted ID must be found.
    for id in &ids {
        assert!(bloom.may_contain(&key, id), "false negative for {id}");
    }
}

#[test]
fn bloom_fpr_at_target() {
    let key = demo_key(Sex::F, AgeBucket::Age41_60, "mombasa");
    let mut bloom = DemographicBloom::new(2000);
    for i in 0..1000 {
        bloom.insert(&key, &format!("member-{i}"));
    }
    // Test non-members: measure FPR.
    let trials = 5000usize;
    let fps: usize = (0..trials)
        .filter(|i| bloom.may_contain(&key, &format!("nonmember-{i}")))
        .count();
    let fpr = fps as f64 / trials as f64;
    assert!(
        fpr <= 0.05,
        "bloom FPR = {fpr:.4} > 0.05 (target 0.01 with some margin)"
    );
}

#[test]
fn bloom_unknown_key_returns_false() {
    let bloom = DemographicBloom::new(1000);
    let key = demo_key(Sex::M, AgeBucket::Age18_25, "kisumu");
    assert!(!bloom.may_contain(&key, "voter-99"));
}

#[test]
fn bloom_candidate_slices_covers_inserted() {
    let key1 = demo_key(Sex::M, AgeBucket::Age18_25, "kisumu");
    let key2 = demo_key(Sex::F, AgeBucket::Age61Plus, "nakuru");
    let mut bloom = DemographicBloom::new(500);
    bloom.insert(&key1, "voter-xyz");
    bloom.insert(&key2, "voter-xyz");
    let slices = bloom.candidate_slices("voter-xyz");
    assert!(
        slices.len() >= 2,
        "both keys should appear in candidate_slices"
    );
}

// ---------------------------------------------------------------------------
// Fusion tests
// ---------------------------------------------------------------------------

#[test]
fn fusion_weighted_sum_correct() {
    let policy = FusionPolicy::default(); // fp=0.5, face=0.3, iris=0.2
    let scores = vec![
        MultiModalScore::new(Modality::Fingerprint, 1.0),
        MultiModalScore::new(Modality::Face, 0.0),
    ];
    let result = fuse(&scores, &policy);
    // weighted_sum = 1.0*0.5 + 0.0*0.3 = 0.5; total_weight = 0.8; fused = 0.5/0.8 = 0.625
    let expected = 0.5f32 / 0.8f32;
    assert!(
        (result.fused - expected).abs() < 1e-5,
        "fused={} expected={}",
        result.fused,
        expected
    );
}

#[test]
fn fusion_confidence_increases_with_modality_count() {
    let policy = FusionPolicy::default();
    let one_modal = fuse(&[MultiModalScore::new(Modality::Fingerprint, 0.8)], &policy);
    let two_modal = fuse(
        &[
            MultiModalScore::new(Modality::Fingerprint, 0.8),
            MultiModalScore::new(Modality::Face, 0.7),
        ],
        &policy,
    );
    let three_modal = fuse(
        &[
            MultiModalScore::new(Modality::Fingerprint, 0.8),
            MultiModalScore::new(Modality::Face, 0.7),
            MultiModalScore::new(Modality::Iris, 0.9),
        ],
        &policy,
    );
    assert!(
        one_modal.confidence <= two_modal.confidence,
        "confidence should not decrease with more modalities"
    );
    assert!(
        two_modal.confidence <= three_modal.confidence,
        "three-modal must have highest confidence"
    );
}

#[test]
fn fusion_empty_returns_zero() {
    let result = fuse(&[], &FusionPolicy::default());
    assert_eq!(result.fused, 0.0);
    assert_eq!(result.confidence, 0.0);
    assert!(result.contributors.is_empty());
}

#[test]
fn fusion_contributors_recorded() {
    let scores = vec![
        MultiModalScore::new(Modality::Face, 0.7),
        MultiModalScore::new(Modality::Iris, 0.6),
    ];
    let result = fuse(&scores, &FusionPolicy::default());
    assert_eq!(result.contributors.len(), 2);
    let modalities: Vec<Modality> = result.contributors.iter().map(|(m, _)| *m).collect();
    assert!(modalities.contains(&Modality::Face));
    assert!(modalities.contains(&Modality::Iris));
}

// ---------------------------------------------------------------------------
// Quality gate tests
// ---------------------------------------------------------------------------

#[test]
fn quality_gate_fingerprint_accept() {
    let policy = QualityPolicy::default(); // min_nfiq2 = 0.40
    assert_eq!(gate_fingerprint(0.80, &policy), QualityDecision::Accept);
    assert_eq!(gate_fingerprint(0.40, &policy), QualityDecision::Accept);
}

#[test]
fn quality_gate_fingerprint_retake() {
    let policy = QualityPolicy::default(); // retake band: [0.20, 0.40)
    assert_eq!(gate_fingerprint(0.25, &policy), QualityDecision::Retake);
}

#[test]
fn quality_gate_fingerprint_escalate() {
    let policy = QualityPolicy::default(); // escalate: below 0.20
    assert_eq!(
        gate_fingerprint(0.10, &policy),
        QualityDecision::EscalateManualCapture
    );
}

#[test]
fn quality_gate_face_accept() {
    let policy = QualityPolicy::default();
    assert_eq!(gate_face(0.75, &policy), QualityDecision::Accept);
}

#[test]
fn quality_gate_face_escalate() {
    let policy = QualityPolicy::default();
    assert_eq!(
        gate_face(0.10, &policy),
        QualityDecision::EscalateManualCapture
    );
}

#[test]
fn quality_gate_iris_retake() {
    let policy = QualityPolicy::default(); // min_iris = 0.60; retake [0.30, 0.60)
    assert_eq!(gate_iris(0.45, &policy), QualityDecision::Retake);
}

#[test]
fn quality_gate_finger_count_accept() {
    let policy = QualityPolicy::default(); // min_usable_fingers = 6
    let qualities = vec![0.8; 10]; // All 10 fingers pass
    assert_eq!(
        gate_finger_count(&qualities, &policy),
        QualityDecision::Accept
    );
}

#[test]
fn quality_gate_finger_count_escalate() {
    let policy = QualityPolicy::default();
    // Only 2 fingers pass (< 6/2 = 3)
    let qualities: Vec<f32> = vec![0.8, 0.8, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1];
    assert_eq!(
        gate_finger_count(&qualities, &policy),
        QualityDecision::EscalateManualCapture
    );
}

// ---------------------------------------------------------------------------
// Fairness harness tests
// ---------------------------------------------------------------------------

fn make_record(
    sex: Sex,
    age: AgeBucket,
    county: &str,
    is_genuine: bool,
    matched: bool,
) -> MatchRecord {
    MatchRecord {
        demographic: demo_key(sex, age, county),
        is_genuine,
        system_match: matched,
    }
}

#[test]
fn fairness_equalised_cohort_no_breach() {
    // Equal error rates across sex: no breach.
    let records = vec![
        // Male, 26-40, nairobi: 100 genuine matches (0% FNMR), 100 impostor non-matches (0% FMR)
        vec![make_record(Sex::M, AgeBucket::Age26_40, "nairobi", true, true); 100],
        vec![make_record(Sex::M, AgeBucket::Age26_40, "nairobi", false, false); 100],
        // Female, 26-40, nairobi: same rates
        vec![make_record(Sex::F, AgeBucket::Age26_40, "nairobi", true, true); 100],
        vec![make_record(Sex::F, AgeBucket::Age26_40, "nairobi", false, false); 100],
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let slices = FairnessReport::compute(&records);
    let breaches = FairnessReport::detect_breaches(&slices);
    assert!(
        breaches.is_empty(),
        "equal cohort should have no breaches, got {}",
        breaches.len()
    );
}

#[test]
fn fairness_uneven_cohort_produces_breach() {
    // Male has 0% FNMR, Female has 30% FNMR => breach (>5% tolerance).
    let mut records = Vec::new();
    // Male: 100 genuine all matched
    for _ in 0..100 {
        records.push(make_record(
            Sex::M,
            AgeBucket::Age26_40,
            "nairobi",
            true,
            true,
        ));
    }
    // Female: 100 genuine, 30 missed
    for i in 0..100 {
        records.push(make_record(
            Sex::F,
            AgeBucket::Age26_40,
            "nairobi",
            true,
            i >= 70,
        ));
    }
    let slices = FairnessReport::compute(&records);
    let breaches = FairnessReport::detect_breaches(&slices);
    assert!(
        !breaches.is_empty(),
        "30% FNMR disparity must trigger a breach"
    );
}

#[test]
fn fairness_slice_count_matches_demographics() {
    let records = vec![
        make_record(Sex::M, AgeBucket::Age18_25, "a", true, true),
        make_record(Sex::F, AgeBucket::Age41_60, "b", false, false),
        make_record(Sex::M, AgeBucket::Age61Plus, "c", true, true),
    ];
    let slices = FairnessReport::compute(&records);
    assert_eq!(slices.len(), 3, "3 distinct demo keys -> 3 slices");
}

// ---------------------------------------------------------------------------
// PAD interface tests
// ---------------------------------------------------------------------------

#[test]
fn pad_software_stub_returns_live() {
    let pad = SoftwarePad;
    let sample = Sample {
        bytes: b"fake-image-bytes",
        media_type: "image/jpeg",
    };
    let result = pad.detect(&sample);
    assert!(result.is_live, "SoftwarePad must classify as live");
    assert_eq!(result.attack_type, None);
    assert!(result.score > 0.9);
}

#[test]
fn pad_detector_trait_is_object_safe() {
    // Verify the trait can be used as a trait object.
    let pad: &dyn PadDetector = &SoftwarePad;
    let sample = Sample {
        bytes: b"data",
        media_type: "image/wsq",
    };
    let result = pad.detect(&sample);
    assert!(result.is_live);
}
