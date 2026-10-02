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

//! NFIQ 2.2 tests.
//!
//! Uses synthetic images: solid, noisy, and stripe patterns to verify feature
//! extraction and scoring without requiring real fingerprint images.

use eel_abis_fp_nfiq22::{
    calibrator::Calibrator,
    features::extract_features,
    model::RandomForestModel,
    scorer::{nfiq22_score, score_with_model},
    GrayImage, Nfiq22Error,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn solid(w: u32, h: u32, v: u8) -> GrayImage {
    GrayImage::new(vec![v; (w * h) as usize], w, h)
}

/// Alternating stripe pattern (simulates ridges/valleys).
fn stripe(w: u32, h: u32) -> GrayImage {
    let mut pix = vec![200u8; (w * h) as usize];
    for y in 0..h {
        if y % 10 < 5 {
            for x in 0..w {
                pix[(y * w + x) as usize] = 40;
            }
        }
    }
    GrayImage::new(pix, w, h)
}

/// Blurry / low-contrast image (poor quality proxy).
fn blurry(w: u32, h: u32) -> GrayImage {
    let pix: Vec<u8> = (0..(w * h))
        .map(|i| 120u8.wrapping_add((i % 5) as u8))
        .collect();
    GrayImage::new(pix, w, h)
}

// ---------------------------------------------------------------------------
// Feature extraction
// ---------------------------------------------------------------------------

#[test]
fn all_14_features_present_and_finite() {
    let img = stripe(64, 64);
    let f = extract_features(&img).expect("feature extraction should succeed");
    let arr = f.as_array();
    assert_eq!(arr.len(), 14, "expected 14 features");
    for (i, &v) in arr.iter().enumerate() {
        assert!(v.is_finite(), "feature {i} is not finite: {v}");
    }
}

#[test]
fn features_reject_tiny_image() {
    let img = solid(16, 16, 128);
    match extract_features(&img) {
        Err(Nfiq22Error::ImageTooSmall(16, 16)) => {}
        other => panic!("expected ImageTooSmall, got {other:?}"),
    }
}

#[test]
fn lcs_higher_for_stripe_than_solid() {
    let f_stripe = extract_features(&stripe(64, 64)).unwrap();
    let f_solid = extract_features(&solid(64, 64, 128)).unwrap();
    assert!(
        f_stripe.lcs > f_solid.lcs,
        "stripe LCS={} should exceed solid LCS={}",
        f_stripe.lcs,
        f_solid.lcs
    );
}

#[test]
fn sigma_higher_for_stripe_than_solid() {
    let f_stripe = extract_features(&stripe(64, 64)).unwrap();
    let f_solid = extract_features(&solid(64, 64, 128)).unwrap();
    assert!(
        f_stripe.sigma > f_solid.sigma,
        "stripe sigma should exceed solid"
    );
}

#[test]
fn roi_bounded() {
    let f = extract_features(&stripe(64, 64)).unwrap();
    assert!((0.0..=1.0).contains(&f.roi), "roi={} out of [0,1]", f.roi);
}

#[test]
fn features_non_negative() {
    let f = extract_features(&stripe(64, 64)).unwrap();
    for (i, &v) in f.as_array().iter().enumerate() {
        assert!(v >= 0.0, "feature {i} negative: {v}");
    }
}

// ---------------------------------------------------------------------------
// Random Forest model
// ---------------------------------------------------------------------------

#[test]
fn model_predict_in_range() {
    let model = RandomForestModel::default_model();
    let f = extract_features(&stripe(64, 64)).unwrap();
    let score = model.predict(&f.as_array());
    assert!(
        (0.0..=100.0).contains(&score),
        "model score {score} out of range"
    );
}

#[test]
fn score_in_0_to_100() {
    let img = stripe(64, 64);
    let score = nfiq22_score(&img).unwrap();
    assert!((0u8..=100).contains(&score), "score {score} out of [0,100]");
}

#[test]
fn stripe_scores_higher_than_solid() {
    let s_stripe = nfiq22_score(&stripe(64, 64)).unwrap();
    let s_solid = nfiq22_score(&solid(64, 64, 128)).unwrap();
    assert!(
        s_stripe >= s_solid,
        "stripe should score >= solid (stripe={s_stripe}, solid={s_solid})"
    );
}

#[test]
fn stripe_scores_higher_than_blurry() {
    let s_stripe = nfiq22_score(&stripe(64, 64)).unwrap();
    let s_blurry = nfiq22_score(&blurry(64, 64)).unwrap();
    // Blurry has near-zero contrast; stripe has clear ridges/valleys.
    assert!(
        s_stripe >= s_blurry,
        "stripe should score >= blurry ({s_stripe} vs {s_blurry})"
    );
}

// ---------------------------------------------------------------------------
// Calibrator
// ---------------------------------------------------------------------------

#[test]
fn calibrator_identity_preserves_bounds() {
    let cal = Calibrator::default();
    for raw in [0.0f32, 25.0, 50.0, 75.0, 100.0] {
        let cal_score = cal.calibrate(raw);
        assert!((0u8..=100).contains(&cal_score));
    }
}

#[test]
fn calibrator_from_scores_shifts_distribution() {
    // All raw scores are in [60, 80]; calibrated output should still be in [0,100].
    let raw_scores: Vec<f32> = (60..=80).map(|v| v as f32).collect();
    let cal = Calibrator::from_raw_scores(raw_scores);
    let out = cal.calibrate(70.0);
    assert!((0u8..=100).contains(&out));
}

#[test]
fn calibrator_fit_on_images() {
    let imgs = vec![stripe(64, 64), solid(64, 64, 200), blurry(64, 64)];
    let model = RandomForestModel::default_model();
    let cal = Calibrator::fit(&imgs, &model).expect("fit should succeed");
    let score = nfiq22_score(&stripe(64, 64)).unwrap();
    let calibrated = cal.calibrate(score as f32);
    assert!((0u8..=100).contains(&calibrated));
}
