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

//! NIST FpVTE + PFT-III validation harness.
//!
//! Provides evaluation infrastructure compatible with the NIST Fingerprint
//! Vendor Technology Evaluation (FpVTE) and Proprietary Fingerprint Template
//! Evaluation III (PFT-III) methodologies.
//!
//! References:
//! - P. Grother et al., "NIST Fingerprint Vendor Technology Evaluation 2003,"
//!   NISTIR 7123, 2004.
//! - P. Grother, M. Indovina, "Proprietary Fingerprint Template Evaluation III,"
//!   NIST IR 8307, 2020.
#![warn(missing_docs)]

pub mod corpus;
pub mod harness;
pub mod matcher_trait;
pub mod report;

pub use corpus::{Corpus, CorpusEntry, DemographicLabel};
pub use harness::{FpVteHarness, HarnessConfig};
pub use matcher_trait::FpMatcher;
pub use report::{Report, RocPoint};

/// Error type for the FpVTE harness.
#[derive(Debug, thiserror::Error)]
pub enum FpVteError {
	#[error("corpus load error: {0}")]
	CorpusLoad(String),
	#[error("report write error: {0}")]
	ReportWrite(String),
	#[error("matcher error: {0}")]
	Matcher(String),
	#[error("insufficient corpus: need >= {needed} entries, got {got}")]
	InsufficientCorpus { needed: usize, got: usize },
}
