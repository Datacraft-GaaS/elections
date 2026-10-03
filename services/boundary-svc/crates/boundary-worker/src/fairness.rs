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

//! Fairness metric computation worker.
//!
//! Computes three metrics per boundary kind:
//! - `compactness`: Polsby-Popper score (4π·Area / Perimeter²).
//! - `population_equality`: coefficient of variation of registered voters per
//!   ward within each constituency.
//! - `vra_analog`: share of minority-majority districts (proxy only; true VRA
//!   analysis requires census microdata not held by this service).

use anyhow::Result;
use tracing::info;

/// Single computation pass over every active boundary kind.
///
/// The full pass needs a PostgreSQL pool (boundaries table with PostGIS
/// area/perimeter) and a per-ward registered-voter store. Neither is wired in
/// this service yet: no crate in boundary-svc constructs a PgPool and the
/// schema holds no voter-count column. The pass therefore logs a skip and
/// returns Ok so the worker loop stays healthy; wiring the pool and the voter
/// store is the prerequisite for computing and upserting fairness_metrics.
pub async fn run_once() -> Result<()> {
    info!("fairness metrics: skipped, boundary pool and voter store not wired");
    Ok(())
}

/// Polsby-Popper compactness score in [0, 1]. Higher = more compact.
pub fn polsby_popper(area_m2: f64, perimeter_m: f64) -> f64 {
    if perimeter_m == 0.0 {
        return 0.0;
    }
    4.0 * std::f64::consts::PI * area_m2 / (perimeter_m * perimeter_m)
}

/// Coefficient of variation of registered voters across wards, expressed as
/// population standard deviation divided by mean. 0.0 means perfectly equal
/// wards. Returns 0.0 for empty input, a single ward, or an all-zero mean.
pub fn population_equality(voter_counts: &[u64]) -> f64 {
    if voter_counts.len() < 2 {
        return 0.0;
    }
    let n = voter_counts.len() as f64;
    let mean = voter_counts.iter().sum::<u64>() as f64 / n;
    if mean == 0.0 {
        return 0.0;
    }
    let var = voter_counts
        .iter()
        .map(|c| {
            let d = *c as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / n;
    var.sqrt() / mean
}

/// Share of minority-majority districts among all districts, in [0, 1].
/// Proxy metric only; true VRA analysis requires census microdata not held
/// by this service. Returns 0.0 when there are no districts.
pub fn vra_analog(minority_majority_districts: usize, total_districts: usize) -> f64 {
    if total_districts == 0 {
        return 0.0;
    }
    (minority_majority_districts.min(total_districts) as f64) / total_districts as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_is_most_compact() {
        // A circle with r = 1 has area = π, perimeter = 2π.
        // PP = 4π * π / (2π)² = 4π² / 4π² = 1.0
        let r = 1.0_f64;
        let area = std::f64::consts::PI * r * r;
        let perim = 2.0 * std::f64::consts::PI * r;
        let pp = polsby_popper(area, perim);
        assert!(
            (pp - 1.0).abs() < 1e-10,
            "circle should score 1.0, got {pp}"
        );
    }

    #[test]
    fn degenerate_perimeter_zero() {
        assert_eq!(polsby_popper(1.0, 0.0), 0.0);
    }
    #[test]
    fn uniform_wards_have_zero_cv() {
        assert_eq!(population_equality(&[1000, 1000, 1000]), 0.0);
    }

    #[test]
    fn known_cv_value() {
        // [0, 2, 4]: mean 2, population stddev sqrt(8/3), CV = sqrt(8/3)/2.
        let expected = (8.0_f64 / 3.0).sqrt() / 2.0;
        let cv = population_equality(&[0, 2, 4]);
        assert!((cv - expected).abs() < 1e-12, "got {cv}");
    }

    #[test]
    fn cv_degenerate_inputs() {
        assert_eq!(population_equality(&[]), 0.0);
        assert_eq!(population_equality(&[42]), 0.0);
        assert_eq!(population_equality(&[0, 0, 0]), 0.0);
    }

    #[test]
    fn vra_analog_share() {
        assert!((vra_analog(3, 10) - 0.3).abs() < 1e-12);
        assert_eq!(vra_analog(0, 0), 0.0);
        assert_eq!(vra_analog(5, 0), 0.0);
        assert!((vra_analog(12, 10) - 1.0).abs() < 1e-12);
    }
}
