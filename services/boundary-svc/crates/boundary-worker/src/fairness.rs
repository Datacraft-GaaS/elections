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

pub async fn run_once() -> Result<()> {
    // TODO(WP-2.4): query boundaries + voter counts from DB, compute metrics,
    // upsert fairness_metrics, emit FairnessMetricsComputedV1 event.
    info!("fairness metrics: no-op stub");
    Ok(())
}

/// Polsby-Popper compactness score in [0, 1]. Higher = more compact.
pub fn polsby_popper(area_m2: f64, perimeter_m: f64) -> f64 {
    if perimeter_m == 0.0 {
        return 0.0;
    }
    4.0 * std::f64::consts::PI * area_m2 / (perimeter_m * perimeter_m)
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
        assert!((pp - 1.0).abs() < 1e-10, "circle should score 1.0, got {pp}");
    }

    #[test]
    fn degenerate_perimeter_zero() {
        assert_eq!(polsby_popper(1.0, 0.0), 0.0);
    }
}
