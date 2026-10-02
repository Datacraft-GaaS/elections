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

//! Iris template: 2048-bit Daugman IrisCode + mask bitmap.

use super::{Modality, Template};

/// Number of bits in an IrisCode (256 bytes).
pub const IRIS_CODE_BITS: usize = 2048;
/// Byte length of IrisCode.
pub const IRIS_CODE_BYTES: usize = IRIS_CODE_BITS / 8;

/// Daugman-style iris template.
///
/// `code` holds the 2048-bit IrisCode; `mask` marks which bits are usable
/// (1 = usable, 0 = occluded/specular). Hamming distance is computed only
/// over positions where both probe and gallery masks are set.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IrisTemplate {
    /// 256-byte IrisCode bit array (stored as Vec for serde compat).
    pub code: Vec<u8>,
    /// 256-byte mask (1 = valid bit).
    pub mask: Vec<u8>,
    /// Iris quality in [0.0, 1.0] (ISO 29794-6: usable bit fraction).
    pub quality_score: f32,
}

impl IrisTemplate {
    /// Construct from raw code and mask byte arrays.
    pub fn new(code: [u8; IRIS_CODE_BYTES], mask: [u8; IRIS_CODE_BYTES]) -> Self {
        let mask_vec = mask.to_vec();
        let usable = mask_vec.iter().map(|b| b.count_ones() as f32).sum::<f32>();
        let quality_score = usable / IRIS_CODE_BITS as f32;
        Self {
            code: code.to_vec(),
            mask: mask_vec,
            quality_score,
        }
    }

    /// Construct from vecs (for tests / synthetic templates).
    pub fn from_vecs(code: Vec<u8>, mask: Vec<u8>) -> Self {
        assert_eq!(code.len(), IRIS_CODE_BYTES);
        assert_eq!(mask.len(), IRIS_CODE_BYTES);
        let usable = mask.iter().map(|b| b.count_ones() as f32).sum::<f32>();
        let quality_score = usable / IRIS_CODE_BITS as f32;
        Self {
            code,
            mask,
            quality_score,
        }
    }

    /// Masked Hamming distance between two IrisCodes.
    ///
    /// Returns `None` if the mask overlap fraction is below `min_overlap` (default 0.30).
    pub fn hamming_distance(&self, other: &Self, min_overlap: f32) -> Option<f32> {
        let len = self.code.len().min(other.code.len());
        let mut overlap_bits: u32 = 0;
        let mut differing_bits: u32 = 0;

        for i in 0..len {
            let combined_mask = self.mask[i] & other.mask[i];
            overlap_bits += combined_mask.count_ones();
            differing_bits += ((self.code[i] ^ other.code[i]) & combined_mask).count_ones();
        }

        let overlap_frac = overlap_bits as f32 / IRIS_CODE_BITS as f32;
        if overlap_frac < min_overlap {
            return None;
        }
        if overlap_bits == 0 {
            return None;
        }
        Some(differing_bits as f32 / overlap_bits as f32)
    }
}

impl Template for IrisTemplate {
    fn modality() -> Modality {
        Modality::Iris
    }

    fn quality(&self) -> f32 {
        self.quality_score
    }

    fn feature_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(IRIS_CODE_BYTES * 2);
        b.extend_from_slice(&self.code);
        b.extend_from_slice(&self.mask);
        b
    }

    fn normalise(&self) -> Self {
        // IrisCode normalisation is done at capture time (Daugman rubber-sheet);
        // no further transform needed here.
        self.clone()
    }
}
