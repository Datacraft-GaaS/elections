//! FFI bindings to the vendored NBIS C library.
//!
//! Wraps the public API of:
//! - `bozorth3.c` — Bozorth3 minutiae matcher (1:1 + 1:N)
//! - `mindtct` — minutiae extraction (image → template)
//! - `nfiq` — NIST Fingerprint Image Quality
//!
//! All calls are unsafe but the Rust wrappers below validate inputs and
//! return safe types.

use std::os::raw::{c_int, c_char, c_double};

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

/// Bozorth3: compute match score between two .xyt byte arrays.
/// Returns integer score. Higher = better match.
#[link(name = "nbis_core", kind = "static")]
extern "C" {
    /// Legacy ANSI C entry: takes file paths.
    /// `bozorth3(int argc, char **argv)` — but we use the direct call.
    fn bz_match(
        probe_x: *const c_int, probe_y: *const c_int, probe_theta: *const c_int,
        probe_count: c_int,
        enrolled_x: *const c_int, enrolled_y: *const c_int, enrolled_theta: *const c_int,
        enrolled_count: c_int,
    ) -> c_int;
}

pub fn bozorth_match(
    probe: &[NbisMinutia],
    enrolled: &[NbisMinutia],
) -> i32 {
    if probe.is_empty() || enrolled.is_empty() {
        return 0;
    }
    unsafe {
        let mut p_x = Vec::with_capacity(probe.len());
        let mut p_y = Vec::with_capacity(probe.len());
        let mut p_t = Vec::with_capacity(probe.len());
        for m in probe {
            p_x.push(m.x);
            p_y.push(m.y);
            p_t.push(m.theta);
        }
        let mut e_x = Vec::with_capacity(enrolled.len());
        let mut e_y = Vec::with_capacity(enrolled.len());
        let mut e_t = Vec::with_capacity(enrolled.len());
        for m in enrolled {
            e_x.push(m.x);
            e_y.push(m.y);
            e_t.push(m.theta);
        }
        bz_match(
            p_x.as_ptr(), p_y.as_ptr(), p_t.as_ptr(), probe.len() as c_int,
            e_x.as_ptr(), e_y.as_ptr(), e_t.as_ptr(), enrolled.len() as c_int,
        )
    }
}
