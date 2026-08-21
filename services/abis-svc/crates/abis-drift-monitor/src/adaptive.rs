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
