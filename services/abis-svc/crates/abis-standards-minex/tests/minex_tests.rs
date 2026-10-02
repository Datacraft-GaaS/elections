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

use abis_standards_minex::{
    interop::compute_interop_score,
    template::{MinexMinutia, MinexTemplate, MINEX_MAGIC},
    validator::MinexValidator,
    MinexError,
};

fn make_minutia(x: u16, y: u16) -> MinexMinutia {
    MinexMinutia {
        x,
        y,
        angle: 0,
        quality: 80,
        mtype: 1,
    }
}

fn template_with_n_minutiae(n: usize) -> MinexTemplate {
    let mut t = MinexTemplate::new(1);
    for i in 0..n {
        t.add_minutia(make_minutia((i * 4) as u16 % 490, (i * 3) as u16 % 490))
            .unwrap();
    }
    t
}

#[test]
fn serialize_parse_roundtrip() {
    let mut tmpl = MinexTemplate::new(2);
    for i in 0..10u16 {
        tmpl.add_minutia(MinexMinutia {
            x: i * 40,
            y: i * 30,
            angle: i as u8 * 10,
            quality: 75,
            mtype: 1,
        })
        .unwrap();
    }
    let bytes = tmpl.serialize();
    let parsed = MinexTemplate::parse(&bytes).unwrap();
    assert_eq!(parsed.finger_position, 2);
    assert_eq!(parsed.minutiae.len(), 10);
    for (orig, got) in tmpl.minutiae.iter().zip(parsed.minutiae.iter()) {
        assert_eq!(orig.x, got.x);
        assert_eq!(orig.y, got.y);
        assert_eq!(orig.angle, got.angle);
        assert_eq!(orig.quality, got.quality);
        assert_eq!(orig.mtype, got.mtype);
    }
}

#[test]
fn add_129th_minutia_returns_error() {
    let mut tmpl = template_with_n_minutiae(128);
    let result = tmpl.add_minutia(make_minutia(10, 10));
    assert!(matches!(
        result,
        Err(MinexError::InvalidMinutiaeCount {
            found: 129,
            max: 128
        })
    ));
}

#[test]
fn out_of_bounds_x_returns_error() {
    let mut tmpl = MinexTemplate::new(1);
    let result = tmpl.add_minutia(make_minutia(501, 100));
    assert!(matches!(
        result,
        Err(MinexError::InvalidCoordinate { x: 501, y: 100 })
    ));
}

#[test]
fn quality_101_returns_error() {
    let mut tmpl = MinexTemplate::new(1);
    let result = tmpl.add_minutia(MinexMinutia {
        x: 100,
        y: 100,
        angle: 0,
        quality: 101,
        mtype: 1,
    });
    assert!(matches!(result, Err(MinexError::InvalidQuality(101))));
}

#[test]
fn validator_empty_template_fails() {
    let tmpl = MinexTemplate::new(1);
    let result = MinexValidator::validate(&tmpl);
    assert!(!result.passes);
    assert!(!result.violations.is_empty());
}

#[test]
fn validator_valid_template_passes() {
    let tmpl = template_with_n_minutiae(30);
    let result = MinexValidator::validate(&tmpl);
    assert!(result.passes, "violations: {:?}", result.violations);
    assert_eq!(result.minutiae_count, 30);
}

#[test]
fn estimated_quality_computed_correctly() {
    let mut tmpl = MinexTemplate::new(1);
    tmpl.add_minutia(MinexMinutia {
        x: 10,
        y: 10,
        angle: 0,
        quality: 60,
        mtype: 1,
    })
    .unwrap();
    tmpl.add_minutia(MinexMinutia {
        x: 20,
        y: 20,
        angle: 0,
        quality: 80,
        mtype: 2,
    })
    .unwrap();
    let result = MinexValidator::validate(&tmpl);
    assert!((result.estimated_quality - 70.0).abs() < 0.01);
}

#[test]
fn interop_score_identical_templates() {
    let tmpl = template_with_n_minutiae(20);
    let result = compute_interop_score(&tmpl, &tmpl);
    assert!(
        (result.score - 1.0).abs() < 1e-5,
        "expected 1.0, got {}",
        result.score
    );
}

#[test]
fn interop_score_non_overlapping_templates() {
    // Probe in top-left quadrant, gallery in bottom-right quadrant.
    let mut probe = MinexTemplate::new(1);
    let mut gallery = MinexTemplate::new(1);
    for i in 0..10u16 {
        probe.add_minutia(make_minutia(i * 4, i * 4)).unwrap();
        gallery.add_minutia(make_minutia(400 + i, 400 + i)).unwrap();
    }
    let result = compute_interop_score(&probe, &gallery);
    assert_eq!(result.minutiae_overlap, 0);
    assert!((result.score - 0.0).abs() < 1e-5);
}

#[test]
fn fmr_magic_bytes_in_serialized_output() {
    let tmpl = MinexTemplate::new(1);
    let bytes = tmpl.serialize();
    assert_eq!(&bytes[..4], &MINEX_MAGIC);
}
