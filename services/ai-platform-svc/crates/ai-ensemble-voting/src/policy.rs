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

//! EnsemblePolicy: K-of-N quorum and disagreement handling.

use serde::{Deserialize, Serialize};

use crate::error::EnsembleError;

/// What to do when fewer than K models agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisagreementAction {
    /// Reject the request with an error.
    Reject,
    /// Escalate to human review and return an escalation receipt.
    Escalate,
    /// Accept the plurality label but emit a warning log.
    LogOnly,
}

/// Policy governing one decision class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsemblePolicy {
    /// Minimum number of models that must agree on the winning label.
    pub k: usize,
    /// Total number of models in the ensemble.
    pub n: usize,
    /// Action taken when fewer than k models agree.
    pub disagreement_action: DisagreementAction,
}

impl EnsemblePolicy {
    /// Create a new policy, validating that k <= n.
    pub fn new(
        k: usize,
        n: usize,
        disagreement_action: DisagreementAction,
    ) -> Result<Self, EnsembleError> {
        if k > n {
            return Err(EnsembleError::PolicyInvalid { k, n });
        }
        Ok(Self {
            k,
            n,
            disagreement_action,
        })
    }
}
