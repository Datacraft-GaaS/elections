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

use chrono::Utc;
use eel_abis_modality_periocular::{
    matcher::{chi_square, match_periocular, DEFAULT_THRESHOLD},
    parser::{compute_lbp_histogram, parse_periocular, serialize_periocular, PeriocularError},
    quality::{laplacian_variance, score_quality},
    LbpHistogram, PeriocularTemplate, PATCH_SIZE,
};

fn make_template(patch: Vec<u8>, occlusion: f32) -> PeriocularTemplate {
    assert_eq!(patch.len(), PATCH_SIZE * PATCH_SIZE);
    let lbp = compute_lbp_histogram(&patch, PATCH_SIZE, PATCH_SIZE);
    let sharpness = laplacian_variance(&patch, PATCH_SIZE, PATCH_SIZE);
    PeriocularTemplate {
        template_id: "test".to_string(),
        patch,
        lbp_histogram: lbp,
        sharpness,
        occlusion_ratio: occlusion,
        captured_at: Utc::now(),
    }
}

fn uniform_patch(val: u8) -> Vec<u8> {
    vec![val; PATCH_SIZE * PATCH_SIZE]
}

fn checkerboard_patch() -> Vec<u8> {
    (0..PATCH_SIZE * PATCH_SIZE)
        .map(|i| {
            let x = i % PATCH_SIZE;
            let y = i / PATCH_SIZE;
            if (x + y) % 2 == 0 {
                255
            } else {
                0
            }
        })
        .collect()
}

// Test 1: compute_lbp_histogram returns 256 bins
#[test]
fn test_lbp_returns_256_bins() {
    let patch = uniform_patch(100);
    let hist = compute_lbp_histogram(&patch, PATCH_SIZE, PATCH_SIZE);
    assert_eq!(hist.bins.len(), 256);
}

// Test 2: lbp_histogram sums to 1.0 (normalized)
#[test]
fn test_lbp_sums_to_one() {
    let patch = checkerboard_patch();
    let hist = compute_lbp_histogram(&patch, PATCH_SIZE, PATCH_SIZE);
    let sum: f32 = hist.bins.iter().sum();
    assert!(
        (sum - 1.0).abs() < 1e-4,
        "LBP histogram sum should be 1.0, got {sum}"
    );
}

// Test 3: parse round-trip
#[test]
fn test_parse_round_trip() {
    let tmpl = make_template(checkerboard_patch(), 0.1);
    let bytes = serialize_periocular(&tmpl);
    let parsed = parse_periocular(&bytes).expect("should parse");
    assert_eq!(parsed.patch, tmpl.patch);
    assert!((parsed.sharpness - tmpl.sharpness).abs() < 1e-3);
    assert!((parsed.occlusion_ratio - tmpl.occlusion_ratio).abs() < 1e-5);
    for (a, b) in parsed
        .lbp_histogram
        .bins
        .iter()
        .zip(tmpl.lbp_histogram.bins.iter())
    {
        assert!((a - b).abs() < 1e-5);
    }
}

// Test 4: parse rejects wrong magic
#[test]
fn test_parse_wrong_magic() {
    let tmpl = make_template(uniform_patch(128), 0.0);
    let mut bytes = serialize_periocular(&tmpl);
    bytes[0] = 0xFF;
    assert!(matches!(
        parse_periocular(&bytes),
        Err(PeriocularError::InvalidMagic)
    ));
}

// Test 5: parse rejects wrong length
#[test]
fn test_parse_wrong_length() {
    let short = vec![0x50, 0x45, 0x52, 0x49, 0x00];
    assert!(matches!(
        parse_periocular(&short),
        Err(PeriocularError::BufferTooShort { .. })
    ));
}

// Test 6: chi_square of identical histograms = 0.0
#[test]
fn test_chi_square_identical() {
    let bins: Vec<f32> = (0..256).map(|i| i as f32 / 255.0).collect();
    let dist = chi_square(&bins, &bins);
    assert!(
        dist.abs() < 1e-5,
        "chi_square of identical histograms should be 0, got {dist}"
    );
}

// Test 7: chi_square of different histograms > 0
#[test]
fn test_chi_square_different() {
    let mut h1 = vec![0.0f32; 256];
    let mut h2 = vec![0.0f32; 256];
    h1[0] = 1.0;
    h2[255] = 1.0;
    let dist = chi_square(&h1, &h2);
    assert!(
        dist > 0.0,
        "different histograms should have positive chi_square"
    );
}

// Test 8: match identical template > DEFAULT_THRESHOLD
#[test]
fn test_match_identical_above_threshold() {
    let tmpl = make_template(checkerboard_patch(), 0.0);
    let score = match_periocular(&tmpl, &tmpl);
    assert!(
        score.raw >= DEFAULT_THRESHOLD,
        "identical templates score {} should exceed threshold {}",
        score.raw,
        DEFAULT_THRESHOLD
    );
    assert!(score.threshold_pass);
}

// Test 9: match completely different histograms < DEFAULT_THRESHOLD
#[test]
fn test_match_different_below_threshold() {
    let mut h1_bins = vec![0.0f32; 256];
    let mut h2_bins = vec![0.0f32; 256];
    h1_bins[0] = 1.0;
    h2_bins[255] = 1.0;

    let patch = uniform_patch(128);
    let make = |bins: Vec<f32>| PeriocularTemplate {
        template_id: "t".to_string(),
        patch: patch.clone(),
        lbp_histogram: LbpHistogram { bins },
        sharpness: 0.0,
        occlusion_ratio: 0.0,
        captured_at: Utc::now(),
    };

    let probe = make(h1_bins);
    let gallery = make(h2_bins);
    let score = match_periocular(&probe, &gallery);
    assert!(
        score.raw < DEFAULT_THRESHOLD,
        "different histograms score {} should be below threshold {}",
        score.raw,
        DEFAULT_THRESHOLD
    );
}

// Test 10: laplacian_variance of uniform patch = 0.0
#[test]
fn test_laplacian_uniform_zero() {
    let patch = uniform_patch(128);
    let var = laplacian_variance(&patch, PATCH_SIZE, PATCH_SIZE);
    assert!(
        var.abs() < 1e-5,
        "uniform patch laplacian variance should be 0, got {var}"
    );
}

// Test 11: laplacian_variance of checkerboard > 0.0
#[test]
fn test_laplacian_checkerboard_positive() {
    let patch = checkerboard_patch();
    let var = laplacian_variance(&patch, PATCH_SIZE, PATCH_SIZE);
    assert!(
        var > 0.0,
        "checkerboard laplacian variance should be positive, got {var}"
    );
}

// Test 12: quality occlusion_ratio=0.0 gives occlusion_score=100.0
#[test]
fn test_quality_no_occlusion_score_100() {
    let tmpl = make_template(uniform_patch(128), 0.0);
    let q = score_quality(&tmpl);
    assert!((q.occlusion_score - 100.0).abs() < 1e-3);
}

// Test 13: quality occlusion_ratio=1.0 gives occlusion_score=0.0
#[test]
fn test_quality_full_occlusion_score_zero() {
    let tmpl = make_template(uniform_patch(128), 1.0);
    let q = score_quality(&tmpl);
    assert!(q.occlusion_score.abs() < 1e-3);
}

// Test 14: quality overall in [0, 100]
#[test]
fn test_quality_overall_in_range() {
    let tmpl = make_template(checkerboard_patch(), 0.3);
    let q = score_quality(&tmpl);
    assert!(
        q.overall >= 0.0 && q.overall <= 100.0,
        "overall {} out of range",
        q.overall
    );
}
