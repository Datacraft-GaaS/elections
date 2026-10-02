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
