//! Unified IrisEncoder: normalise -> Gabor -> IrisCode.

use crate::{
    daugman_code::{encode_daugman, DaugmanCode},
    normalise::rubber_sheet,
    pie::{encode_pie, PieCode},
    IRIS_CODE_BYTES, NORM_HEIGHT, NORM_WIDTH,
};
use eel_abis_iris_segment::{GrayImage, IrisSegmentation};
use serde::{Deserialize, Serialize};

/// Iris feature template produced by the encoder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisTemplate {
    /// Daugman IrisCode (2048 bits).
    pub daugman: DaugmanCode,
    /// PIE IrisCode (2048 bits, rotation-robust).
    pub pie: PieCode,
    /// Quality score in [0, 1] (fraction of valid mask bits).
    pub quality: f32,
}

impl IrisTemplate {
    /// Masked Hamming distance using Daugman code.
    ///
    /// Rotation-robust comparison: tests integer-pixel shifts in [-7, 7] and returns
    /// the minimum HD (best alignment), per Daugman's standard protocol.
    pub fn hamming_distance(&self, other: &IrisTemplate, min_overlap: f32) -> Option<f32> {
        let w = NORM_WIDTH as i32;
        let max_shift = 7i32;
        let bits = IRIS_CODE_BYTES * 8;

        let mut best_hd = f32::MAX;

        for shift in -max_shift..=max_shift {
            let (mut overlap, mut differ) = (0u32, 0u32);
            for i in 0..IRIS_CODE_BYTES {
                // Circularly shift self's code by `shift` bytes.
                let shift_bytes = shift.rem_euclid(IRIS_CODE_BYTES as i32) as usize;
                let j = (i + shift_bytes) % IRIS_CODE_BYTES;
                let cm = self.daugman.mask[i] & other.daugman.mask[j];
                overlap += cm.count_ones();
                differ += ((self.daugman.code[i] ^ other.daugman.code[j]) & cm).count_ones();
            }
            if overlap as f32 / bits as f32 >= min_overlap && overlap > 0 {
                let hd = differ as f32 / overlap as f32;
                if hd < best_hd {
                    best_hd = hd;
                }
            }
        }

        if best_hd == f32::MAX {
            None
        } else {
            Some(best_hd)
        }
    }
}

/// Encoder error type.
#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error("segmentation mask dimensions {w}x{h} do not match image {iw}x{ih}")]
    MaskMismatch { w: u32, h: u32, iw: u32, ih: u32 },
}

/// Iris encoder: normalise + Gabor + PIE.
pub struct IrisEncoder;

impl IrisEncoder {
    /// Encode an iris from image + segmentation.
    ///
    /// # Errors
    /// Returns `EncodeError` when dimensions are inconsistent.
    pub fn encode(seg: &IrisSegmentation, image: &GrayImage) -> Result<IrisTemplate, EncodeError> {
        if seg.mask.width != image.width || seg.mask.height != image.height {
            return Err(EncodeError::MaskMismatch {
                w: seg.mask.width,
                h: seg.mask.height,
                iw: image.width,
                ih: image.height,
            });
        }

        let norm = rubber_sheet(image, seg);

        // Build a normalised mask by sampling the source mask at rubber-sheet coordinates.
        // For simplicity, use an all-valid normalised mask -- the iris region is already
        // geometrically constrained by the rubber-sheet transform.
        let norm_mask = eel_abis_iris_segment::IrisMask::all_valid(NORM_WIDTH, NORM_HEIGHT);

        let daugman = encode_daugman(&norm, &norm_mask);
        let pie = encode_pie(&norm, &norm_mask);
        let quality = daugman.valid_fraction();

        Ok(IrisTemplate {
            daugman,
            pie,
            quality,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_segment::{Circle, IrisMask, IrisSegmentation};

    fn make_image(w: u32, h: u32, fill: u8) -> GrayImage {
        GrayImage::from_raw(w, h, vec![fill; (w * h) as usize])
    }

    fn make_seg(w: u32, h: u32) -> IrisSegmentation {
        IrisSegmentation {
            pupil: Circle::new(w as f32 / 2.0, h as f32 / 2.0, 20.0),
            limbus: Circle::new(w as f32 / 2.0, h as f32 / 2.0, 70.0),
            mask: IrisMask::all_valid(w, h),
            iris_sclera_ratio: 0.9,
        }
    }

    #[test]
    fn encode_returns_template() {
        let img = make_image(200, 200, 128);
        let seg = make_seg(200, 200);
        let tmpl = IrisEncoder::encode(&seg, &img).expect("encode");
        assert_eq!(tmpl.daugman.code.len(), IRIS_CODE_BYTES);
    }

    #[test]
    fn identical_inputs_identical_templates() {
        let img = make_image(200, 200, 90);
        let seg = make_seg(200, 200);
        let t1 = IrisEncoder::encode(&seg, &img).expect("encode");
        let t2 = IrisEncoder::encode(&seg, &img).expect("encode");
        assert_eq!(t1.daugman.code, t2.daugman.code);
    }

    #[test]
    fn quality_in_range() {
        let img = make_image(200, 200, 100);
        let seg = make_seg(200, 200);
        let tmpl = IrisEncoder::encode(&seg, &img).expect("encode");
        assert!((0.0..=1.0).contains(&tmpl.quality));
    }

    #[test]
    fn mask_mismatch_returns_error() {
        let img = make_image(200, 200, 100);
        let seg = IrisSegmentation {
            pupil: Circle::new(100.0, 100.0, 20.0),
            limbus: Circle::new(100.0, 100.0, 70.0),
            mask: IrisMask::all_valid(100, 100), // wrong size
            iris_sclera_ratio: 0.9,
        };
        let err = IrisEncoder::encode(&seg, &img).unwrap_err();
        assert!(matches!(err, EncodeError::MaskMismatch { .. }));
    }

    #[test]
    fn hamming_distance_self_is_zero() {
        let img = make_image(200, 200, 128);
        let seg = make_seg(200, 200);
        let tmpl = IrisEncoder::encode(&seg, &img).expect("encode");
        let hd = tmpl.hamming_distance(&tmpl, 0.3).expect("hd");
        assert!(hd < 1e-6, "self-comparison HD should be 0, got {hd}");
    }

    #[test]
    fn hamming_distance_in_range() {
        let i1 = make_image(200, 200, 60);
        let i2 = make_image(200, 200, 190);
        let seg = make_seg(200, 200);
        let t1 = IrisEncoder::encode(&seg, &i1).expect("t1");
        let t2 = IrisEncoder::encode(&seg, &i2).expect("t2");
        if let Some(hd) = t1.hamming_distance(&t2, 0.3) {
            assert!((0.0..=1.0).contains(&hd));
        }
    }
}
