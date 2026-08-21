//! Statistical drift detectors: PSI, KS, JS divergence, Wasserstein distance.

use chrono::Utc;

use crate::domain::{DriftReport, DriftSeverity, DriftWindow, FeatureDistribution};
use crate::error::DriftError;

/// Population Stability Index: PSI = sum((P_i - Q_i) * ln(P_i / Q_i)).
pub fn psi(p: &[f64], q: &[f64]) -> Result<f64, DriftError> {
    if p.is_empty() || q.is_empty() {
        return Err(DriftError::EmptyDistribution);
    }
    if p.len() != q.len() {
        return Err(DriftError::BinCountMismatch {
            expected: p.len(),
            got: q.len(),
        });
    }
    let val: f64 = p
        .iter()
        .zip(q.iter())
        .map(|(&pi, &qi)| {
            let pi = pi.max(1e-10);
            let qi = qi.max(1e-10);
            (pi - qi) * (pi / qi).ln()
        })
        .sum();
    Ok(val.abs())
}

/// Kolmogorov-Smirnov statistic: max absolute CDF difference.
pub fn ks_statistic(p: &[f64], q: &[f64]) -> Result<f64, DriftError> {
    if p.is_empty() || q.is_empty() {
        return Err(DriftError::EmptyDistribution);
    }
    if p.len() != q.len() {
        return Err(DriftError::BinCountMismatch {
            expected: p.len(),
            got: q.len(),
        });
    }
    let mut cdf_p = 0.0_f64;
    let mut cdf_q = 0.0_f64;
    let mut max_diff = 0.0_f64;
    for (&pi, &qi) in p.iter().zip(q.iter()) {
        cdf_p += pi;
        cdf_q += qi;
        max_diff = max_diff.max((cdf_p - cdf_q).abs());
    }
    Ok(max_diff)
}

/// Jensen-Shannon divergence: symmetric, bounded [0, ln2].
pub fn js_divergence(p: &[f64], q: &[f64]) -> Result<f64, DriftError> {
    if p.is_empty() || q.is_empty() {
        return Err(DriftError::EmptyDistribution);
    }
    if p.len() != q.len() {
        return Err(DriftError::BinCountMismatch {
            expected: p.len(),
            got: q.len(),
        });
    }
    let kl = |a: &[f64], b: &[f64]| -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(&ai, &bi)| {
                let ai = ai.max(1e-10);
                let bi = bi.max(1e-10);
                ai * (ai / bi).ln()
            })
            .sum::<f64>()
    };
    let m: Vec<f64> = p
        .iter()
        .zip(q.iter())
        .map(|(&pi, &qi)| (pi + qi) / 2.0)
        .collect();
    Ok(0.5 * kl(p, &m) + 0.5 * kl(q, &m))
}

/// Wasserstein distance (1D earth mover's via CDF integral).
pub fn wasserstein_distance(p: &[f64], q: &[f64]) -> Result<f64, DriftError> {
    if p.is_empty() || q.is_empty() {
        return Err(DriftError::EmptyDistribution);
    }
    if p.len() != q.len() {
        return Err(DriftError::BinCountMismatch {
            expected: p.len(),
            got: q.len(),
        });
    }
    let mut cdf_p = 0.0_f64;
    let mut cdf_q = 0.0_f64;
    let mut dist = 0.0_f64;
    for (&pi, &qi) in p.iter().zip(q.iter()) {
        cdf_p += pi;
        cdf_q += qi;
        dist += (cdf_p - cdf_q).abs();
    }
    Ok(dist / p.len() as f64)
}

fn severity_from_psi(psi_val: f64) -> DriftSeverity {
    if psi_val < 0.1 {
        DriftSeverity::Info
    } else if psi_val < 0.25 {
        DriftSeverity::Warning
    } else {
        DriftSeverity::Critical
    }
}

pub struct DriftDetector {
    pub psi_warning: f64,
    pub psi_critical: f64,
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self {
            psi_warning: 0.1,
            psi_critical: 0.25,
        }
    }
}

impl DriftDetector {
    pub fn detect(
        &self,
        baseline: &FeatureDistribution,
        current: &FeatureDistribution,
        window: DriftWindow,
    ) -> Result<DriftReport, DriftError> {
        let p = baseline.to_probs();
        let q = current.to_probs();
        let psi_val = psi(&p, &q)?;
        let ks = ks_statistic(&p, &q)?;
        let js = js_divergence(&p, &q)?;
        let wd = wasserstein_distance(&p, &q)?;
        let severity = severity_from_psi(psi_val);
        Ok(DriftReport {
            feature_name: baseline.feature_name.clone(),
            window,
            psi: psi_val,
            ks_stat: ks,
            js_div: js,
            wasserstein: wd,
            severity,
            computed_at: Utc::now(),
        })
    }
}
