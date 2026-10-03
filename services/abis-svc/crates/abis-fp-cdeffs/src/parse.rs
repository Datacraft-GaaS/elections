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

//! Parse ANSI/NIST-ITL Type-9 extended fields (300-999) into ExtendedFeatureSet.
//!
//! Reference: NIST SP 500-290 e3 Annex A; ANSI/NIST-ITL 1-2011 Type-9.
//!
//! Wire format (each extended field):
//!   u16 BE  field tag (e.g. 300, 301, ...)
//!   u16 BE  payload length in bytes
//!   [u8]    payload
//!
//! The function reads until `bytes` is exhausted or an unknown field is skipped.

use thiserror::Error;

use crate::features::{
    Dot, ExtendedFeatureSet, IncipientRidge, MinutiaExtended, PatternClass, RidgeEdgeShape,
    RidgeFlowMap, RidgeQualityMap, RidgeWavelengthMap, SegmentationMask, SingularPoint,
    SingularPointKind, SkinCondition, SkinConditionKind,
};

/// Error type for EFS parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("buffer too short: need {need}, have {have}")]
    TooShort { need: usize, have: usize },
    #[error("invalid field tag: {0}")]
    InvalidTag(u16),
    #[error("invalid value: {0}")]
    InvalidValue(String),
}

fn read_u16_be(buf: &[u8], off: usize) -> Result<u16, ParseError> {
    if off + 2 > buf.len() {
        return Err(ParseError::TooShort {
            need: off + 2,
            have: buf.len(),
        });
    }
    Ok(u16::from_be_bytes([buf[off], buf[off + 1]]))
}

