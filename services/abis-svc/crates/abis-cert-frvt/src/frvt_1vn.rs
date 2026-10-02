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

//! FRVT 1:N Identification track submission.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frvt1vNReport {
    pub fpir_at_fnir_1e2: f64,
    pub rank1_id_rate: f64,
    pub gallery_size: usize,
    pub probe_count: u64,
}

pub struct Frvt1vNSubmission {
    algorithm_id: String,
}

impl Frvt1vNSubmission {
    pub fn new(algorithm_id: &str) -> Self {
        Self {
            algorithm_id: algorithm_id.to_owned(),
        }
    }

    /// Runs 1:N identification over the gallery and returns a report card.
    pub fn run(&self, gallery_size: usize, probe_count: u64) -> Frvt1vNReport {
        // Stub: return fixed rates proportional to gallery size.
        let rank1 = if gallery_size == 0 {
            0.0
        } else {
            0.98 - (gallery_size as f64).log10() * 0.01
        };
        Frvt1vNReport {
            fpir_at_fnir_1e2: 0.001,
            rank1_id_rate: rank1.clamp(0.0, 1.0),
            gallery_size,
            probe_count,
        }
    }

    pub fn algorithm_id(&self) -> &str {
        &self.algorithm_id
    }
}
