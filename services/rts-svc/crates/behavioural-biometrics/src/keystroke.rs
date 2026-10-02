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

//! Keystroke dynamics: dwell time, flight time, pressure histogram with rolling window.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key_code: u32,
    pub dwell_ms: f64,
    pub flight_ms: f64,
    pub pressure: f64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeFeatures {
    pub mean_dwell: f64,
    pub std_dwell: f64,
    pub mean_flight: f64,
    pub std_flight: f64,
    /// 10-bucket normalised pressure histogram; values sum to approximately 1.0.
    pub pressure_histogram: [f64; 10],
}

pub struct KeystrokeExtractor {
    window: VecDeque<KeyEvent>,
    window_size: usize,
}

impl KeystrokeExtractor {
    pub fn new(window_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    pub fn push(&mut self, event: KeyEvent) {
        if self.window.len() == self.window_size {
            self.window.pop_front();
        }
        self.window.push_back(event);
    }

    /// Returns None when fewer than 5 events are in the window.
    pub fn features(&self) -> Option<KeystrokeFeatures> {
        if self.window.len() < 5 {
            return None;
        }
        let dwells: Vec<f64> = self.window.iter().map(|e| e.dwell_ms).collect();
        let flights: Vec<f64> = self.window.iter().map(|e| e.flight_ms).collect();
        let pressures: Vec<f64> = self.window.iter().map(|e| e.pressure).collect();

        let mean_dwell = Self::mean(&dwells);
        let std_dwell = Self::std(&dwells, mean_dwell);
        let mean_flight = Self::mean(&flights);
        let std_flight = Self::std(&flights, mean_flight);
        let pressure_histogram = Self::build_pressure_histogram(&pressures);

        Some(KeystrokeFeatures {
            mean_dwell,
            std_dwell,
            mean_flight,
            std_flight,
            pressure_histogram,
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

    /// Buckets pressures in [0.0, 1.0] into 10 equal bins; normalises so the histogram sums to 1.0.
    fn build_pressure_histogram(vals: &[f64]) -> [f64; 10] {
        let mut counts = [0u32; 10];
        for &v in vals {
            let clamped = v.clamp(0.0, 1.0);
            let bucket = ((clamped * 10.0) as usize).min(9);
            counts[bucket] += 1;
        }
        let total = vals.len() as f64;
        let mut hist = [0.0f64; 10];
        if total > 0.0 {
            for (i, &c) in counts.iter().enumerate() {
                hist[i] = c as f64 / total;
            }
        }
        hist
    }
}
