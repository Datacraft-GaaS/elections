use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const NUM_LANDMARKS: usize = 32;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarTemplate {
    pub template_id: String,
    pub landmarks: [Point2D; NUM_LANDMARKS],
    pub visibility_mask: u32,
    pub rotation_angle_deg: f32,
    pub captured_at: DateTime<Utc>,
}

impl EarTemplate {
    pub fn landmark_visible(&self, idx: usize) -> bool {
        idx < 32 && (self.visibility_mask >> idx) & 1 == 1
    }

    pub fn visible_count(&self) -> u32 {
        self.visibility_mask.count_ones()
    }
}
