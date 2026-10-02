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

//! FFI bindings to the vendored NBIS C library.
//!
//! Wraps the public API of:
//! - `bozorth3.c` — Bozorth3 minutiae matcher (1:1 + 1:N)
//! - `mindtct` — minutiae extraction (image → template)
//! - `nfiq` — NIST Fingerprint Image Quality
//!
//! All calls are unsafe but the Rust wrappers below validate inputs and
//! return safe types.

// Raw-pointer FFI to the linked `nbis_core` static library: same scoped
// opt-out rationale as `packages/verifier-rs/src/ffi.rs` (workspace denies
// `unsafe_code` via RUSTFLAGS; the C boundary cannot comply).
#![allow(unsafe_code)]

use std::os::raw::c_int;

/// A minutia — NIST internal format. The template is an array of these
/// (X, Y, Theta, Quality).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NbisMinutia {
    pub x: c_int,
    pub y: c_int,
    pub theta: c_int,    // angle in tenths of degrees [0..1799]
    pub quality: c_int,  // 0..100
    pub mclass: c_int,   // 0=end, 1=bifurcation, 2=don't care
}

#[repr(C)]
pub struct MinutiaXytHeader {
    pub width: c_int,
    pub height: c_int,
    pub n: c_int,
}

/// Maximum minutiae Bozorth3 handles (bozorth.h MAX_BOZORTH_MINUTIAE).
const MAX_BOZORTH_MINUTIAE: usize = 200;

/// NBIS `struct xyt_struct` (bozorth3/include/bozorth.h).
#[repr(C)]
pub struct XytStruct {
    pub nrows: c_int,
    pub xcol: [c_int; MAX_BOZORTH_MINUTIAE],
    pub ycol: [c_int; MAX_BOZORTH_MINUTIAE],
    pub thetacol: [c_int; MAX_BOZORTH_MINUTIAE],
}

impl Default for XytStruct {
    fn default() -> Self {
        XytStruct {
            nrows: 0,
            xcol: [0; MAX_BOZORTH_MINUTIAE],
            ycol: [0; MAX_BOZORTH_MINUTIAE],
            thetacol: [0; MAX_BOZORTH_MINUTIAE],
        }
    }
}

impl XytStruct {
    pub fn from_minutiae(minutiae: &[NbisMinutia]) -> Self {
        let mut xy = XytStruct::default();
        let n = minutiae.len().min(MAX_BOZORTH_MINUTIAE) as c_int;
        xy.nrows = n;
        for (i, m) in minutiae.iter().take(n as usize).enumerate() {
            xy.xcol[i] = m.x;
            xy.ycol[i] = m.y;
            xy.thetacol[i] = m.theta;
        }
        xy
    }
}

/// Bozorth3: compute match score between two sets of NBIS minutiae.
///
/// Wraps the real `bozorth_main(pstruct, gstruct)` which calls
/// `bozorth_probe_init` + `bozorth_gallery_init` + `bz_match` +
/// `bz_match_score` internally. Returns integer score; higher = better.
/// Matches above ~40 are generally accepted as the same finger.
#[link(name = "nbis_core", kind = "static")]
extern "C" {
    fn bozorth_main(pstruct: *mut XytStruct, gstruct: *mut XytStruct) -> c_int;
}

pub fn bozorth_match(
    probe: &[NbisMinutia],
    enrolled: &[NbisMinutia],
) -> i32 {
    if probe.is_empty() || enrolled.is_empty() {
        return 0;
    }
    let mut p = XytStruct::from_minutiae(probe);
    let mut g = XytStruct::from_minutiae(enrolled);
    unsafe { bozorth_main(&mut p, &mut g) }
}
