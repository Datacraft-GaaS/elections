//! Prickle Interpolated Encoding (PIE) -- rotation-robust IrisCode variant.
//!
//! PIE encodes the iris at multiple rotational offsets and takes a majority vote,
//! making the code robust to small angular misalignments that shift the iris texture.
//!
//! Reference: internal variant; conceptually related to the shift-invariant comparison
//! described in: Ma, L., et al. (2004). Efficient iris recognition by characterizing key local
//! variations. IEEE TIP, 13(6), 739-750.

use crate::{
    daugman_code::{encode_daugman, DaugmanCode},
    normalise::NormalisedIris,
    IRIS_CODE_BYTES,
};
use eel_abis_iris_segment::IrisMask;

/// PIE template: majority-vote IrisCode from multiple angular offsets.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PieCode {
    /// Consensus IrisCode (majority-vote across offsets).
    #[serde(with = "crate::serde_array")]
    pub code: [u8; IRIS_CODE_BYTES],
    /// Confidence mask: 1 where the majority was decisive (>60% agreement).
    #[serde(with = "crate::serde_array")]
    pub mask: [u8; IRIS_CODE_BYTES],
}

/// Number of rotational offsets used in PIE encoding.
const PIE_OFFSETS: usize = 7;
/// Maximum angular shift in pixels (at most 3 pixels = ~2 degrees at 512 col width).
const PIE_MAX_SHIFT: i32 = 3;

/// Encode a normalised iris using PIE.
///
/// Produces a single IrisCode by majority-voting sign(Gabor) across `PIE_OFFSETS` angular shifts.
pub fn encode_pie(norm: &NormalisedIris, norm_mask: &IrisMask) -> PieCode {
    let shifts: Vec<i32> = (-PIE_MAX_SHIFT..=PIE_MAX_SHIFT).collect();
    debug_assert_eq!(shifts.len(), PIE_OFFSETS);

    let w = NormalisedIris::WIDTH as usize;

    // Collect vote tallies per bit (positive = how many offsets voted '1').
    let mut votes_one = vec![0i32; IRIS_CODE_BYTES * 8];
    let mut votes_valid = vec![0i32; IRIS_CODE_BYTES * 8];

    for &shift in &shifts {
        // Build a shifted normalised image.
        let shifted = shift_normalised(norm, shift);
        let shifted_mask = shift_mask(norm_mask, shift);
        let code = encode_daugman(&shifted, &shifted_mask);

        for byte_idx in 0..IRIS_CODE_BYTES {
            for bit_pos in 0..8 {
                let bit_idx = byte_idx * 8 + bit_pos;
                let bit = (code.code[byte_idx] >> (7 - bit_pos)) & 1;
                let valid = (code.mask[byte_idx] >> (7 - bit_pos)) & 1;
                if bit == 1 {
                    votes_one[bit_idx] += 1;
                }
                votes_valid[bit_idx] += valid as i32;
            }
        }
    }

    let total = shifts.len() as i32;
    let threshold = total / 2;

    let mut code = [0u8; IRIS_CODE_BYTES];
    let mut mask = [0u8; IRIS_CODE_BYTES];

    for bit_idx in 0..IRIS_CODE_BYTES * 8 {
        let byte_idx = bit_idx / 8;
        let bit_pos = 7 - (bit_idx % 8);
        // Majority vote for code bit.
        if votes_one[bit_idx] > threshold {
            code[byte_idx] |= 1 << bit_pos;
        }
        // Confidence mask: valid in majority of offsets.
        if votes_valid[bit_idx] > threshold {
            mask[byte_idx] |= 1 << bit_pos;
        }
    }

    PieCode { code, mask }
}

/// Circularly shift the normalised iris horizontally by `shift` columns.
fn shift_normalised(norm: &NormalisedIris, shift: i32) -> NormalisedIris {
    let w = NormalisedIris::WIDTH as usize;
    let h = NormalisedIris::HEIGHT as usize;
    let mut data = vec![0u8; w * h];

    for row in 0..h {
        for col in 0..w {
            let src_col = ((col as i32 + shift).rem_euclid(w as i32)) as usize;
            data[row * w + col] = norm.data[row * w + src_col];
        }
    }
    NormalisedIris { data }
}

/// Circularly shift the normalised mask horizontally by `shift` columns.
fn shift_mask(mask: &IrisMask, shift: i32) -> IrisMask {
    let w = mask.width;
    let h = mask.height;
    let mut new_mask = IrisMask::zeroed(w, h);

    for row in 0..h {
        for col in 0..w {
            let src_col = ((col as i32 + shift).rem_euclid(w as i32)) as u32;
            new_mask.set(col, row, mask.get(src_col, row));
        }
    }
    new_mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_segment::IrisMask;

    fn uniform_norm(val: u8) -> NormalisedIris {
        NormalisedIris {
            data: vec![val; (NormalisedIris::WIDTH * NormalisedIris::HEIGHT) as usize],
        }
    }

    #[test]
    fn pie_code_correct_size() {
        let norm = uniform_norm(128);
        let mask = IrisMask::all_valid(NormalisedIris::WIDTH, NormalisedIris::HEIGHT);
        let pie = encode_pie(&norm, &mask);
        assert_eq!(pie.code.len(), IRIS_CODE_BYTES);
        assert_eq!(pie.mask.len(), IRIS_CODE_BYTES);
    }

    #[test]
    fn pie_identical_inputs_identical_codes() {
        let norm = uniform_norm(90);
        let mask = IrisMask::all_valid(NormalisedIris::WIDTH, NormalisedIris::HEIGHT);
        let p1 = encode_pie(&norm, &mask);
        let p2 = encode_pie(&norm, &mask);
        assert_eq!(p1.code, p2.code);
    }

    #[test]
    fn shift_normalised_wraps_correctly() {
        let mut data = vec![0u8; (NormalisedIris::WIDTH * NormalisedIris::HEIGHT) as usize];
        // Set column 0 of row 0 to 255.
        data[0] = 255;
        let norm = NormalisedIris { data };
        let shifted = shift_normalised(&norm, -1);
        // Column 511 of row 0 should now be 255.
        assert_eq!(shifted.data[NormalisedIris::WIDTH as usize - 1], 255);
    }

    #[test]
    fn pie_uniform_image_has_consistent_bits() {
        // A uniform image should produce the same code with and without shifts.
        let norm = uniform_norm(128);
        let mask = IrisMask::all_valid(NormalisedIris::WIDTH, NormalisedIris::HEIGHT);
        let pie = encode_pie(&norm, &mask);
        let daugman = encode_daugman(&norm, &mask);
        // Both should agree (majority = all shifted codes agree).
        assert_eq!(pie.code, daugman.code);
    }
}
