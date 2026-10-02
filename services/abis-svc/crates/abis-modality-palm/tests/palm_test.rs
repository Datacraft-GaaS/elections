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
use eel_abis_modality_palm::{
    domain::{Minutia, MinutiaType, PalmPosition, PalmTemplate},
    matcher::{match_palm, Score, DEFAULT_THRESHOLD},
    parser::{parse_iso19794_2, serialize_iso19794_2, PalmError},
    quality::{score_quality, segment_zones},
};

fn make_minutia(x: u16, y: u16, angle: u8, quality: u8) -> Minutia {
    Minutia {
        x,
        y,
        angle,
        quality,
        minutia_type: MinutiaType::RidgeEnding,
    }
}

fn make_template(minutiae: Vec<Minutia>, crease_pattern: Vec<u8>) -> PalmTemplate {
    PalmTemplate {
        template_id: "test-001".into(),
        palm_position: PalmPosition::RightFull,
        minutiae,
        crease_pattern,
        orientation_field: None,
        nfiq2_palm: None,
        captured_at: Utc::now(),
        zones: vec![],
    }
}

fn valid_buffer(minutiae: &[Minutia]) -> Vec<u8> {
    serialize_iso19794_2(minutiae, 500, 400)
}

// Test 1: parse_iso19794_2 round-trip with valid data.
#[test]
fn test_parse_round_trip_minutia_values() {
    let minutiae = vec![
        make_minutia(100, 200, 45, 80),
        make_minutia(300, 150, 90, 70),
    ];
    let buf = valid_buffer(&minutiae);
    let parsed = parse_iso19794_2(&buf).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].x, 100);
    assert_eq!(parsed[0].y, 200);
    assert_eq!(parsed[0].angle, 45);
    assert_eq!(parsed[0].quality, 80);
    assert_eq!(parsed[1].x, 300);
    assert_eq!(parsed[1].y, 150);
}

// Test 2: parse rejects data shorter than header.
#[test]
fn test_parse_rejects_short_buffer() {
    let buf = vec![0x46, 0x4D, 0x52, 0x00, 0x00]; // too short
    let result = parse_iso19794_2(&buf);
    assert!(matches!(result, Err(PalmError::BufferTooShort { .. })));
}

// Test 3: parse rejects invalid magic.
#[test]
fn test_parse_rejects_invalid_magic() {
    let mut buf = valid_buffer(&[make_minutia(10, 10, 0, 50)]);
    buf[0] = 0xFF; // corrupt magic
    let result = parse_iso19794_2(&buf);
    assert!(matches!(result, Err(PalmError::InvalidMagic)));
}

// Test 4: serialize then parse produces identical minutiae count.
#[test]
fn test_serialize_parse_count_identity() {
    let minutiae: Vec<Minutia> = (0..10)
        .map(|i| make_minutia(i * 30, i * 20, i as u8 * 15, 75))
        .collect();
    let buf = valid_buffer(&minutiae);
    let parsed = parse_iso19794_2(&buf).unwrap();
    assert_eq!(parsed.len(), minutiae.len());
}

// Test 5: match_palm genuine pair returns score > threshold.
#[test]
fn test_match_genuine_pair_above_threshold() {
    let minutiae: Vec<Minutia> = (0..20)
        .map(|i| make_minutia(i * 20, i * 15, i as u8 * 5, 80))
        .collect();
    let probe = make_template(minutiae.clone(), vec![0xAA, 0xBB]);
    let gallery = make_template(minutiae, vec![0xAA, 0xBB]);
    let score = match_palm(&probe, &gallery);
    assert!(
        score.raw > DEFAULT_THRESHOLD,
        "expected score > {DEFAULT_THRESHOLD}, got {}",
        score.raw
    );
}

// Test 6: match_palm imposter pair returns score < threshold.
#[test]
fn test_match_imposter_pair_below_threshold() {
    let probe_min: Vec<Minutia> = (0..15)
        .map(|i| make_minutia(i * 10, i * 10, i as u8 * 12, 75))
        .collect();
    let gallery_min: Vec<Minutia> = (0..15)
        .map(|i| make_minutia(i * 10 + 200, i * 10 + 200, (i as u8).wrapping_mul(90), 75))
        .collect();
    let probe = make_template(probe_min, vec![0xAA]);
    let gallery = make_template(gallery_min, vec![0xFF]);
    let score = match_palm(&probe, &gallery);
    assert!(
        score.raw < DEFAULT_THRESHOLD,
        "expected score < {DEFAULT_THRESHOLD}, got {}",
        score.raw
    );
}

