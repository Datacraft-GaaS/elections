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

//! Detection-per-request (DPR) gating.
//!
//! Reference: NIST SP 500-290 MINEX III protocol, Section 5.2 -- multi-finger gate.
//!
//! Reject a transaction when too few fingers produce a usable match score relative to the
//! number of available fingers.  Prevents a single-finger impersonation attack on a 10-print
//! gallery where the probe is deliberately degraded.

/// Configuration for the DPR policy.
#[derive(Debug, Clone)]
pub struct DprPolicy {
    /// Minimum number of fingers that must individually exceed per-finger threshold.
    pub min_matched: usize,
    /// Minimum number of available (acquired) fingers required to trigger the gate.
    pub min_available: usize,
}

impl Default for DprPolicy {
    fn default() -> Self {
        Self {
            min_matched: 2,
            min_available: 6,
        }
    }
}

/// Outcome of the DPR check.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DprResult {
    pub passed: bool,
    pub matched: usize,
    pub available: usize,
    pub reason: Option<String>,
}

impl DprPolicy {
    /// Evaluate DPR.  Returns passed=true when the transaction is acceptable.
    pub fn evaluate(&self, matched: usize, available: usize) -> DprResult {
        if available >= self.min_available && matched < self.min_matched {
            DprResult {
                passed: false,
                matched,
                available,
                reason: Some(format!(
                    "DPR gate: only {matched} of {available} fingers matched (minimum {min})",
                    min = self.min_matched
                )),
            }
        } else {
            DprResult {
                passed: true,
                matched,
                available,
                reason: None,
            }
        }
    }
}
