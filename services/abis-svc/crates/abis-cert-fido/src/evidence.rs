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

//! FIDO Biometric Component Certification evidence pack.

use crate::matrix::PaiMatrix;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvidenceError {
    #[error("PAI matrix must have at least one entry")]
    EmptyMatrix,
    #[error("FMR {fmr:.6} exceeds target {target:.6}")]
    FmrExceedsTarget { fmr: f64, target: f64 },
    #[error("FNMR {fnmr:.6} exceeds target {target:.6}")]
    FnmrExceedsTarget { fnmr: f64, target: f64 },
}

/// FMR/FNMR measurement at a given decision threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmrFnmrReport {
    pub threshold: f64,
    pub fmr: f64,
    pub fnmr: f64,
    pub genuine_attempts: u64,
    pub impostor_attempts: u64,
}

/// Complete certification evidence pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePack {
    pub component_id: String,
    pub pad_report: PadReport,
    pub fmr_fnmr_report: FmrFnmrReport,
    pub coverage_matrix: PaiMatrix,
    pub tester_signature: String,
    pub generated_at: DateTime<Utc>,
}

/// PAD performance summary for the evidence pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadReport {
    pub species_tested: usize,
    pub species_passed: usize,
    pub max_apcer: f64,
    pub max_bpcer: f64,
}

impl EvidencePack {
    /// Assembles a certification evidence pack.
    ///
    /// # Errors
    /// Returns an error if the matrix is empty or error rates exceed targets.
    pub fn assemble(
        component_id: &str,
        pad_report: PadReport,
        fmr_fnmr: FmrFnmrReport,
        matrix: PaiMatrix,
        target_fmr: f64,
        target_fnmr: f64,
    ) -> Result<Self, EvidenceError> {
        if matrix.entries.is_empty() {
            return Err(EvidenceError::EmptyMatrix);
        }
        if fmr_fnmr.fmr > target_fmr {
            return Err(EvidenceError::FmrExceedsTarget {
                fmr: fmr_fnmr.fmr,
                target: target_fmr,
            });
        }
        if fmr_fnmr.fnmr > target_fnmr {
            return Err(EvidenceError::FnmrExceedsTarget {
                fnmr: fmr_fnmr.fnmr,
                target: target_fnmr,
            });
        }

        let signature = sign_stub(component_id, &fmr_fnmr);

        Ok(Self {
            component_id: component_id.to_owned(),
            pad_report,
            fmr_fnmr_report: fmr_fnmr,
            coverage_matrix: matrix,
            tester_signature: signature,
            generated_at: Utc::now(),
        })
    }
}

/// Stub tester signature: SHA-256(component_id || fmr || fnmr).
fn sign_stub(component_id: &str, report: &FmrFnmrReport) -> String {
    let mut h = Sha256::new();
    h.update(component_id.as_bytes());
    h.update(report.fmr.to_le_bytes());
    h.update(report.fnmr.to_le_bytes());
    hex::encode(h.finalize())
}
