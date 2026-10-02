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

//! Public contract models (governance-recall/recall.yaml).
//!
//! These are the spec-facing DTOs (hash-based, PII-safe) — distinct from the
//! internal aggregate in `crate::models`.

use serde::{Deserialize, Serialize};

/// Request body for initiating a recall petition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetitionRequest {
    pub constituency_code: String,
    pub member_id_hash: String,
    pub grounds: String,
    pub initiator_id_hash: String,
    pub petition_date: String,
}

/// Public recall petition view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Petition {
    pub petition_id: String,
    pub constituency_code: String,
    pub member_id_hash: String,
    pub grounds: String,
    pub initiator_id_hash: String,
    pub petition_date: String,
    pub status: String,
    pub threshold_required: u64,
    pub signatures_collected: u64,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for recording a signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRequest {
    pub petitioner_id_hash: String,
    pub ward_code: String,
    pub signed_at: String,
}

/// Response body for recording a signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureResponse {
    pub accepted: bool,
    pub count_so_far: u64,
    pub rejection_reason: Option<String>,
}

/// Request/response body for signature-count verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerification {
    pub verified_count: u64,
    pub verified_by_hash: String,
    pub method: String,
    pub verified_at: Option<String>,
}
