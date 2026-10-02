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

use crate::domain::SignatureTemplate;

pub struct SignatureQuality {
    /// 100 at 100 Hz, proportionally lower below that.
    pub sample_rate_score: f32,
    /// 100 for durations in [1000, 5000] ms, 0 outside [100, 10000].
    pub duration_score: f32,
    /// 100 for 3-8 strokes, lower outside that range.
    pub stroke_score: f32,
    /// Weighted average of the three sub-scores.
    pub overall: f32,
}

pub fn score_quality(template: &SignatureTemplate) -> SignatureQuality {
    let sample_rate_score = ((template.sample_rate_hz as f32 / 100.0) * 100.0).clamp(0.0, 100.0);

    let dur_ms = template.total_duration_ms;
    let duration_score = if dur_ms >= 1000 && dur_ms <= 5000 {
        100.0
    } else if dur_ms < 100 || dur_ms > 10_000 {
        0.0
    } else if dur_ms < 1000 {
        ((dur_ms as f32 - 100.0) / 900.0) * 100.0
    } else {
        ((10_000.0 - dur_ms as f32) / 5_000.0) * 100.0
    };

    let strokes = template.stroke_count;
    let stroke_score = if strokes == 0 {
        0.0
    } else if strokes >= 3 && strokes <= 8 {
        100.0
    } else if strokes < 3 {
        (strokes as f32 / 3.0) * 100.0
    } else {
        // strokes > 8: penalize linearly up to 20 strokes.
        ((20.0 - strokes as f32) / 12.0 * 100.0).clamp(0.0, 100.0)
    };

    let overall =
        (sample_rate_score * 0.4 + duration_score * 0.3 + stroke_score * 0.3).clamp(0.0, 100.0);

    SignatureQuality {
        sample_rate_score,
        duration_score,
        stroke_score,
        overall,
    }
}
