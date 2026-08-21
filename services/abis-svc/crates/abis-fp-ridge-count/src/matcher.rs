//! Ridge-count matcher and K-plet matcher.
//!
//! Reference: Watson et al., NISTIR 7392, 2007; Ratha et al., IEEE AVSPA 2000.

use abis_matcher::modality::fingerprint::FingerprintTemplate;

use crate::count::OrientationField;
use crate::kplet::{build_kplets, KPlet, KPletDescriptor};

/// Positional tolerance for rho matching (pixels).
const RHO_TOL: f32 = 15.0;
/// Angular tolerance for phi matching (degrees).
const PHI_TOL: f32 = 20.0;
/// Angular tolerance for theta_diff matching (degrees).
const THETA_TOL: f32 = 15.0;
/// Ridge count tolerance.
const RIDGE_COUNT_TOL: u8 = 2;

/// Compare two K-plet tuples and return a similarity in [0, 1] if compatible.
fn tuple_similarity(a: &KPlet, b: &KPlet) -> Option<f32> {
    let rho_diff = (a.rho - b.rho).abs();
    let phi_diff = (a.phi - b.phi).abs().min(360.0 - (a.phi - b.phi).abs());
    let theta_diff = (a.theta_diff - b.theta_diff).abs();
    let rc_diff = a.ridge_count.abs_diff(b.ridge_count);

    if rho_diff > RHO_TOL
        || phi_diff > PHI_TOL
        || theta_diff > THETA_TOL
        || rc_diff > RIDGE_COUNT_TOL
    {
        return None;
    }

    // Weighted similarity.
    let rho_sim = 1.0 - rho_diff / RHO_TOL;
    let phi_sim = 1.0 - phi_diff / PHI_TOL;
    let theta_sim = 1.0 - theta_diff / THETA_TOL;
    let rc_sim = 1.0 - rc_diff as f32 / (RIDGE_COUNT_TOL as f32 + 1.0);

    Some(rho_sim * 0.3 + phi_sim * 0.3 + theta_sim * 0.2 + rc_sim * 0.2)
}

/// Descriptor-level similarity: fraction of probe tuples with a compatible gallery tuple.
fn descriptor_similarity(probe: &KPletDescriptor, gallery: &KPletDescriptor) -> f32 {
    if probe.tuples.is_empty() {
        return 0.0;
    }
    let mut matched = 0.0f32;
    for pt in &probe.tuples {
        let best = gallery
            .tuples
            .iter()
            .filter_map(|gt| tuple_similarity(pt, gt))
            .fold(0.0f32, f32::max);
        matched += best;
    }
    (matched / probe.tuples.len() as f32).clamp(0.0, 1.0)
}

/// K-plet matcher: compare two templates using their K-plet descriptor sets.
pub struct KPletMatcher {
    pub orientation_field: OrientationField,
}

impl KPletMatcher {
    pub fn new(orientation_field: OrientationField) -> Self {
        Self { orientation_field }
    }

    /// Build K-plet descriptors and compare via set intersection similarity.
    pub fn compare(&self, probe: &FingerprintTemplate, gallery: &FingerprintTemplate) -> f32 {
        let probe_kplets = build_kplets(probe, &self.orientation_field);
        let gallery_kplets = build_kplets(gallery, &self.orientation_field);

        if probe_kplets.is_empty() || gallery_kplets.is_empty() {
            return 0.0;
        }

        // For each probe descriptor, find the best matching gallery descriptor.
        let total: f32 = probe_kplets
            .iter()
            .map(|pd| {
                gallery_kplets
                    .iter()
                    .map(|gd| descriptor_similarity(pd, gd))
                    .fold(0.0f32, f32::max)
            })
            .sum();

        (total / probe_kplets.len() as f32).clamp(0.0, 1.0)
    }
}

/// Ridge-count matcher: uses the stored ridge-count matrix in the templates.
///
/// The matrix `template.ridge_counts` stores the lower-triangle of pairwise ridge
/// counts. This matcher computes the fraction of ridge-count pairs that agree within
/// tolerance between probe and gallery (after sorting by distance).
pub struct RidgeCountMatcher {
    pub tolerance: u8,
}

impl Default for RidgeCountMatcher {
    fn default() -> Self {
        Self { tolerance: 2 }
    }
}

impl RidgeCountMatcher {
    pub fn new(tolerance: u8) -> Self {
        Self { tolerance }
    }

    /// Compare two templates by their ridge-count matrices.
    ///
    /// Returns a score in [0.0, 1.0] representing the fraction of matching
    /// ridge-count pairs (within `self.tolerance`).
    pub fn compare(&self, probe: &FingerprintTemplate, gallery: &FingerprintTemplate) -> f32 {
        let pn = probe.minutiae.len();
        let gn = gallery.minutiae.len();

        if pn < 2 || gn < 2 {
            return 0.0;
        }

        let min_n = pn.min(gn);
        let pairs = min_n * (min_n.saturating_sub(1)) / 2;
        if pairs == 0 {
            return 0.0;
        }

        let mut matched = 0u32;
        for idx in 0..pairs
            .min(probe.ridge_counts.len())
            .min(gallery.ridge_counts.len())
        {
            let pr = probe.ridge_counts[idx];
            let gr = gallery.ridge_counts[idx];
            if pr.abs_diff(gr) <= self.tolerance {
                matched += 1;
            }
        }

        (matched as f32 / pairs as f32).clamp(0.0, 1.0)
    }
}
