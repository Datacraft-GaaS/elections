use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureSample {
    pub timestamp_ms: u32,
    /// Pen pressure, normalized to [0.0, 1.0].
    pub pressure: f32,
    /// Pen velocity in pixels/ms.
    pub velocity: f32,
    /// Pen acceleration in pixels/ms^2.
    pub acceleration: f32,
    pub pen_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureTemplate {
    pub template_id: String,
    pub capture_device: Option<String>,
    pub samples: Vec<SignatureSample>,
    pub pen_up_count: u16,
    pub sample_rate_hz: u16,
    pub total_duration_ms: u32,
    pub stroke_count: u16,
    pub captured_at: DateTime<Utc>,
}
