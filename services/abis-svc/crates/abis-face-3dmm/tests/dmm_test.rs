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

use eel_abis_face_3dmm::{
    bfm::{Camera3d, DmmError, Fit3dmm, Landmark, Model3dmm, Pose3d, StubModel3dmm},
    spoof::{SpoofDecision, SpoofDetector},
};

fn default_landmarks() -> Vec<Landmark> {
    vec![
        Landmark::new(38.0, 51.0), // left eye
        Landmark::new(74.0, 51.0), // right eye
        Landmark::new(56.0, 72.0), // nose
        Landmark::new(42.0, 92.0), // left mouth
        Landmark::new(70.0, 92.0), // right mouth
    ]
}

// --- Fit3dmm construction ---

#[test]
fn fit_identity_correct_dimensions() {
    let fit = Fit3dmm::identity();
    assert_eq!(fit.shape_coeffs.len(), 85);
    assert_eq!(fit.expr_coeffs.len(), 64);
}

#[test]
fn fit_to_frontal_zeros_yaw_pitch() {
    let fit = Fit3dmm {
        shape_coeffs: vec![0.0; 85],
        expr_coeffs: vec![0.0; 64],
        pose: Pose3d {
            pitch: 15.0,
            yaw: 20.0,
            roll: 5.0,
        },
        camera: Camera3d::default(),
    };
    let frontal = fit.to_frontal();
    assert!(frontal.pose.yaw.abs() < 1e-5);
    assert!(frontal.pose.pitch.abs() < 1e-5);
    assert!((frontal.pose.roll - 5.0).abs() < 1e-5);
}

// --- StubModel3dmm ---

#[test]
fn stub_fit_frontal_landmarks() {
    let model = StubModel3dmm::new();
    let lm = default_landmarks();
    let img = vec![128u8; 112 * 112 * 3];
    let fit = model.fit(&img, 112, 112, &lm).unwrap();
    assert_eq!(fit.shape_coeffs.len(), 85);
    assert_eq!(fit.expr_coeffs.len(), 64);
}

#[test]
fn stub_fit_too_few_landmarks_errors() {
    let model = StubModel3dmm::new();
    let err = model
        .fit(&[], 112, 112, &[Landmark::new(0.0, 0.0)])
        .unwrap_err();
    assert!(matches!(err, DmmError::TooFewLandmarks(_)));
}

#[test]
fn stub_render_returns_correct_size() {
    let model = StubModel3dmm::new();
    let fit = Fit3dmm::identity();
    let out = model.render(&fit, &Pose3d::default(), 64, 64);
    assert_eq!(out.len(), (64 * 64 * 3) as usize);
}

#[test]
fn stub_pose_roundtrip_yaw_direction() {
    let model = StubModel3dmm::new();
    // Shift landmarks right => positive yaw estimate.
    let shifted: Vec<Landmark> = default_landmarks()
        .iter()
        .map(|l| Landmark::new(l.x + 20.0, l.y))
        .collect();
    let img = vec![128u8; 112 * 112 * 3];
    let fit = model.fit(&img, 112, 112, &shifted).unwrap();
    // Yaw should be positive (face turned right from camera).
    assert!(fit.pose.yaw > 0.0, "yaw={}", fit.pose.yaw);
}

// --- Pose3d helpers ---

#[test]
fn pose_frontal_is_near_frontal() {
    let p = Pose3d::default();
    assert!(p.is_near_frontal(1.0));
}

#[test]
fn pose_large_yaw_not_near_frontal() {
    let p = Pose3d {
        yaw: 30.0,
        pitch: 0.0,
        roll: 0.0,
    };
    assert!(!p.is_near_frontal(10.0));
}

// --- Spoof detection ---

#[test]
fn spoof_consistent_pose_returns_live() {
    let det = SpoofDetector::default();
    let recovered = Pose3d {
        yaw: 5.0,
        pitch: 3.0,
        roll: 0.0,
    };
    let expected = Pose3d {
        yaw: 5.0,
        pitch: 3.0,
        roll: 0.0,
    };
    assert_eq!(det.check(&recovered, &expected), SpoofDecision::Live);
}

#[test]
fn spoof_inconsistent_yaw_returns_attack() {
    let det = SpoofDetector::new(15.0, 15.0);
    let recovered = Pose3d {
        yaw: 40.0,
        pitch: 0.0,
        roll: 0.0,
    };
    let expected = Pose3d::default();
    assert!(matches!(
        det.check(&recovered, &expected),
        SpoofDecision::Attack(_)
    ));
}

#[test]
fn spoof_check_frontal_live_for_frontal_pose() {
    let det = SpoofDetector::default();
    let recovered = Pose3d {
        yaw: 5.0,
        pitch: 3.0,
        roll: 1.0,
    };
    assert_eq!(det.check_frontal(&recovered), SpoofDecision::Live);
}

#[test]
fn spoof_check_frontal_attack_for_large_deviation() {
    let det = SpoofDetector::new(10.0, 10.0);
    let recovered = Pose3d {
        yaw: 45.0,
        pitch: 0.0,
        roll: 0.0,
    };
    assert!(matches!(
        det.check_frontal(&recovered),
        SpoofDecision::Attack(_)
    ));
}
