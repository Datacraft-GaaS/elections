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
use eel_abis_modality_ear::{
    domain::{EarTemplate, Point2D, NUM_LANDMARKS},
    matcher::{match_ear, DEFAULT_THRESHOLD},
    parser::{parse_ear, serialize_ear},
    quality::score_quality,
};

fn make_template(landmarks: [Point2D; NUM_LANDMARKS]) -> EarTemplate {
    EarTemplate {
        template_id: "test".to_string(),
        landmarks,
        visibility_mask: 0xFFFF_FFFF,
        rotation_angle_deg: 0.0,
        captured_at: Utc::now(),
    }
}

fn uniform_landmarks(x: f32, y: f32) -> [Point2D; NUM_LANDMARKS] {
    [Point2D { x, y }; NUM_LANDMARKS]
}

fn circle_landmarks(radius: f32) -> [Point2D; NUM_LANDMARKS] {
    let mut lms = [Point2D { x: 0.0, y: 0.0 }; NUM_LANDMARKS];
    for (i, lm) in lms.iter_mut().enumerate() {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / NUM_LANDMARKS as f32;
        lm.x = radius * angle.cos();
        lm.y = radius * angle.sin();
    }
    lms
}

#[test]
fn parse_round_trip() {
    let landmarks = circle_landmarks(1.0);
    let orig = EarTemplate {
        template_id: "rt".to_string(),
        landmarks,
        visibility_mask: 0xAAAA_AAAA,
        rotation_angle_deg: 12.5,
        captured_at: Utc::now(),
    };
    let bytes = serialize_ear(&orig);
    let parsed = parse_ear(&bytes).expect("should parse");
    assert_eq!(parsed.visibility_mask, orig.visibility_mask);
    assert!((parsed.rotation_angle_deg - orig.rotation_angle_deg).abs() < 1e-5);
    for (p, o) in parsed.landmarks.iter().zip(orig.landmarks.iter()) {
        assert!((p.x - o.x).abs() < 1e-5, "x mismatch");
        assert!((p.y - o.y).abs() < 1e-5, "y mismatch");
    }
}

#[test]
fn parse_rejects_wrong_magic() {
    let mut bytes = serialize_ear(&make_template(circle_landmarks(1.0)));
    bytes[0] = 0xFF;
    let err = parse_ear(&bytes).unwrap_err();
    assert!(matches!(err, eel_abis_modality_ear::EarError::InvalidMagic));
}

#[test]
fn parse_rejects_short_buffer() {
    let bytes = vec![0x45, 0x41, 0x52, 0x4C, 0x00];
    let err = parse_ear(&bytes).unwrap_err();
    assert!(matches!(
        err,
        eel_abis_modality_ear::EarError::BufferTooShort { .. }
    ));
}

#[test]
fn serialize_correct_byte_length() {
    let tmpl = make_template(circle_landmarks(1.0));
    let bytes = serialize_ear(&tmpl);
    // 4 magic + 4 mask + 4 rotation + 32 * 8 landmarks = 268
    assert_eq!(bytes.len(), 268);
}

#[test]
fn landmark_visible_works() {
    let tmpl = EarTemplate {
        template_id: "v".to_string(),
        landmarks: [Point2D { x: 0.0, y: 0.0 }; NUM_LANDMARKS],
        visibility_mask: 0b0000_0101,
        rotation_angle_deg: 0.0,
        captured_at: Utc::now(),
    };
    assert!(tmpl.landmark_visible(0));
    assert!(!tmpl.landmark_visible(1));
    assert!(tmpl.landmark_visible(2));
    assert!(!tmpl.landmark_visible(3));
}

#[test]
fn visible_count_correct() {
    let tmpl = EarTemplate {
        template_id: "vc".to_string(),
        landmarks: [Point2D { x: 0.0, y: 0.0 }; NUM_LANDMARKS],
        visibility_mask: 0b1111,
        rotation_angle_deg: 0.0,
        captured_at: Utc::now(),
    };
    assert_eq!(tmpl.visible_count(), 4);
}

