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

//! In-process C mindtct extraction (`get_minutiae`) over the vendored
//! NBIS static library.
//!
//! Unlike `mindtct.rs` (which shells out to an installed `bin/mindtct`
//! binary), this module calls the extraction core directly: grayscale bytes
//! in, minutiae out. Used by the round-trip smoke test
//! (mindtct → `bz_match`) and available to future workflow integration.

// Raw-pointer FFI: same scoped opt-out rationale as `ffi.rs`.
#![allow(unsafe_code)]

use std::os::raw::{c_double, c_int, c_uchar, c_void};

/// Opaque default parameters (exported `lfsparms_V2` global).
#[repr(C)]
pub struct LfsParams([u8; 0]);

/// Mirrors NBIS `MINUTIA` (lfs.h). Only x/y/direction/reliability/type
/// are read; neighbor lists are skipped.
#[repr(C)]
pub struct CMINUTIA {
    pub x: c_int,
    pub y: c_int,
    pub ex: c_int,
    pub ey: c_int,
    pub direction: c_int,
    pub reliability: c_double,
    pub type_: c_int,
    pub appearing: c_int,
    pub feature_id: c_int,
    pub nbrs: *mut c_int,
    pub ridge_counts: *mut c_int,
    pub num_nbrs: c_int,
}

/// Mirrors NBIS `MINUTIAE` (lfs.h).
#[repr(C)]
pub struct CMINUTIAE {
    pub alloc: c_int,
    pub num: c_int,
    pub list: *mut *mut CMINUTIA,
}

extern "C" {
    static lfsparms_V2: LfsParams;
    fn get_minutiae(
        ominutiae: *mut *mut CMINUTIAE,
        oquality_map: *mut *mut c_int,
        odirection_map: *mut *mut c_int,
        olow_contrast_map: *mut *mut c_int,
        olow_flow_map: *mut *mut c_int,
        ohigh_curve_map: *mut *mut c_int,
        omap_w: *mut c_int,
        omap_h: *mut c_int,
        obdata: *mut *mut c_uchar,
        obw: *mut c_int,
        obh: *mut c_int,
        obd: *mut c_int,
        idata: *const c_uchar,
        iw: c_int,
        ih: c_int,
        id: c_int,
        ppmm: c_double,
        lfsparms: *const LfsParams,
    ) -> c_int;
    fn free_minutiae(minutiae: *mut CMINUTIAE);
}

/// One extracted minutia (owned, safe).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtractedMinutia {
    pub x: i32,
    pub y: i32,
    /// Ridge direction, degrees.
    pub direction: i32,
    /// Reliability 0.0..=1.0.
    pub reliability: f64,
    /// 0 = bifurcation, 1 = ridge ending (NBIS convention).
    pub kind: i32,
}

/// Extract minutiae from 8-bit grayscale bytes via real C mindtct.
///
/// `ppmm` is pixels per millimetre (≈19.69 for 500 ppi). Returns an empty
/// vec (not an error) when nothing is found; errors only on C failure.
pub fn extract_minutiae(
    pixels: &[u8],
    width: u32,
    height: u32,
    ppmm: f64,
) -> Result<Vec<ExtractedMinutia>, String> {
    if pixels.len() != width as usize * height as usize || width == 0 || height == 0 {
        return Err(format!(
            "bad image: {} bytes for {width}x{height}",
            pixels.len()
        ));
    }
    unsafe {
        let mut minutiae: *mut CMINUTIAE = std::ptr::null_mut();
        let mut quality_map: *mut c_int = std::ptr::null_mut();
        let mut direction_map: *mut c_int = std::ptr::null_mut();
        let mut low_contrast_map: *mut c_int = std::ptr::null_mut();
        let mut low_flow_map: *mut c_int = std::ptr::null_mut();
        let mut high_curve_map: *mut c_int = std::ptr::null_mut();
        let mut map_w: c_int = 0;
        let mut map_h: c_int = 0;
        let mut bdata: *mut c_uchar = std::ptr::null_mut();
        let mut bw: c_int = 0;
        let mut bh: c_int = 0;
        let mut bd: c_int = 0;
        let ret = get_minutiae(
            &mut minutiae,
            &mut quality_map,
            &mut direction_map,
            &mut low_contrast_map,
            &mut low_flow_map,
            &mut high_curve_map,
            &mut map_w,
            &mut map_h,
            &mut bdata,
            &mut bw,
            &mut bh,
            &mut bd,
            pixels.as_ptr(),
            width as c_int,
            height as c_int,
            8,
            ppmm,
            &lfsparms_V2,
        );
        // Free every C-allocated buffer (all malloc'd by NBIS).
        let free_ptr = |p: *mut c_void| {
            if !p.is_null() {
                libc::free(p);
            }
        };
        free_ptr(quality_map as *mut c_void);
        free_ptr(direction_map as *mut c_void);
        free_ptr(low_contrast_map as *mut c_void);
        free_ptr(low_flow_map as *mut c_void);
        free_ptr(high_curve_map as *mut c_void);
        free_ptr(bdata as *mut c_void);
        if ret < 0 {
            return Err(format!("get_minutiae failed: {ret}"));
        }
        if minutiae.is_null() {
            return Ok(vec![]);
        }
        let n = (*minutiae).num as usize;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let m = &**(*minutiae).list.add(i);
            out.push(ExtractedMinutia {
                x: m.x,
                y: m.y,
                direction: m.direction,
                reliability: m.reliability,
                kind: m.type_,
            });
        }
        free_minutiae(minutiae);
        Ok(out)
    }
}
