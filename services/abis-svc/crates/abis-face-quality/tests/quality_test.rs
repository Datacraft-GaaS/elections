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

use eel_abis_face_quality::{
    expression_neutrality, gaze_offset, illumination,
    metrics::{pose_angles, pose_quality, PoseAngles},
    occlusion_coverage,
    policy::{make_report, PolicyViolation, QualityPolicy},
    resolution_interocular, sharpness, uniformity, QualityError, QualityLandmarks,
};

fn frontal_lm() -> QualityLandmarks {
    QualityLandmarks {
        left_eye: (38.0, 51.0),
        right_eye: (74.0, 51.0),
        nose: (56.0, 72.0),
        left_mouth: (42.0, 92.0),
        right_mouth: (70.0, 92.0),
        left_pupil: Some((38.0, 51.0)),
        right_pupil: Some((74.0, 51.0)),
    }
}

fn sharp_gray(w: u32, h: u32) -> Vec<u8> {
    // Alternating black-white checkerboard => high Laplacian variance.
    let mut v = vec![0u8; (w * h) as usize];
    for y in 0..h as usize {
        for x in 0..w as usize {
            v[y * w as usize + x] = if (x + y) % 2 == 0 { 255 } else { 0 };
        }
    }
    v
}

fn blurry_gray(w: u32, h: u32, val: u8) -> Vec<u8> {
    vec![val; (w * h) as usize]
}

fn skin_rgb(w: u32, h: u32) -> Vec<u8> {
    // Typical skin: R=180, G=140, B=100 (satisfies heuristic).
    let mut v = vec![0u8; (w * h * 3) as usize];
    for chunk in v.chunks_mut(3) {
        chunk[0] = 180;
        chunk[1] = 140;
        chunk[2] = 100;
    }
    v
}

// --- Sharpness tests ---

#[test]
fn sharpness_checkerboard_high() {
    let g = sharp_gray(64, 64);
    let s = sharpness(&g, 64, 64).unwrap();
    assert!(s > 0.3, "sharpness={}", s);
}

#[test]
fn sharpness_uniform_low() {
    let g = blurry_gray(64, 64, 128);
    let s = sharpness(&g, 64, 64).unwrap();
    assert!(s < 0.05, "sharpness={}", s);
}

#[test]
fn sharpness_bounded_0_1() {
    let g = sharp_gray(64, 64);
    let s = sharpness(&g, 64, 64).unwrap();
    assert!((0.0..=1.0).contains(&s));
}

#[test]
fn sharpness_too_small_errors() {
    assert!(matches!(
        sharpness(&[], 10, 10),
        Err(QualityError::ImageTooSmall)
    ));
}

// --- Resolution tests ---

#[test]
fn resolution_90px_iod_scores_one() {
    let mut lm = frontal_lm();
    lm.left_eye = (0.0, 0.0);
    lm.right_eye = (90.0, 0.0);
    let s = resolution_interocular(&lm).unwrap();
    assert!((s - 1.0).abs() < 1e-4, "s={}", s);
}

#[test]
fn resolution_45px_iod_scores_half() {
    let mut lm = frontal_lm();
    lm.left_eye = (0.0, 0.0);
    lm.right_eye = (45.0, 0.0);
    let s = resolution_interocular(&lm).unwrap();
    assert!((s - 0.5).abs() < 0.01, "s={}", s);
}

#[test]
fn resolution_zero_iod_errors() {
    let mut lm = frontal_lm();
    lm.right_eye = lm.left_eye;
    assert!(matches!(
        resolution_interocular(&lm),
        Err(QualityError::ZeroInterocularDistance)
    ));
}

#[test]
fn resolution_bounded_0_1() {
    let lm = frontal_lm();
    let s = resolution_interocular(&lm).unwrap();
    assert!((0.0..=1.0).contains(&s));
}

// --- Pose tests ---

#[test]
fn pose_frontal_low_angles() {
    let lm = frontal_lm();
    let p = pose_angles(&lm);
    assert!(p.roll.abs() < 5.0, "roll={}", p.roll);
}

#[test]
fn pose_quality_frontal_high() {
    let angles = PoseAngles {
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
    };
    let q = pose_quality(&angles);
    assert!((q - 1.0).abs() < 0.01);
}

#[test]
fn pose_quality_large_yaw_low() {
    let angles = PoseAngles {
        yaw: 45.0,
        pitch: 0.0,
        roll: 0.0,
    };
    let q = pose_quality(&angles);
    assert!(q < 0.1, "q={}", q);
}

#[test]
fn pose_quality_bounded_0_1() {
    let angles = PoseAngles {
        yaw: 30.0,
        pitch: 20.0,
        roll: 10.0,
    };
    let q = pose_quality(&angles);
    assert!((0.0..=1.0).contains(&q));
}

// --- Illumination tests ---

#[test]
fn illumination_mid_range_high() {
    let g: Vec<u8> = (0..112 * 112).map(|i| (80 + (i % 80) as u8)).collect();
    let s = illumination(&g);
    assert!(s > 0.5, "s={}", s);
}

#[test]
fn illumination_all_black_low() {
    let g = vec![0u8; 112 * 112];
    let s = illumination(&g);
    assert!(s < 0.5, "s={}", s);
}

#[test]
fn illumination_bounded_0_1() {
    let g = blurry_gray(64, 64, 200);
    let s = illumination(&g);
    assert!((0.0..=1.0).contains(&s));
}

