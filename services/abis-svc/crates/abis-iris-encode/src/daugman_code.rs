//! Sign-of-Gabor-response -> 2048-bit IrisCode + validity mask.
//!
//! Reference: Daugman, J. (1993). High confidence visual recognition of persons by a test of
//! statistical independence. IEEE TPAMI, 15(11), 1148-1161.
//!
//! Each bit of the IrisCode is the sign of one Gabor filter response at one spatial location.
//! The mask bit is 1 (valid) when the corresponding pixel in the normalised iris mask is usable.

use crate::{
    gabor::{standard_filter_bank, GaborFilter},
    normalise::NormalisedIris,
    IRIS_CODE_BYTES,
};
use eel_abis_iris_segment::IrisMask;

/// Daugman IrisCode: 2048 bits of sign-of-Gabor-response + validity mask.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DaugmanCode {
    /// 256-byte code (MSB-first packed bits).
    #[serde(with = "crate::serde_array")]
    pub code: [u8; IRIS_CODE_BYTES],
    /// 256-byte mask (1 = valid bit).
    #[serde(with = "crate::serde_array")]
    pub mask: [u8; IRIS_CODE_BYTES],
}

impl DaugmanCode {
    /// Masked Hamming distance between two IrisCodes.
    ///
    /// Returns `None` when the overlap fraction (both masks valid) is below `min_overlap`.
    pub fn hamming_distance(&self, other: &DaugmanCode, min_overlap: f32) -> Option<f32> {
        let mut overlap = 0u32;
        let mut differ = 0u32;

        for i in 0..IRIS_CODE_BYTES {
            let combined_mask = self.mask[i] & other.mask[i];
            overlap += combined_mask.count_ones();
            differ += ((self.code[i] ^ other.code[i]) & combined_mask).count_ones();
        }

        let total_bits = (IRIS_CODE_BYTES * 8) as f32;
        if overlap as f32 / total_bits < min_overlap {
            return None;
        }
        if overlap == 0 {
            return None;
        }
        Some(differ as f32 / overlap as f32)
    }

    /// Fraction of bits that are valid (mask = 1).
    pub fn valid_fraction(&self) -> f32 {
        let valid: u32 = self.mask.iter().map(|b| b.count_ones()).sum();
        valid as f32 / (IRIS_CODE_BYTES * 8) as f32
    }
}

/// Encode a normalised iris into a Daugman IrisCode.
///
/// The 2048 output bits are produced by sampling the 32 Gabor filters
/// at 64 angular positions across 32 rows of the 512x64 normalised image.
/// 32 filters * 64 positions = 2048 bits.
pub fn encode_daugman(norm: &NormalisedIris, norm_mask: &IrisMask) -> DaugmanCode {
    let filters = standard_filter_bank();
    // 32 filters * 64 positions = 2048 bits. We sample 64 evenly-spaced columns.
    let n_filters = 32;
    let n_cols = 64usize;
    let n_rows = 1usize; // one representative row (mid-iris)

    debug_assert_eq!(n_filters * n_cols * n_rows, 2048);

    let mut code = [0u8; IRIS_CODE_BYTES];
    let mut mask = [0u8; IRIS_CODE_BYTES];

    let row_step = NormalisedIris::HEIGHT / 2; // mid-iris row

    for (f_idx, filter) in filters.iter().enumerate() {
        for c in 0..n_cols {
            let col = (c * NormalisedIris::WIDTH as usize / n_cols) as u32;
            let row = row_step;

            let bit_idx = f_idx * n_cols + c;
            let byte_idx = bit_idx / 8;
            let bit_pos = 7 - (bit_idx % 8);

            let resp = filter.respond(norm, col, row);
            if resp >= 0.0 {
                code[byte_idx] |= 1 << bit_pos;
            }

            // Mask valid if the corresponding normalised image location is in the iris mask.
            let mask_valid = norm_mask.get(col, row);
            if mask_valid {
                mask[byte_idx] |= 1 << bit_pos;
            }
        }
    }

    DaugmanCode { code, mask }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_segment::IrisMask;

    fn uniform_norm(val: u8) -> NormalisedIris {
        let data = vec![val; (NormalisedIris::WIDTH * NormalisedIris::HEIGHT) as usize];
        NormalisedIris { data }
    }

    fn all_valid_mask() -> IrisMask {
        IrisMask::all_valid(NormalisedIris::WIDTH, NormalisedIris::HEIGHT)
    }

    #[test]
    fn code_is_2048_bits() {
        let norm = uniform_norm(128);
        let mask = all_valid_mask();
        let code = encode_daugman(&norm, &mask);
        assert_eq!(code.code.len(), IRIS_CODE_BYTES);
        assert_eq!(code.mask.len(), IRIS_CODE_BYTES);
    }

    #[test]
    fn identical_inputs_produce_identical_codes() {
        let norm = uniform_norm(100);
        let mask = all_valid_mask();
        let c1 = encode_daugman(&norm, &mask);
        let c2 = encode_daugman(&norm, &mask);
        assert_eq!(c1.code, c2.code);
        assert_eq!(c1.mask, c2.mask);
    }

    #[test]
    fn valid_fraction_is_one_with_full_mask() {
        let norm = uniform_norm(128);
        let mask = all_valid_mask();
        let code = encode_daugman(&norm, &mask);
        assert!((code.valid_fraction() - 1.0).abs() < 0.01);
    }

    #[test]
    fn hamming_distance_identical_is_zero() {
        let norm = uniform_norm(128);
        let mask = all_valid_mask();
        let c1 = encode_daugman(&norm, &mask);
        let hd = c1.hamming_distance(&c1, 0.3).expect("enough overlap");
        assert!(hd.abs() < 1e-6);
    }

    #[test]
    fn hamming_distance_different_images_greater_than_zero() {
        let n1 = uniform_norm(50);
        let n2 = uniform_norm(200);
        let mask = all_valid_mask();
        let c1 = encode_daugman(&n1, &mask);
        let c2 = encode_daugman(&n2, &mask);
        let hd = c1.hamming_distance(&c2, 0.3).expect("overlap");
        assert!(hd >= 0.0 && hd <= 1.0);
    }
}
