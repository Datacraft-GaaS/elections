use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Width and height of the periocular patch in pixels.
pub const PATCH_SIZE: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LbpHistogram {
    /// 256 bins, normalized to sum to 1.0.
    pub bins: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriocularTemplate {
    pub template_id: String,
    /// 128x128 grayscale patch (16384 bytes).
    pub patch: Vec<u8>,
    pub lbp_histogram: LbpHistogram,
    /// Laplacian variance (sharpness metric).
    pub sharpness: f32,
    /// Fraction of the patch that is occluded, in [0.0, 1.0].
    pub occlusion_ratio: f32,
    pub captured_at: DateTime<Utc>,
}
