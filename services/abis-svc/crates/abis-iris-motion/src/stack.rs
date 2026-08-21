//! Multi-frame stacking to improve signal-to-noise ratio.

use eel_abis_iris_segment::GrayImage;
use thiserror::Error;

/// Errors from frame stacking.
#[derive(Debug, Error)]
pub enum StackError {
    #[error("no frames provided")]
    EmptyFrames,
    #[error("frame size mismatch: expected {expected:?}, got {got:?}")]
    SizeMismatch {
        expected: (u32, u32),
        got: (u32, u32),
    },
}

/// Stacks multiple frames via mean pixel averaging.
pub struct FrameStacker;

impl FrameStacker {
    /// Average-stack a slice of grayscale frames.
    ///
    /// All frames must have the same dimensions.
    ///
    /// # Errors
    /// Returns `StackError::EmptyFrames` when `frames` is empty.
    /// Returns `StackError::SizeMismatch` when any frame differs from the first.
    pub fn stack(frames: &[GrayImage]) -> Result<GrayImage, StackError> {
        if frames.is_empty() {
            return Err(StackError::EmptyFrames);
        }

        let (w, h) = (frames[0].width, frames[0].height);
        for frame in frames.iter().skip(1) {
            if frame.width != w || frame.height != h {
                return Err(StackError::SizeMismatch {
                    expected: (w, h),
                    got: (frame.width, frame.height),
                });
            }
        }

        let n = frames.len() as u32;
        let pixel_count = (w * h) as usize;
        let mut sums = vec![0u32; pixel_count];

        for frame in frames {
            for (i, &px) in frame.pixels().iter().enumerate() {
                sums[i] += px as u32;
            }
        }

        let averaged: Vec<u8> = sums
            .into_iter()
            .map(|s| ((s + n / 2) / n).min(255) as u8)
            .collect();
        Ok(GrayImage::from_raw(w, h, averaged))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(w: u32, h: u32, fill: u8) -> GrayImage {
        GrayImage::from_raw(w, h, vec![fill; (w * h) as usize])
    }

    #[test]
    fn stack_empty_returns_error() {
        assert!(matches!(
            FrameStacker::stack(&[]),
            Err(StackError::EmptyFrames)
        ));
    }

    #[test]
    fn stack_single_frame_returns_same_content() {
        let img = make_image(4, 4, 100);
        let result = FrameStacker::stack(&[img]).expect("should stack");
        assert_eq!(result.pixels(), &[100u8; 16]);
    }

    #[test]
    fn stack_two_identical_frames_same_content() {
        let img1 = make_image(4, 4, 80);
        let img2 = make_image(4, 4, 80);
        let result = FrameStacker::stack(&[img1, img2]).expect("should stack");
        assert_eq!(result.pixels(), &[80u8; 16]);
    }

    #[test]
    fn stack_mismatched_sizes_returns_error() {
        let img1 = make_image(4, 4, 50);
        let img2 = make_image(8, 4, 50);
        assert!(matches!(
            FrameStacker::stack(&[img1, img2]),
            Err(StackError::SizeMismatch { .. })
        ));
    }

    #[test]
    fn stack_averages_pixel_values() {
        let img1 = make_image(2, 2, 100);
        let img2 = make_image(2, 2, 200);
        let result = FrameStacker::stack(&[img1, img2]).expect("should stack");
        for &px in result.pixels() {
            assert!(px == 150, "expected 150, got {px}");
        }
    }

    #[test]
    fn stack_clamps_to_u8() {
        let img1 = make_image(2, 2, 200);
        let img2 = make_image(2, 2, 200);
        let result = FrameStacker::stack(&[img1, img2]).expect("should stack");
        for &px in result.pixels() {
            assert!(px <= 255);
        }
    }

    #[test]
    fn stack_improves_snr_mean_closer_to_clean() {
        // Clean signal is 100. Add noise: [80, 120, 90, 110].
        // Mean of noisy frames should be 100, closer to clean than any single noisy frame.
        let clean = 100u8;
        let noisy: Vec<GrayImage> = [80u8, 120, 90, 110]
            .iter()
            .map(|&v| make_image(1, 1, v))
            .collect();
        let stacked = FrameStacker::stack(&noisy).expect("should stack");
        let stacked_val = stacked.pixels()[0] as i32;
        let clean_val = clean as i32;
        // Stacked mean should be within 5 of clean, better than worst noisy frame (|120-100|=20).
        let dist_stacked = (stacked_val - clean_val).abs();
        assert!(
            dist_stacked <= 5,
            "stacked {stacked_val} not close enough to clean {clean_val}"
        );
    }
}
