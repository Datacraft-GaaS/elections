//! ISO/IEC 29794-5 face image quality assessment.
//!
//! Reference: ISO/IEC 29794-5:2010 "Information technology -- Biometric
//! sample quality -- Part 5: Face image data." (and 2023 revision draft).
//!
//! Each metric is independently computable and returns a normalised score
//! in [0.0, 1.0] where 1.0 is the highest quality.

pub mod metrics;
pub mod policy;
pub mod report;

pub use metrics::{
    sharpness, resolution_interocular, pose_angles, illumination,
    uniformity, occlusion_coverage, expression_neutrality, gaze_offset,
};
pub use policy::{QualityPolicy, PolicyViolation};
pub use report::QualityReport;

use thiserror::Error;

/// Quality computation error.
#[derive(Debug, Error)]
pub enum QualityError {
    #[error("image dimensions are too small (min 32x32)")]
    ImageTooSmall,
    #[error("landmark positions are outside image bounds")]
    LandmarksOutOfBounds,
    #[error("interocular distance is zero (landmarks are coincident)")]
    ZeroInterocularDistance,
}

/// Pixel-space face landmark positions used by quality metrics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityLandmarks {
    pub left_eye:    (f32, f32),
    pub right_eye:   (f32, f32),
    pub nose:        (f32, f32),
    pub left_mouth:  (f32, f32),
    pub right_mouth: (f32, f32),
    /// Optional: pupil centres for gaze estimation.
    pub left_pupil:  Option<(f32, f32)>,
    pub right_pupil: Option<(f32, f32)>,
}

impl QualityLandmarks {
    /// Interocular distance in pixels.
    pub fn interocular_distance(&self) -> f32 {
        let dx = self.right_eye.0 - self.left_eye.0;
        let dy = self.right_eye.1 - self.left_eye.1;
        (dx * dx + dy * dy).sqrt()
    }

    /// Eye midpoint.
    pub fn eye_midpoint(&self) -> (f32, f32) {
        (
            (self.left_eye.0 + self.right_eye.0) / 2.0,
            (self.left_eye.1 + self.right_eye.1) / 2.0,
        )
    }
}
