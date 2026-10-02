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

use crate::domain::{EarTemplate, Point2D};

pub const DEFAULT_THRESHOLD: f32 = 0.50;

#[derive(Debug, Clone)]
pub struct Score {
    pub raw: f32,
    pub procrustes_dist: f32,
    pub used_landmarks: u32,
    pub threshold: f32,
    pub threshold_pass: bool,
}

pub fn match_ear(probe: &EarTemplate, gallery: &EarTemplate) -> Score {
    let shared_mask = probe.visibility_mask & gallery.visibility_mask;
    let used_landmarks = shared_mask.count_ones();

    if used_landmarks < 3 {
        return Score {
            raw: 0.0,
            procrustes_dist: f32::INFINITY,
            used_landmarks,
            threshold: DEFAULT_THRESHOLD,
            threshold_pass: false,
        };
    }

    let probe_pts: Vec<Point2D> = (0..32)
        .filter(|&i| (shared_mask >> i) & 1 == 1)
        .map(|i| probe.landmarks[i])
        .collect();

    let gallery_pts: Vec<Point2D> = (0..32)
        .filter(|&i| (shared_mask >> i) & 1 == 1)
        .map(|i| gallery.landmarks[i])
        .collect();

    let dist = procrustes_distance(&probe_pts, &gallery_pts);
    let raw = (-dist * 5.0_f32).exp();

    Score {
        raw,
        procrustes_dist: dist,
        used_landmarks,
        threshold: DEFAULT_THRESHOLD,
        threshold_pass: raw >= DEFAULT_THRESHOLD,
    }
}

fn centroid(pts: &[Point2D]) -> Point2D {
    let n = pts.len() as f32;
    let sx: f32 = pts.iter().map(|p| p.x).sum();
    let sy: f32 = pts.iter().map(|p| p.y).sum();
    Point2D {
        x: sx / n,
        y: sy / n,
    }
}

fn scale(pts: &[Point2D], center: Point2D) -> f32 {
    let rms = pts
        .iter()
        .map(|p| {
            let dx = p.x - center.x;
            let dy = p.y - center.y;
            dx * dx + dy * dy
        })
        .sum::<f32>()
        / pts.len() as f32;
    rms.sqrt().max(f32::EPSILON)
}

fn procrustes_distance(probe: &[Point2D], gallery: &[Point2D]) -> f32 {
    let c_p = centroid(probe);
    let c_g = centroid(gallery);

    let s_p = scale(probe, c_p);
    let s_g = scale(gallery, c_g);

    // Normalise.
    let np: Vec<Point2D> = probe
        .iter()
        .map(|p| Point2D {
            x: (p.x - c_p.x) / s_p,
            y: (p.y - c_p.y) / s_p,
        })
        .collect();

    let ng: Vec<Point2D> = gallery
        .iter()
        .map(|p| Point2D {
            x: (p.x - c_g.x) / s_g,
            y: (p.y - c_g.y) / s_g,
        })
        .collect();

    // Optimal rotation using atan2 of cross-product / dot-product sum.
    let num: f32 = np
        .iter()
        .zip(ng.iter())
        .map(|(p, g)| p.x * g.y - p.y * g.x)
        .sum();
    let den: f32 = np
        .iter()
        .zip(ng.iter())
        .map(|(p, g)| p.x * g.x + p.y * g.y)
        .sum();
    let theta = num.atan2(den);

    let cos_t = theta.cos();
    let sin_t = theta.sin();

    let msd: f32 = np
        .iter()
        .zip(ng.iter())
        .map(|(p, g)| {
            let rx = p.x * cos_t - p.y * sin_t;
            let ry = p.x * sin_t + p.y * cos_t;
            let dx = rx - g.x;
            let dy = ry - g.y;
            dx * dx + dy * dy
        })
        .sum::<f32>()
        / np.len() as f32;

    msd.sqrt()
}