/// Parse Type-9 fields 300+ from a byte slice into ExtendedFeatureSet.
pub fn parse_efs(bytes: &[u8]) -> Result<ExtendedFeatureSet, ParseError> {
    let mut efs = ExtendedFeatureSet::new(PatternClass::Unknown);
    let mut pos = 0usize;

    while pos + 4 <= bytes.len() {
        let tag = read_u16_be(bytes, pos)?;
        let len = read_u16_be(bytes, pos + 2)? as usize;
        pos += 4;

        if pos + len > bytes.len() {
            return Err(ParseError::TooShort {
                need: pos + len,
                have: bytes.len(),
            });
        }
        let payload = &bytes[pos..pos + len];
        pos += len;

        match tag {
            300 => {
                // Pattern class: 1 byte
                if payload.is_empty() {
                    return Err(ParseError::TooShort { need: 1, have: 0 });
                }
                efs.pattern_class = PatternClass::from_code(payload[0]);
            }
            301 => {
                // Singular points: each record = 1 (kind) + 2 (x) + 2 (y) + 2 (orientation*10) = 7 bytes
                let mut p = 0;
                while p + 7 <= payload.len() {
                    let kind = if payload[p] == 0 {
                        SingularPointKind::Core
                    } else {
                        SingularPointKind::Delta
                    };
                    let x = u16::from_be_bytes([payload[p + 1], payload[p + 2]]);
                    let y = u16::from_be_bytes([payload[p + 3], payload[p + 4]]);
                    let orient_raw = u16::from_be_bytes([payload[p + 5], payload[p + 6]]);
                    let orientation = orient_raw as f32 / 10.0;
                    efs.singular_points.push(SingularPoint {
                        kind,
                        x,
                        y,
                        orientation,
                    });
                    p += 7;
                }
            }
            302 => {
                // Ridge quality map: 2 (w) + 2 (h) + w*h bytes
                if payload.len() < 4 {
                    return Err(ParseError::TooShort {
                        need: 4,
                        have: payload.len(),
                    });
                }
                let w = u16::from_be_bytes([payload[0], payload[1]]);
                let h = u16::from_be_bytes([payload[2], payload[3]]);
                let expected = 4 + w as usize * h as usize;
                if payload.len() < expected {
                    return Err(ParseError::TooShort {
                        need: expected,
                        have: payload.len(),
                    });
                }
                efs.quality_map = Some(RidgeQualityMap {
                    width: w,
                    height: h,
                    data: payload[4..expected].to_vec(),
                });
            }
            303 => {
                // Ridge flow map: 2 (w) + 2 (h) + w*h bytes
                if payload.len() < 4 {
                    return Err(ParseError::TooShort {
                        need: 4,
                        have: payload.len(),
                    });
                }
                let w = u16::from_be_bytes([payload[0], payload[1]]);
                let h = u16::from_be_bytes([payload[2], payload[3]]);
                let expected = 4 + w as usize * h as usize;
                if payload.len() < expected {
                    return Err(ParseError::TooShort {
                        need: expected,
                        have: payload.len(),
                    });
                }
                efs.flow_map = Some(RidgeFlowMap {
                    width: w,
                    height: h,
                    data: payload[4..expected].to_vec(),
                });
            }
            304 => {
                // Ridge wavelength map: 2 (w) + 2 (h) + w*h*2 bytes (u16 per pixel)
                if payload.len() < 4 {
                    return Err(ParseError::TooShort {
                        need: 4,
                        have: payload.len(),
                    });
                }
                let w = u16::from_be_bytes([payload[0], payload[1]]);
                let h = u16::from_be_bytes([payload[2], payload[3]]);
                let expected = 4 + w as usize * h as usize * 2;
                if payload.len() < expected {
                    return Err(ParseError::TooShort {
                        need: expected,
                        have: payload.len(),
                    });
                }
                let mut data = Vec::with_capacity(w as usize * h as usize);
                let mut q = 4;
                while q + 2 <= payload.len() {
                    data.push(u16::from_be_bytes([payload[q], payload[q + 1]]));
                    q += 2;
                }
                efs.wavelength_map = Some(RidgeWavelengthMap {
                    width: w,
                    height: h,
                    data,
                });
            }
            305 => {
                // Segmentation mask: 2 (w) + 2 (h) + ceil(w*h/8) bytes
                if payload.len() < 4 {
                    return Err(ParseError::TooShort {
                        need: 4,
                        have: payload.len(),
                    });
                }
                let w = u16::from_be_bytes([payload[0], payload[1]]);
                let h = u16::from_be_bytes([payload[2], payload[3]]);
                let bits = w as usize * h as usize;
                let expected = 4 + bits.div_ceil(8);
                if payload.len() < expected {
                    return Err(ParseError::TooShort {
                        need: expected,
                        have: payload.len(),
                    });
                }
                efs.segmentation = Some(SegmentationMask {
                    width: w,
                    height: h,
                    data: payload[4..expected].to_vec(),
                });
            }
            320 => {
                // Minutia extended: each record = 1 (idx) + 1 (feature_type) + 1 (flags) + 1 (n_neighbours)
                //   + n_neighbours * 2 (neighbour_idx u8 + ridge_count u8)
                let mut p = 0;
                while p < payload.len() {
                    if p + 4 > payload.len() {
                        break;
                    }
                    let minutia_idx = payload[p] as usize;
                    let feature_type = payload[p + 1];
                    let flags = payload[p + 2];
                    let n = payload[p + 3] as usize;
                    p += 4;
                    if p + n * 2 > payload.len() {
                        break;
                    }
                    let mut rc = Vec::with_capacity(n);
                    for _ in 0..n {
                        let nidx = payload[p] as usize;
                        let count = payload[p + 1];
                        rc.push((nidx, count));
                        p += 2;
                    }
                    efs.minutia_extended.push(MinutiaExtended {
                        minutia_idx,
                        ridge_counts_to_neighbours: rc,
                        feature_type,
                        is_dot: flags & 0x01 != 0,
                        is_incipient: flags & 0x02 != 0,
                    });
                }
            }
            330 => {
                // Dots: each record = 2 (x) + 2 (y) + 1 (quality) = 5 bytes
                let mut p = 0;
                while p + 5 <= payload.len() {
                    let x = u16::from_be_bytes([payload[p], payload[p + 1]]);
                    let y = u16::from_be_bytes([payload[p + 2], payload[p + 3]]);
                    let quality = payload[p + 4];
                    efs.dots.push(Dot { x, y, quality });
                    p += 5;
                }
            }
            331 => {
                // Incipient ridges: each = 2+2+2+2+1 = 9 bytes
                let mut p = 0;
                while p + 9 <= payload.len() {
                    let sx = u16::from_be_bytes([payload[p], payload[p + 1]]);
                    let sy = u16::from_be_bytes([payload[p + 2], payload[p + 3]]);
                    let ex = u16::from_be_bytes([payload[p + 4], payload[p + 5]]);
                    let ey = u16::from_be_bytes([payload[p + 6], payload[p + 7]]);
                    let q = payload[p + 8];
                    efs.incipient_ridges.push(IncipientRidge {
                        start_x: sx,
                        start_y: sy,
                        end_x: ex,
                        end_y: ey,
                        quality: q,
                    });
                    p += 9;
                }
            }
            350 => {
                // Skin conditions: each = 1 (kind) + 2+2+2+2 (bbox) = 9 bytes
                let mut p = 0;
                while p + 9 <= payload.len() {
                    let kind = match payload[p] {
                        0 => SkinConditionKind::Crease,
                        1 => SkinConditionKind::Scar,
                        2 => SkinConditionKind::Wart,
                        _ => SkinConditionKind::OtherSkinCondition,
                    };
                    let x = u16::from_be_bytes([payload[p + 1], payload[p + 2]]);
                    let y = u16::from_be_bytes([payload[p + 3], payload[p + 4]]);
                    let w = u16::from_be_bytes([payload[p + 5], payload[p + 6]]);
                    let h = u16::from_be_bytes([payload[p + 7], payload[p + 8]]);
                    efs.skin_conditions.push(SkinCondition {
                        kind,
                        x,
                        y,
                        width: w,
                        height: h,
                    });
                    p += 9;
                }
            }
            360 => {
                // Ridge edge shapes: each = 1 (minutia_idx) + 4 (curvature f32 BE) + 4 (width f32 BE) = 9 bytes
                let mut p = 0;
                while p + 9 <= payload.len() {
                    let midx = payload[p] as usize;
                    let curv = f32::from_be_bytes([
                        payload[p + 1],
                        payload[p + 2],
                        payload[p + 3],
                        payload[p + 4],
                    ]);
                    let rw = f32::from_be_bytes([
                        payload[p + 5],
                        payload[p + 6],
                        payload[p + 7],
                        payload[p + 8],
                    ]);
                    efs.ridge_edge_shapes.push(RidgeEdgeShape {
                        minutia_idx: midx,
                        curvature: curv,
                        ridge_width: rw,
                    });
                    p += 9;
                }
            }
            // Skip unknown tags; they remain in the stream but are not decoded.
            _ => {}
        }
    }

    Ok(efs)
}
