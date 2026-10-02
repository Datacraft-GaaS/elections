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

//! Multi-algorithm consensus fingerprint matcher.
//!
//! Runs up to 5 sub-matchers and fuses results via weighted voting.
//! Sub-matchers: Bozorth3, MCC, FingerCode texture, orientation field, skeleton graph.
//!
//! Reference: A. K. Jain, K. Nandakumar, A. Ross, "Score normalization in multimodal biometric
//! systems," Pattern Recognition, vol. 38, no. 12, pp. 2270-2285, 2005.
#![warn(missing_docs)]

pub mod calibration;
pub mod consensus;
pub mod policy;

pub use calibration::ScoreCalibrator;
pub use consensus::{ConsensusInput, ConsensusMatcher, ConsensusScore};
pub use policy::MinAgreementPolicy;

use thiserror::Error;

/// Errors from this crate.
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("consensus input is empty (no sub-matchers available)")]
    NoMatchers,
    #[error("policy rejected: only {votes} of {total} matchers voted match (minimum {min})")]
    PolicyRejected { votes: usize, total: usize, min: usize },
    #[error("sub-matcher error: {name}: {detail}")]
    SubMatcherError { name: String, detail: String },
}
