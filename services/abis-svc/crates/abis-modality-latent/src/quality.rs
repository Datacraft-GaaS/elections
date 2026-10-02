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

//! Latent print quality scoring.

use crate::domain::LatentTemplate;

/// Compute a quality score in [0.0, 100.0] for a latent template.
///
/// Score = usable_ridge_pct * (1.0 - distortion_score) * minutiae_density,
/// where minutiae_density = clamp(count / 20.0, 0, 1).
pub fn score_quality(template: &LatentTemplate) -> f32 {
    if template.minutiae.is_empty() {
        return 0.0;
    }

    let minutiae_density = (template.minutiae.len() as f32 / 20.0).min(1.0);
    let clarity = (1.0 - template.distortion_score).max(0.0);
    (template.usable_ridge_pct * clarity * minutiae_density * 100.0).min(100.0)
}
