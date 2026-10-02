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
use eel_abis_modality_gait::{
    domain::{GaitCycle, GaitFrame, GaitTemplate, JointAngle, NUM_JOINTS},
    matcher::{dtw_cycle, frame_distance, match_gait, DEFAULT_THRESHOLD},
    parser::{parse_gait, serialize_gait},
    quality::score_quality,
};

fn zero_joint() -> JointAngle {
    JointAngle {
        pitch: 0.0,
        yaw: 0.0,
        roll: 0.0,
    }
}

fn make_frame(idx: u32) -> GaitFrame {
    GaitFrame {
        frame_idx: idx,
        joints: [zero_joint(); NUM_JOINTS],
    }
}

fn make_frame_with(idx: u32, pitch: f32) -> GaitFrame {
    let mut j = zero_joint();
    j.pitch = pitch;
    GaitFrame {
        frame_idx: idx,
        joints: [j; NUM_JOINTS],
    }
}

fn make_cycle(num_frames: usize) -> GaitCycle {
    GaitCycle {
        frames: (0..num_frames).map(|i| make_frame(i as u32)).collect(),
        duration_ms: 800,
    }
}

fn make_template(num_cycles: usize, num_frames: usize) -> GaitTemplate {
    GaitTemplate {
        template_id: "t".to_string(),
        cycles: (0..num_cycles).map(|_| make_cycle(num_frames)).collect(),
        cadence_steps_per_min: 100.0,
        walking_speed_ms: 1.5,
        captured_at: Utc::now(),
    }
}

#[test]
fn parse_round_trip() {
    let tmpl = make_template(2, 5);
    let bytes = serialize_gait(&tmpl);
    let parsed = parse_gait(&bytes).expect("should parse");
    assert_eq!(parsed.cycles.len(), 2);
    assert_eq!(parsed.cycles[0].frames.len(), 5);
    assert!((parsed.cadence_steps_per_min - 100.0).abs() < 1e-4);
    assert!((parsed.walking_speed_ms - 1.5).abs() < 1e-4);
}

#[test]
fn parse_rejects_wrong_magic() {
    let mut bytes = serialize_gait(&make_template(1, 3));
    bytes[0] = 0xFF;
    let err = parse_gait(&bytes).unwrap_err();
    assert!(matches!(
        err,
        eel_abis_modality_gait::GaitError::InvalidMagic
    ));
}

#[test]
fn serialize_parse_two_cycles_ten_frames() {
    let tmpl = make_template(2, 10);
    let bytes = serialize_gait(&tmpl);
    let parsed = parse_gait(&bytes).expect("should parse");
    assert_eq!(parsed.cycles.len(), 2);
    for c in &parsed.cycles {
        assert_eq!(c.frames.len(), 10);
    }
}

#[test]
fn frame_distance_identical_is_zero() {
    let f = make_frame(0);
    assert!(frame_distance(&f, &f) < 1e-6);
}

#[test]
fn frame_distance_different_is_positive() {
    let f1 = make_frame(0);
    let f2 = make_frame_with(0, 90.0);
    assert!(frame_distance(&f1, &f2) > 0.0);
}

#[test]
fn dtw_cycle_identical_is_zero() {
    let c = make_cycle(5);
    let cost = dtw_cycle(&c, &c);
    assert!(
        cost < 1e-6,
        "identical cycles should have zero DTW cost, got {cost}"
    );
}

#[test]
fn dtw_cycle_similar_less_than_different() {
    let c_base = make_cycle(5);

    let c_similar: GaitCycle = GaitCycle {
        frames: (0..5).map(|i| make_frame_with(i as u32, 1.0)).collect(),
        duration_ms: 800,
    };

    let c_different: GaitCycle = GaitCycle {
        frames: (0..5).map(|i| make_frame_with(i as u32, 100.0)).collect(),
        duration_ms: 800,
    };

    let cost_similar = dtw_cycle(&c_base, &c_similar);
    let cost_different = dtw_cycle(&c_base, &c_different);

    assert!(
        cost_similar < cost_different,
        "similar cycle should have lower DTW cost: {cost_similar} vs {cost_different}"
    );
}

#[test]
fn match_gait_identical_above_threshold() {
    let t = make_template(3, 8);
    let score = match_gait(&t, &t);
    assert!(
        score.raw > DEFAULT_THRESHOLD,
        "identical match should exceed threshold, got {}",
        score.raw
    );
}

#[test]
fn match_gait_different_cadence_reduces_score() {
    let t1 = make_template(3, 8);
    let mut t2 = make_template(3, 8);
    t2.cadence_steps_per_min = 200.0; // large difference

    let score_same = match_gait(&t1, &t1);
    let score_diff = match_gait(&t1, &t2);

    assert!(
        score_same.raw > score_diff.raw,
        "large cadence diff should reduce score: {} vs {}",
        score_same.raw,
        score_diff.raw
    );
}

#[test]
fn quality_zero_cycles_returns_zero() {
    let t = make_template(0, 0);
    let q = score_quality(&t);
    assert!((q.overall - 0.0).abs() < 1e-6);
}

#[test]
fn quality_five_cycles_optimal_speed_high_overall() {
    let mut t = make_template(5, 8);
    t.walking_speed_ms = 1.5;
    let q = score_quality(&t);
    assert!(
        q.overall > 80.0,
        "5 cycles at 1.5 m/s should be high quality, got {}",
        q.overall
    );
}

#[test]
fn quality_cadence_100_scores_well() {
    let mut t = make_template(5, 8);
    t.cadence_steps_per_min = 100.0;
    t.walking_speed_ms = 1.5;
    let q = score_quality(&t);
    assert!(
        q.overall > 80.0,
        "optimal cadence/speed should yield high quality, got {}",
        q.overall
    );
}

#[test]
fn serialize_correct_byte_structure() {
    // 1 cycle, 1 frame: 4 magic + 4 cadence + 4 speed + 2 cycle_count
    //   + 2 frame_count + 4 duration_ms + 4 frame_idx + NUM_JOINTS*3*4 = 14 + 10 + 204 = 228
    let t = make_template(1, 1);
    let bytes = serialize_gait(&t);
    let expected = 4 + 4 + 4 + 2 + 2 + 4 + 4 + NUM_JOINTS * 3 * 4;
    assert_eq!(
        bytes.len(),
        expected,
        "expected {expected} bytes, got {}",
        bytes.len()
    );
}

#[test]
fn parse_multiple_cycles_correctly() {
    let t = make_template(4, 6);
    let bytes = serialize_gait(&t);
    let parsed = parse_gait(&bytes).expect("parse");
    assert_eq!(parsed.cycles.len(), 4);
    for c in &parsed.cycles {
        assert_eq!(c.frames.len(), 6);
        assert_eq!(c.duration_ms, 800);
    }
}
