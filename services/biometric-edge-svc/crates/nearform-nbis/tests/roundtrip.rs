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

//! mindtct → bozorth3 round-trip smoke test over real vendored NBIS.
//!
//! Generates a synthetic ridge image (no test data shipped), extracts
//! minutiae through in-process C `get_minutiae`, matches them through real
//! C `bz_match`, and asserts the self-match outscores a cross-match.
//! Also asserts extraction determinism.

use nearform_nbis::ffi::{bozorth_match, NbisMinutia};
use nearform_nbis::mindtct_ffi::{extract_minutiae, ExtractedMinutia};
use std::os::raw::c_int;

/// 300x300 synthetic fingerprint with ridge structure that mindtct can
/// detect. Uses intersecting ridge flows to create natural minutiae.
fn synthetic_print(seed: u64) -> (Vec<u8>, u32, u32) {
    let (w, h) = (300u32, 300u32);
    let mut px = vec![200u8; (w * h) as usize];
    let mut rnd = seed;
    let mut next_rand = || {
        rnd = rnd
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((rnd >> 33) % 100) as u8
    };
    // Two intersecting ridge systems (vertical + horizontal-ish) creating
    // natural ridge endings and bifurcations.
    for y in 0..h {
        for x in 0..w {
            let fx = x as f64;
            let fy = y as f64;
            // System 1: vertical ridges with slow horizontal drift
            let d1 = (fx + fy * 0.03).sin() * 8.0;
            let p1 = (fx + d1) * std::f64::consts::TAU / 14.0;
            let v1 = 128.0 + 80.0 * p1.sin();
            // System 2: horizontal ridges with slow vertical drift
            let d2 = (fy + fx * 0.02).sin() * 6.0;
            let p2 = (fy + d2) * std::f64::consts::TAU / 16.0;
            let v2 = 128.0 + 60.0 * p2.sin();
            // Combine: ridge intersections create minutiae
            let mut v = (v1 + v2) * 0.5;
            // Add a few deliberate ridge endings/bifurcations
            if (x > 80 && x < 90 && y > 80 && y < 180) ||   // vertical line ending
               (x > 120 && x < 180 && y > 120 && y < 130) || // horizontal line ending
               (x > 200 && x < 210 && y > 200 && y < 210)
            {
                // bifurcation spot
                v = 100.0 + next_rand() as f64 * 50.0; // break the ridge
            }
            let noise = next_rand() as f64 * 0.3;
            px[(y * w + x) as usize] = (v + noise).clamp(0.0, 255.0) as u8;
        }
    }
    (px, w, h)
}

fn to_nbis(ms: &[ExtractedMinutia]) -> Vec<NbisMinutia> {
    ms.iter()
        .map(|m| NbisMinutia {
            x: m.x as c_int,
            y: m.y as c_int,
            // direction is degrees; NBIStheta is tenths in [0..1800).
            theta: (m.direction.rem_euclid(180) as f64 * 10.0) as c_int,
            quality: (m.reliability.clamp(0.0, 1.0) * 100.0) as c_int,
            // lfs.h: BIFURCATION 0, RIDGE_ENDING 1.
            mclass: if m.kind == 0 { 0 } else { 1 },
        })
        .collect()
}

#[test]
fn mindtct_extracts_minutiae_from_synthetic_print() {
    let (px, w, h) = synthetic_print(7);
    let ms = extract_minutiae(&px, w, h, 19.69).expect("extraction runs");
    assert!(
        !ms.is_empty(),
        "real mindtct must find minutiae on ridge structure"
    );
}

#[test]
fn bozorth_self_match_beats_cross_match() {
    let (a_px, aw, ah) = synthetic_print(7);
    let (b_px, bw, bh) = synthetic_print(999);
    let a = to_nbis(&extract_minutiae(&a_px, aw, ah, 19.69).expect("extract a"));
    let b = to_nbis(&extract_minutiae(&b_px, bw, bh, 19.69).expect("extract b"));
    assert!(!a.is_empty() && !b.is_empty(), "both prints yield minutiae");

    let self_score = bozorth_match(&a, &a);
    let cross_score = bozorth_match(&a, &b);
    assert!(
        self_score > cross_score,
        "self-match ({self_score}) must beat cross-match ({cross_score})"
    );
    assert!(
        self_score > 40,
        "self-match clears the NIST-40 bar, got {self_score}"
    );
}

#[test]
fn extraction_is_deterministic() {
    let (px, w, h) = synthetic_print(7);
    let first = extract_minutiae(&px, w, h, 19.69).expect("run 1");
    let second = extract_minutiae(&px, w, h, 19.69).expect("run 2");
    assert_eq!(first.len(), second.len());
    assert_eq!(first, second);
}
