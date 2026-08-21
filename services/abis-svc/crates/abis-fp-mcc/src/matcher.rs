//! MCC end-to-end matcher.
//!
//! Reference: Cappelli, Ferrara, Maltoni, IEEE TPAMI 2010.

use abis_matcher::modality::fingerprint::FingerprintTemplate;

use crate::build::build_mcc;
use crate::consolidate::global_similarity;
use crate::params::MccParams;

/// MCC matcher: builds cylinders for both templates and runs LSSR consolidation.
pub struct MccMatcher {
    pub params: MccParams,
}

impl Default for MccMatcher {
    fn default() -> Self {
        Self {
            params: MccParams::default(),
        }
    }
}

impl MccMatcher {
    pub fn new(params: MccParams) -> Self {
        Self { params }
    }

    /// Compare two fingerprint templates. Returns a score in [0.0, 1.0].
    pub fn compare(&self, probe: &FingerprintTemplate, gallery: &FingerprintTemplate) -> f32 {
        let probe_cyls = build_mcc(probe, &self.params);
        let gallery_cyls = build_mcc(gallery, &self.params);
        global_similarity(&probe_cyls, &gallery_cyls, self.params.min_matched)
    }
}