// Test 7: threshold_pass=true when score >= DEFAULT_THRESHOLD.
#[test]
fn test_threshold_pass_when_above() {
    let minutiae: Vec<Minutia> = (0..20)
        .map(|i| make_minutia(i * 20, i * 15, i as u8 * 5, 80))
        .collect();
    let t = make_template(minutiae.clone(), vec![]);
    let score = match_palm(&t, &t);
    assert!(score.threshold_pass);
}

// Test 8: threshold_pass=false when score < DEFAULT_THRESHOLD.
#[test]
fn test_threshold_fail_when_below() {
    let probe_min: Vec<Minutia> = (0..10)
        .map(|i| make_minutia(i * 10, i * 10, i as u8, 75))
        .collect();
    let gallery_min: Vec<Minutia> = (0..10)
        .map(|i| make_minutia(i * 10 + 300, i * 10 + 300, (i as u8).wrapping_add(90), 75))
        .collect();
    let probe = make_template(probe_min, vec![0x01]);
    let gallery = make_template(gallery_min, vec![0xFF]);
    let score = match_palm(&probe, &gallery);
    assert!(!score.threshold_pass);
}

// Test 9: quality scorer returns 0.0 for empty minutiae.
#[test]
fn test_quality_empty_returns_zero() {
    let t = make_template(vec![], vec![]);
    assert_eq!(score_quality(&t), 0.0);
}

// Test 10: quality scorer returns value in [0, 100] for normal input.
#[test]
fn test_quality_in_range() {
    let minutiae: Vec<Minutia> = (0..30)
        .map(|i| make_minutia((i * 15) as u16, (i * 10) as u16, i as u8 * 5, 80))
        .collect();
    let t = make_template(minutiae, vec![]);
    let q = score_quality(&t);
    assert!(q >= 0.0 && q <= 100.0, "quality out of range: {q}");
}

// Test 11: segment_zones produces exactly 5 zones.
#[test]
fn test_segment_zones_count() {
    let minutiae: Vec<Minutia> = (0..20)
        .map(|i| make_minutia((i * 25) as u16, (i * 20) as u16, 0, 80))
        .collect();
    let t = make_template(minutiae, vec![]);
    let zones = segment_zones(&t);
    assert_eq!(zones.len(), 5);
}

// Test 12: crease bonus applied when crease overlap > 50%.
#[test]
fn test_crease_bonus_applied() {
    let minutiae: Vec<Minutia> = (0..10)
        .map(|i| make_minutia(i * 50, i * 40, i as u8 * 10, 80))
        .collect();
    // matching crease patterns to trigger bonus
    let crease = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    let probe = make_template(minutiae.clone(), crease.clone());
    let gallery = make_template(minutiae, crease);
    let score = match_palm(&probe, &gallery);
    // With identical minutiae and matching crease, score should be at least base + bonus
    assert!(
        score.raw >= 0.95,
        "expected bonus applied, got {}",
        score.raw
    );
}

// Test 13: rotation tolerance - minutiae rotated ~29 deg still match.
#[test]
fn test_rotation_tolerance() {
    let probe_min = vec![make_minutia(100, 100, 50, 80)];
    // rotate by 29 degrees (within ROT_TOL_DEG=30)
    let gallery_min = vec![make_minutia(100, 100, 50 + 29, 80)];
    let probe = make_template(probe_min, vec![]);
    let gallery = make_template(gallery_min, vec![]);
    let score = match_palm(&probe, &gallery);
    assert!(
        score.raw >= DEFAULT_THRESHOLD,
        "rotation within tolerance should match, got {}",
        score.raw
    );
}

// Test 14: translation tolerance - minutiae shifted 79px still match.
#[test]
fn test_translation_tolerance() {
    let probe_min = vec![make_minutia(100, 100, 45, 80)];
    // shift by 79px in x (within TRANS_TOL_PX=80)
    let gallery_min = vec![make_minutia(100 + 79, 100, 45, 80)];
    let probe = make_template(probe_min, vec![]);
    let gallery = make_template(gallery_min, vec![]);
    let score = match_palm(&probe, &gallery);
    assert!(
        score.raw >= DEFAULT_THRESHOLD,
        "translation within tolerance should match, got {}",
        score.raw
    );
}