#[test]
fn procrustes_distance_identical_shapes() {
    let landmarks = circle_landmarks(1.0);
    let t1 = make_template(landmarks);
    let t2 = make_template(landmarks);
    let score = match_ear(&t1, &t2);
    assert!(
        score.procrustes_dist < 1e-4,
        "identical shapes should have near-zero distance, got {}",
        score.procrustes_dist
    );
}

#[test]
fn procrustes_distance_scale_invariant() {
    let lms1 = circle_landmarks(1.0);
    let lms2 = circle_landmarks(2.0);
    let t1 = make_template(lms1);
    let t2 = make_template(lms1);
    let t3 = make_template(lms2);

    let score_same = match_ear(&t1, &t2);
    let score_scaled = match_ear(&t1, &t3);

    assert!(
        (score_same.procrustes_dist - score_scaled.procrustes_dist).abs() < 1e-3,
        "scale should not affect procrustes distance: {} vs {}",
        score_same.procrustes_dist,
        score_scaled.procrustes_dist
    );
}

#[test]
fn procrustes_distance_rotation_invariant() {
    let lms = circle_landmarks(1.0);
    let mut lms_rot = lms;
    let theta = std::f32::consts::FRAC_PI_4; // 45 degrees
    for p in lms_rot.iter_mut() {
        let nx = p.x * theta.cos() - p.y * theta.sin();
        let ny = p.x * theta.sin() + p.y * theta.cos();
        p.x = nx;
        p.y = ny;
    }

    let t1 = make_template(lms);
    let t2 = make_template(lms_rot);
    let score = match_ear(&t1, &t2);

    assert!(
        score.procrustes_dist < 1e-3,
        "rotation should not affect procrustes distance, got {}",
        score.procrustes_dist
    );
}

#[test]
fn match_identical_template_near_one() {
    let landmarks = circle_landmarks(1.0);
    let t = make_template(landmarks);
    let score = match_ear(&t, &t);
    assert!(
        score.raw > 0.99,
        "identical match should be near 1.0, got {}",
        score.raw
    );
}

#[test]
fn match_different_shapes_below_threshold() {
    let lms_a = circle_landmarks(1.0);
    let mut lms_b = [Point2D { x: 0.0, y: 0.0 }; NUM_LANDMARKS];
    for (i, p) in lms_b.iter_mut().enumerate() {
        p.x = (i as f32) * 3.7;
        p.y = (i as f32) * -2.1;
    }
    let t1 = make_template(lms_a);
    let t2 = make_template(lms_b);
    let score = match_ear(&t1, &t2);
    assert!(
        !score.threshold_pass || score.raw < 0.95,
        "very different shapes should score low: {}",
        score.raw
    );
}

#[test]
fn quality_all_visible_scores_100() {
    let tmpl = EarTemplate {
        template_id: "q1".to_string(),
        landmarks: [Point2D { x: 0.0, y: 0.0 }; NUM_LANDMARKS],
        visibility_mask: 0xFFFF_FFFF,
        rotation_angle_deg: 0.0,
        captured_at: Utc::now(),
    };
    let q = score_quality(&tmpl);
    assert!((q.visibility_score - 100.0).abs() < 1e-3);
}

#[test]
fn quality_rotation_zero_deg_scores_100() {
    let tmpl = EarTemplate {
        template_id: "q2".to_string(),
        landmarks: [Point2D { x: 0.0, y: 0.0 }; NUM_LANDMARKS],
        visibility_mask: 0xFFFF_FFFF,
        rotation_angle_deg: 0.0,
        captured_at: Utc::now(),
    };
    let q = score_quality(&tmpl);
    assert!((q.rotation_score - 100.0).abs() < 1e-3);
}

#[test]
fn quality_rotation_90_deg_low_score() {
    let tmpl = EarTemplate {
        template_id: "q3".to_string(),
        landmarks: [Point2D { x: 0.0, y: 0.0 }; NUM_LANDMARKS],
        visibility_mask: 0xFFFF_FFFF,
        rotation_angle_deg: 90.0,
        captured_at: Utc::now(),
    };
    let q = score_quality(&tmpl);
    assert!(
        q.rotation_score < 10.0,
        "rotation 90 should yield low score, got {}",
        q.rotation_score
    );
}
