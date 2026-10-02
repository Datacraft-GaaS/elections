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

pub const DEFAULT_THRESHOLD: f32 = 0.55;

pub struct Score {
    pub raw: f32,
    pub threshold: f32,
    pub threshold_pass: bool,
    pub dtw_distance: f32,
}

pub fn match_signature(probe: &SignatureTemplate, gallery: &SignatureTemplate) -> Score {
    let p_strokes = split_strokes(probe);
    let g_strokes = split_strokes(gallery);

    let total_dist = if p_strokes.is_empty() || g_strokes.is_empty() {
        let pv: Vec<(f32, f32, f32)> = to_tuples(&probe.samples);
        let gv: Vec<(f32, f32, f32)> = to_tuples(&gallery.samples);
        dtw(&pv, &gv)
    } else {
        // Match stroke-by-stroke up to the minimum stroke count.
        let n = p_strokes.len().min(g_strokes.len());
        let mut total = 0.0f32;
        for i in 0..n {
            let pv = to_tuples(&p_strokes[i]);
            let gv = to_tuples(&g_strokes[i]);
            total += dtw(&pv, &gv);
        }
        total / n as f32
    };

    // Convert DTW distance to similarity. Scale = 1.0 maps distance 0.69 to ~0.5.
    let raw = (-total_dist).exp().clamp(0.0, 1.0);
    Score {
        raw,
        threshold: DEFAULT_THRESHOLD,
        threshold_pass: raw >= DEFAULT_THRESHOLD,
        dtw_distance: total_dist,
    }
}

fn to_tuples(samples: &[crate::domain::SignatureSample]) -> Vec<(f32, f32, f32)> {
    samples
        .iter()
        .map(|s| (s.pressure, s.velocity, s.acceleration))
        .collect()
}

fn split_strokes(template: &SignatureTemplate) -> Vec<Vec<crate::domain::SignatureSample>> {
    let mut strokes = Vec::new();
    let mut current: Vec<crate::domain::SignatureSample> = Vec::new();

    for s in &template.samples {
        if s.pen_up {
            if !current.is_empty() {
                strokes.push(std::mem::take(&mut current));
            }
        } else {
            current.push(s.clone());
        }
    }
    if !current.is_empty() {
        strokes.push(current);
    }
    strokes
}

pub fn dtw(probe: &[(f32, f32, f32)], gallery: &[(f32, f32, f32)]) -> f32 {
    let n = probe.len();
    let m = gallery.len();

    if n == 0 || m == 0 {
        return 0.0;
    }

    let mut dp = vec![vec![f32::INFINITY; m + 1]; n + 1];
    dp[0][0] = 0.0;

    for i in 1..=n {
        for j in 1..=m {
            let d = euclidean3(probe[i - 1], gallery[j - 1]);
            let prev = dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1]);
            dp[i][j] = d + prev;
        }
    }

    dp[n][m] / (n + m) as f32
}

fn euclidean3(a: (f32, f32, f32), b: (f32, f32, f32)) -> f32 {
    let dp = a.0 - b.0;
    let dv = a.1 - b.1;
    let da = a.2 - b.2;
    (dp * dp + dv * dv + da * da).sqrt()
}
