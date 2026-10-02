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

//! Adaptive threshold suggestion when drift persists over 7 days.

use crate::domain::DriftReport;

/// Suggest a new PSI warning threshold as 1.5x the median observed PSI over
/// a sustained period. Requires at least 7 reports (one per day).
pub fn suggest_adaptive_threshold(reports: &[DriftReport]) -> Option<f64> {
    if reports.len() < 7 {
        return None;
    }
    let mut psi_vals: Vec<f64> = reports.iter().map(|r| r.psi).collect();
    psi_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = psi_vals.len() / 2;
    let median = if psi_vals.len() % 2 == 0 {
        (psi_vals[mid - 1] + psi_vals[mid]) / 2.0
    } else {
        psi_vals[mid]
    };
    Some(median * 1.5)
}
