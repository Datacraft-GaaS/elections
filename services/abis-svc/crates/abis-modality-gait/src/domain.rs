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
