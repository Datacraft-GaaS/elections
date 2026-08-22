//! Mouse trajectory: velocity, acceleration, jerk, curvature.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    pub x: f64,
    pub y: f64,
    pub timestamp_ms: u64,
    pub button_state: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseFeatures {
    pub mean_velocity: f64,
    pub std_velocity: f64,
    pub mean_acceleration: f64,
    pub mean_jerk: f64,
    pub mean_curvature: f64,
}

pub struct MouseExtractor {
    window: VecDeque<MouseEvent>,
    window_size: usize,
}

impl MouseExtractor {
    pub fn new(window_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    pub fn push(&mut self, event: MouseEvent) {
        if self.window.len() == self.window_size {
            self.window.pop_front();
        }
        self.window.push_back(event);
    }

    /// Returns None when fewer than 3 events are in the window.
    pub fn features(&self) -> Option<MouseFeatures> {
        if self.window.len() < 3 {
            return None;
        }
        let events: Vec<&MouseEvent> = self.window.iter().collect();
        let n = events.len();

        // Velocities between consecutive points.
        let mut velocities: Vec<f64> = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            let dt = (events[i + 1].timestamp_ms as f64 - events[i].timestamp_ms as f64).max(1.0);
            let dx = events[i + 1].x - events[i].x;
            let dy = events[i + 1].y - events[i].y;
            let dist = (dx * dx + dy * dy).sqrt();
            velocities.push(dist / dt);
        }

        // Accelerations: delta-velocity / dt.
        let mut accelerations: Vec<f64> = Vec::with_capacity(velocities.len().saturating_sub(1));
        for i in 0..velocities.len().saturating_sub(1) {
            let dt =
                (events[i + 2].timestamp_ms as f64 - events[i + 1].timestamp_ms as f64).max(1.0);
            accelerations.push((velocities[i + 1] - velocities[i]) / dt);
        }

        // Jerks: delta-acceleration / dt.
        let mut jerks: Vec<f64> = Vec::with_capacity(accelerations.len().saturating_sub(1));
        for i in 0..accelerations.len().saturating_sub(1) {
            let dt =
                (events[i + 3].timestamp_ms as f64 - events[i + 2].timestamp_ms as f64).max(1.0);
            jerks.push((accelerations[i + 1] - accelerations[i]) / dt);
        }

        // Curvature for each triplet of consecutive points using cross-product formula.
        let mut curvatures: Vec<f64> = Vec::with_capacity(n.saturating_sub(2));
        for i in 0..n.saturating_sub(2) {
            let (ax, ay) = (events[i].x, events[i].y);
            let (bx, by) = (events[i + 1].x, events[i + 1].y);
            let (cx, cy) = (events[i + 2].x, events[i + 2].y);
            let ab = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
            let bc = ((cx - bx).powi(2) + (cy - by).powi(2)).sqrt();
            let ac = ((cx - ax).powi(2) + (cy - ay).powi(2)).sqrt();
            // Cross product magnitude: |(b-a) x (c-a)|
            let cross = ((bx - ax) * (cy - ay) - (by - ay) * (cx - ax)).abs();
            // Radius of circumscribed circle = (ab * bc * ac) / (2 * |cross|)
            // Curvature = 1 / radius = 2 * |cross| / (ab * bc * ac)
            let denom = ab * bc * ac;
            if denom > 1e-9 {
                curvatures.push(2.0 * cross / denom);
            } else {
                curvatures.push(0.0);
            }
        }

        let mean_velocity = Self::mean(&velocities);
        let std_velocity = Self::std(&velocities, mean_velocity);
        let mean_acceleration = Self::mean(&accelerations);
        let mean_jerk = Self::mean(&jerks);
        let mean_curvature = Self::mean(&curvatures);

        Some(MouseFeatures {
            mean_velocity,
            std_velocity,
            mean_acceleration,
            mean_jerk,
            mean_curvature,
        })
    }

    fn mean(vals: &[f64]) -> f64 {
        if vals.is_empty() {
            return 0.0;
        }
        vals.iter().sum::<f64>() / vals.len() as f64
    }

    fn std(vals: &[f64], mean: f64) -> f64 {
        if vals.len() < 2 {
            return 0.0;
        }
        let variance = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
        variance.sqrt()
    }
}
