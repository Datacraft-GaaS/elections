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

//! eel-abis-fp-vendor-peak: Cross-vendor 2-of-3 fingerprint consensus during peak window.
//!
//! Activates dual-vendor 2-of-3 voting on fingerprint matches during the
//! election freeze period (T-1..T+7). Wraps VendorRouter with a three-way
//! dispatch: our internal matcher + vendor A + vendor B. Results are fused via
//! MinAgreementPolicy from eel-abis-fp-consensus.
//!
//! Score calibration uses per-vendor Platt scaling so scores from heterogeneous
//! vendors are comparable before voting.
//!
//! References:
//!   Platt, J. "Probabilistic outputs for support vector machines", Advances in
//!   Large Margin Classifiers, MIT Press, 1999.
//!
//!   Jain, A. K., Nandakumar, K., Ross, A. "Score normalization in multimodal
//!   biometric systems," Pattern Recognition, vol. 38, no. 12, pp. 2270-2285, 2005.
#![warn(missing_docs)]

pub mod api;
pub mod calibration;
pub mod degrade;
pub mod metrics;
pub mod peak_policy;
pub mod vendor_router;

pub use calibration::{PlattCalibrator, VendorCalibrators};
pub use degrade::{DegradeOutcome, DegradedCandidate};
pub use peak_policy::PeakPolicy;
pub use vendor_router::{PeakVendorRouter, VendorTriple};

use thiserror::Error;

/// Top-level errors from this crate.
#[derive(Debug, Error)]
pub enum PeakError {
    #[error("consensus rejected: {votes} of {total} vendors agree (minimum 2)")]
    ConsensusFailed { votes: usize, total: usize },
    #[error("not enough healthy vendors: {healthy} available (minimum 2)")]
    InsufficientVendors { healthy: usize },
    #[error("vendor error: {vendor}: {detail}")]
    VendorError { vendor: String, detail: String },
    #[error("calibration error: {0}")]
    Calibration(String),
    #[error("policy error: {0}")]
    Policy(String),
}
