use crate::domain::{GaitCycle, GaitFrame, GaitTemplate, NUM_JOINTS};

pub const DEFAULT_THRESHOLD: f32 = 0.40;

#[derive(Debug, Clone)]
pub struct Score {
    pub raw: f32,
    pub dtw_cost: f32,
    pub cadence_diff: f32,
    pub threshold: f32,
    pub threshold_pass: bool,
}

pub fn frame_distance(a: &GaitFrame, b: &GaitFrame) -> f32 {
    let mse: f32 = a
        .joints
        .iter()
        .zip(b.joints.iter())
        .map(|(ja, jb)| {
            let dp = ja.pitch - jb.pitch;
            let dy = ja.yaw - jb.yaw;
            let dr = ja.roll - jb.roll;
            dp * dp + dy * dy + dr * dr
        })
        .sum::<f32>()
        / NUM_JOINTS as f32;
    mse
}

pub fn dtw_cycle(probe: &GaitCycle, gallery: &GaitCycle) -> f32 {
    let n = probe.frames.len();
    let m = gallery.frames.len();

    if n == 0 || m == 0 {
        return f32::INFINITY;
    }

    let mut dp = vec![vec![f32::INFINITY; m + 1]; n + 1];
    dp[0][0] = 0.0;

    for i in 1..=n {
        for j in 1..=m {
            let cost = frame_distance(&probe.frames[i - 1], &gallery.frames[j - 1]);
            let min_prev = dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1]);
            dp[i][j] = cost + min_prev;
        }
    }

    // Normalize by path length.
    dp[n][m] / (n + m) as f32
}

pub fn match_gait(probe: &GaitTemplate, gallery: &GaitTemplate) -> Score {
    let cadence_diff = (probe.cadence_steps_per_min - gallery.cadence_steps_per_min).abs();

    if probe.cycles.is_empty() || gallery.cycles.is_empty() {
        return Score {
            raw: 0.0,
            dtw_cost: f32::INFINITY,
            cadence_diff,
            threshold: DEFAULT_THRESHOLD,
            threshold_pass: false,
        };
    }

    // Find best cycle-pair match for each probe cycle.
    let mut total_cost = 0.0f32;
    let mut count = 0u32;

    for pc in &probe.cycles {
        let best = gallery
            .cycles
            .iter()
            .map(|gc| dtw_cycle(pc, gc))
            .fold(f32::INFINITY, f32::min);
        if best.is_finite() {
            total_cost += best;
            count += 1;
        }
    }

    if count == 0 {
        return Score {
            raw: 0.0,
            dtw_cost: f32::INFINITY,
            cadence_diff,
            threshold: DEFAULT_THRESHOLD,
            threshold_pass: false,
        };
    }

    let avg_cost = total_cost / count as f32;
    let base_score = (-avg_cost * 5.0_f32).exp();
    let cadence_factor = (-0.01 * cadence_diff).exp();
    let raw = (base_score * cadence_factor).clamp(0.0, 1.0);

    Score {
        raw,
        dtw_cost: avg_cost,
        cadence_diff,
        threshold: DEFAULT_THRESHOLD,
        threshold_pass: raw >= DEFAULT_THRESHOLD,
    }
}
