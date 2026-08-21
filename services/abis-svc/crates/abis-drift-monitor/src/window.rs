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
