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

//! Public contract models (governance-forensics/forensics.yaml).

use serde::{Deserialize, Serialize};

/// Request body for initiating a forensic investigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationRequest {
    pub investigation_type: String,
    pub subject_ref: String,
    pub trigger_event_id: String,
    pub initiated_by_hash: String,
}

/// Public forensic investigation view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investigation {
    pub investigation_id: String,
    pub investigation_type: String,
    pub subject_ref: String,
    pub trigger_event_id: String,
    pub initiated_by_hash: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for attaching evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceAttachmentRequest {
    pub evidence_type: String,
    pub source: String,
    pub hash: String,
    pub storage_ref: String,
}

/// Public evidence attachment view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceAttachment {
    pub attachment_id: String,
    pub investigation_id: String,
    pub evidence_type: String,
    pub source: String,
    pub hash: String,
    pub storage_ref: String,
    pub attached_at: String,
}

/// Request body for adding a forensic finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingRequest {
    pub finding_type: String,
    pub amount_kes: Option<u64>,
    pub description: String,
    pub referral: String,
}

/// Public forensic finding view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub finding_id: String,
    pub investigation_id: String,
    pub finding_type: String,
    pub amount_kes: Option<u64>,
    pub description: String,
    pub referral: String,
    pub created_at: String,
}
