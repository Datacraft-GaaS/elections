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

use crate::domain::GaitTemplate;

#[derive(Debug, Clone)]
pub struct GaitQuality {
    pub cycle_count_score: f32,
    pub speed_score: f32,
    pub overall: f32,
}

pub fn score_quality(template: &GaitTemplate) -> GaitQuality {
    let n = template.cycles.len();

    let cycle_count_score = if n == 0 {
        0.0
    } else if n >= 3 && n <= 8 {
        100.0
    } else if n < 3 {
        (n as f32 / 3.0) * 100.0
    } else {
        // More than 8: mild penalty, still useful.
        let excess = (n - 8) as f32;
        (100.0 - excess * 5.0).max(0.0)
    };

    let speed = template.walking_speed_ms;
    let speed_score = if speed >= 1.0 && speed <= 2.0 {
        100.0
    } else if speed < 1.0 {
        (speed / 1.0) * 100.0
    } else {
        // Above 2.0 m/s.
        let excess = speed - 2.0;
        (100.0 - excess * 50.0).max(0.0)
    };

    let overall = if n == 0 {
        0.0
    } else {
        (cycle_count_score + speed_score) / 2.0
    };

    GaitQuality {
        cycle_count_score,
        speed_score,
        overall,
    }
}