// --- Uniformity tests ---

#[test]
fn uniformity_uniform_background_high() {
    let g = blurry_gray(64, 64, 200);
    let s = uniformity(&g, 64, 64);
    assert!(s > 0.8, "s={}", s);
}

#[test]
fn uniformity_noisy_background_lower() {
    let mut g = blurry_gray(64, 64, 128);
    // Add noise to border pixels.
    for i in (0..64).step_by(2) {
        g[i] = 0;
        g[i + 1] = 255;
    }
    let s = uniformity(&g, 64, 64);
    assert!(s < 0.9);
}

#[test]
fn uniformity_bounded_0_1() {
    let g = sharp_gray(64, 64);
    let s = uniformity(&g, 64, 64);
    assert!((0.0..=1.0).contains(&s));
}

// --- Occlusion tests ---

#[test]
fn occlusion_skin_image_scores_high() {
    let rgb = skin_rgb(112, 112);
    let s = occlusion_coverage(&rgb, 112, 112);
    assert!(s > 0.5, "s={}", s);
}

#[test]
fn occlusion_dark_image_scores_low() {
    let rgb = vec![20u8; 112 * 112 * 3];
    let s = occlusion_coverage(&rgb, 112, 112);
    assert!(s < 0.3, "s={}", s);
}

#[test]
fn occlusion_bounded_0_1() {
    let rgb = skin_rgb(64, 64);
    let s = occlusion_coverage(&rgb, 64, 64);
    assert!((0.0..=1.0).contains(&s));
}

// --- Expression tests ---

#[test]
fn expression_frontal_lm_neutral() {
    let lm = frontal_lm();
    let s = expression_neutrality(&lm);
    assert!(s > 0.0 && s <= 1.0, "s={}", s);
}

#[test]
fn expression_bounded_0_1() {
    let lm = frontal_lm();
    let s = expression_neutrality(&lm);
    assert!((0.0..=1.0).contains(&s));
}

// --- Gaze tests ---

#[test]
fn gaze_pupil_at_corner_perfect() {
    let lm = frontal_lm(); // pupils = eye corners => 0 offset => score 1.0
    let s = gaze_offset(&lm);
    assert!((s - 1.0).abs() < 0.01, "s={}", s);
}

#[test]
fn gaze_no_pupil_data_returns_one() {
    let mut lm = frontal_lm();
    lm.left_pupil = None;
    lm.right_pupil = None;
    let s = gaze_offset(&lm);
    assert!((s - 1.0).abs() < 0.01);
}

#[test]
fn gaze_bounded_0_1() {
    let lm = frontal_lm();
    let s = gaze_offset(&lm);
    assert!((0.0..=1.0).contains(&s));
}

// --- QualityPolicy tests ---

fn good_report() -> (eel_abis_face_quality::QualityReport, f32) {
    let lm = frontal_lm();
    let angles = pose_angles(&lm);
    // Pass 90px explicitly so the report passes the 60px min_iod policy gate.
    let iod = 90.0f32;
    let r = make_report(0.8, 0.9, angles, 0.8, 0.9, 0.05, 0.8, 0.9);
    (r, iod)
}

#[test]
fn policy_default_accepts_good_report() {
    let (r, iod) = good_report();
    let policy = QualityPolicy::default();
    assert!(
        policy.accept(&r, iod),
        "violations: {:?}",
        policy.evaluate(&r, iod)
    );
}

#[test]
fn policy_rejects_low_sharpness() {
    let (mut r, iod) = good_report();
    r.sharpness = 0.01;
    r.overall = 0.01;
    let policy = QualityPolicy::default();
    let v = policy.evaluate(&r, iod);
    assert!(v
        .iter()
        .any(|v| matches!(v, PolicyViolation::SharpnessTooLow { .. })));
}

#[test]
fn policy_rejects_low_iod() {
    let (r, _) = good_report();
    let policy = QualityPolicy::default();
    let v = policy.evaluate(&r, 20.0);
    assert!(v
        .iter()
        .any(|v| matches!(v, PolicyViolation::ResolutionTooLow { .. })));
}

#[test]
fn policy_rejects_large_yaw() {
    let angles = PoseAngles {
        yaw: 45.0,
        pitch: 0.0,
        roll: 0.0,
    };
    let r = make_report(0.8, 0.9, angles, 0.8, 0.9, 0.05, 0.8, 0.9);
    let lm = frontal_lm();
    let policy = QualityPolicy::default();
    let v = policy.evaluate(&r, lm.interocular_distance());
    assert!(v
        .iter()
        .any(|v| matches!(v, PolicyViolation::PoseYawTooLarge { .. })));
}

#[test]
fn policy_rejects_high_occlusion() {
    let (mut r, iod) = good_report();
    r.occlusion = 0.99; // score is skin-tone fraction; high fraction is good...
                        // policy.max_occlusion compares report.occlusion to max_occlusion=0.15.
                        // We need occlusion > 0.15 to trigger rejection.
    let policy = QualityPolicy::default();
    let v = policy.evaluate(&r, iod);
    assert!(v
        .iter()
        .any(|v| matches!(v, PolicyViolation::OcclusionTooHigh { .. })));
}

#[test]
fn policy_composition_strict_accepts_nothing() {
    let (r, iod) = good_report();
    let strict = QualityPolicy {
        min_sharpness: 1.1, // impossible
        ..QualityPolicy::default()
    };
    assert!(!strict.accept(&r, iod));
}
