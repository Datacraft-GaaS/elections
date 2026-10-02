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

//! Rolling window baseline management.

use chrono::{DateTime, Utc};

use crate::domain::{DriftWindow, FeatureDistribution};

pub fn window_start(window: &DriftWindow) -> DateTime<Utc> {
    Utc::now() - chrono::Duration::hours(window.hours())
}

pub fn aggregate_distributions(dists: &[FeatureDistribution]) -> Option<FeatureDistribution> {
    let first = dists.first()?;
    let n_bins = first.bin_counts.len();
    let mut totals = vec![0u64; n_bins];
    let mut n_total = 0u64;
    for d in dists {
        if d.bin_counts.len() != n_bins {
            continue;
        }
        for (i, &c) in d.bin_counts.iter().enumerate() {
            totals[i] += c;
        }
        n_total += d.n_total;
    }
    Some(FeatureDistribution {
        feature_name: first.feature_name.clone(),
        bin_counts: totals,
        n_total,
        captured_at: Utc::now(),
    })
}
