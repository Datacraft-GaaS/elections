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

use crate::domain::EarTemplate;

#[derive(Debug, Clone)]
pub struct EarQuality {
    pub visibility_score: f32,
    pub rotation_score: f32,
    pub overall: f32,
}

pub fn score_quality(template: &EarTemplate) -> EarQuality {
    let visibility_score = (template.visible_count() as f32 / 32.0) * 100.0;

    let abs_angle = template.rotation_angle_deg.abs();
    let rotation_score = if abs_angle < 15.0 {
        100.0
    } else {
        // Linear falloff from 100 at 15 deg to 0 at 90 deg.
        let t = (abs_angle - 15.0) / (90.0 - 15.0);
        (100.0 * (1.0 - t)).max(0.0)
    };

    let overall = (visibility_score + rotation_score) / 2.0;

    EarQuality {
        visibility_score,
        rotation_score,
        overall,
    }
}
