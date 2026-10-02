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

//! PeakVendorRouter: three-way vendor dispatch with 2-of-3 consensus.
//!
//! During an active PeakPolicy window, every fingerprint comparison is sent to
//! three sources: the internal eel matcher, vendor A, and vendor B. Scores are
//! Platt-calibrated and fed to MinAgreementPolicy (min_votes=2).
//!
//! Outside the window the internal matcher runs alone.
//!
//! References:
//!   Jain, A. K., Nandakumar, K., Ross, A. "Score normalization in multimodal
//!   biometric systems," Pattern Recognition, vol. 38, no. 12, pp. 2270-2285, 2005.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use eel_abis_fp_consensus::MinAgreementPolicy;

use crate::calibration::VendorCalibrators;
use crate::degrade::DegradedCandidate;
use crate::metrics::{inc_consensus_agreements, inc_consensus_disagreements, inc_vendor_degraded};
use crate::peak_policy::PeakPolicy;
use crate::PeakError;

/// Mock-able vendor health and scoring interface.
pub trait VendorMatcher: Send + Sync {
    fn name(&self) -> &str;
    fn is_healthy(&self) -> bool;
    /// Score two fingerprint templates represented as byte slices. Returns [0, 1].
    fn score(&self, probe: &[u8], gallery: &[u8]) -> Result<f64, PeakError>;
}

/// The three matchers used during peak window.
pub struct VendorTriple {
    pub internal: Box<dyn VendorMatcher>,
    pub vendor_a: Box<dyn VendorMatcher>,
    pub vendor_b: Box<dyn VendorMatcher>,
}

/// Result of a peak-mode match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakMatchResult {
    /// Calibrated majority score (mean of agreeing vendors).
    pub score: f64,
    /// How many vendors voted agree.
    pub agree_votes: usize,
    /// Total healthy vendors used.
    pub total_vendors: usize,
    /// Whether the 2-of-3 policy accepted this pair.
    pub accepted: bool,
}

/// Wraps the three-way dispatch with policy gating.
pub struct PeakVendorRouter {
    pub policy: PeakPolicy,
    pub calibrators: VendorCalibrators,
    pub agreement: MinAgreementPolicy,
    pub triple: VendorTriple,
}

impl PeakVendorRouter {
    pub fn new(policy: PeakPolicy, calibrators: VendorCalibrators, triple: VendorTriple) -> Self {
        Self {
            policy,
            calibrators,
            // 2-of-3 minimum agreement.
            agreement: MinAgreementPolicy::new(2),
            triple,
        }
    }

    /// Run the match. Falls back to single-vendor when policy is inactive.
    pub fn match_fingerprints(
        &self,
        probe: &[u8],
        gallery: &[u8],
    ) -> Result<PeakMatchResult, PeakError> {
        if !self.policy.is_active(Utc::now()) {
            debug!("peak policy inactive; single-vendor path");
            return self.single_vendor_match(probe, gallery);
        }
        self.three_way_match(probe, gallery)
    }

    /// Three-way dispatch with degrade fallback.
    pub fn three_way_match(
        &self,
        probe: &[u8],
        gallery: &[u8],
    ) -> Result<PeakMatchResult, PeakError> {
        let healthy: Vec<&dyn VendorMatcher> = [
            self.triple.internal.as_ref(),
            self.triple.vendor_a.as_ref(),
            self.triple.vendor_b.as_ref(),
        ]
        .iter()
        .copied()
        .filter(|m| m.is_healthy())
        .collect();

        if healthy.len() < 2 {
            let n = healthy.len();
            warn!(
                healthy = n,
                "insufficient healthy vendors for peak consensus"
            );
            return Err(PeakError::InsufficientVendors { healthy: n });
        }

        // Collect raw scores from healthy vendors.
        let mut raw_scores: Vec<(String, f64)> = Vec::new();
        for m in &healthy {
            match m.score(probe, gallery) {
                Ok(s) => raw_scores.push((m.name().to_string(), s)),
                Err(e) => {
                    warn!(vendor = m.name(), error = %e, "vendor score failed; treating as unhealthy");
                    inc_vendor_degraded(m.name());
                }
            }
        }

        if raw_scores.len() < 2 {
            return Err(PeakError::InsufficientVendors {
                healthy: raw_scores.len(),
            });
        }

        // Calibrate: map vendor names to calibrated scores.
        let calibrated: Vec<f64> = raw_scores
            .iter()
            .enumerate()
            .map(|(i, (_, s))| {
                // Map by index position in the healthy list: 0=internal, 1=vendor_a, 2=vendor_b.
                match i {
                    0 => self.calibrators.internal.calibrate(*s),
                    1 => self.calibrators.vendor_a.calibrate(*s),
                    _ => self.calibrators.vendor_b.calibrate(*s),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Decision threshold: 0.5 on calibrated posterior.
        let threshold = 0.5_f64;
        let agree_scores: Vec<f64> = calibrated
            .iter()
            .copied()
            .filter(|&c| c >= threshold)
            .collect();
        let agree_votes = agree_scores.len();
        let total = calibrated.len();

        if self.agreement.check(agree_votes, total) {
            inc_consensus_agreements();
            let score = agree_scores.iter().sum::<f64>() / agree_scores.len() as f64;
            Ok(PeakMatchResult {
                score,
                agree_votes,
                total_vendors: total,
                accepted: true,
            })
        } else {
            inc_consensus_disagreements();
            let _mean_score = calibrated.iter().sum::<f64>() / calibrated.len() as f64;
            Err(PeakError::ConsensusFailed {
                votes: agree_votes,
                total,
            })
        }
    }

    /// Single-vendor path when policy is inactive.
    fn single_vendor_match(
        &self,
        probe: &[u8],
        gallery: &[u8],
    ) -> Result<PeakMatchResult, PeakError> {
        let s = self.triple.internal.score(probe, gallery)?;
        let cal = self.calibrators.internal.calibrate(s)?;
        Ok(PeakMatchResult {
            score: cal,
            agree_votes: 1,
            total_vendors: 1,
            accepted: cal >= 0.5,
        })
    }

    /// Attempt consensus with a known-unhealthy vendor bypassed, returning a
    /// DegradedCandidate to signal reduced confidence.
    pub fn degraded_match(
        &self,
        probe: &[u8],
        gallery: &[u8],
        unavailable_vendor: &str,
    ) -> Result<DegradedCandidate, PeakError> {
        inc_vendor_degraded(unavailable_vendor);
        let matchers: Vec<&dyn VendorMatcher> = [
            self.triple.internal.as_ref(),
            self.triple.vendor_a.as_ref(),
            self.triple.vendor_b.as_ref(),
        ]
        .iter()
        .copied()
        .filter(|m| m.name() != unavailable_vendor && m.is_healthy())
        .collect();

        if matchers.len() < 2 {
            return Ok(DegradedCandidate::single_vendor(
                0.0,
                vec![unavailable_vendor.to_string()],
            ));
        }

        let scores: Vec<f64> = matchers
            .iter()
            .filter_map(|m| m.score(probe, gallery).ok())
            .collect();

        if scores.len() < 2 {
            return Ok(DegradedCandidate::single_vendor(
                0.0,
                vec![unavailable_vendor.to_string()],
            ));
        }

        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let accepted = scores.iter().all(|&s| s >= 0.5);
        Ok(DegradedCandidate::two_of_two(
            mean,
            accepted,
            unavailable_vendor.to_string(),
        ))
    }
}
