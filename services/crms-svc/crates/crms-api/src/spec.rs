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

//! Public contract models (crms/nomination.yaml quota + ballot-proof surface).

use serde::{Deserialize, Serialize};

/// Public quota compliance status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaStatus {
    /// Quota scope (constituency, county, or national).
    pub scope: String,
    /// Scope code.
    pub code: String,
    /// Whether the two-thirds gender rule is satisfied.
    pub gender_compliance: bool,
    /// Whether the youth threshold is satisfied.
    pub youth_compliance: bool,
    /// Whether PWD representation is satisfied.
    pub pwd_compliance: bool,
}

/// Request body for publishing a ballot proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotProofRequest {
    /// Election cycle identifier.
    pub election_id: String,
    /// Electoral position code.
    pub position_code: String,
    /// SHA-256 hash of the ballot design.
    pub ballot_hash: String,
    /// Printer's digital signature.
    pub printer_sig: String,
}

/// Public ballot proof view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotProof {
    /// UUIDv7 proof identifier.
    pub proof_id: String,
    /// Election cycle identifier.
    pub election_id: String,
    /// Electoral position code.
    pub position_code: String,
    /// SHA-256 hash of the ballot design.
    pub ballot_hash: String,
    /// Printer's digital signature.
    pub printer_sig: String,
    /// Timestamp the proof was published.
    pub published_at: String,
}
