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
