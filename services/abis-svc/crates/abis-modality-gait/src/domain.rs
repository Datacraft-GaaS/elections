use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const NUM_JOINTS: usize = 17;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JointAngle {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaitFrame {
    pub frame_idx: u32,
    pub joints: [JointAngle; NUM_JOINTS],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaitCycle {
    pub frames: Vec<GaitFrame>,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaitTemplate {
    pub template_id: String,
    pub cycles: Vec<GaitCycle>,
    pub cadence_steps_per_min: f32,
    pub walking_speed_ms: f32,
    pub captured_at: DateTime<Utc>,
}
