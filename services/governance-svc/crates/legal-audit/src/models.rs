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

use serde::{Deserialize, Serialize};

/// One transmitted record and its declared legal basis.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransmissionRecord {
    pub record_id: String,
    pub event_type: String,
    pub sender: String,
    pub recipient: String,
    pub transmitted_at: String,
    pub payload_hash: String,
    pub legal_basis: String,
}

/// One §39 legal-basis rule.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegalBasisRule {
    pub rule_id: String,
    pub section_ref: String,
    pub description: String,
    pub applies_to: Vec<String>,
}

/// The outcome of evaluating one record against one rule.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceVerdict {
    pub verdict_id: String,
    pub record_id: String,
    pub rule_id: String,
    /// The section of the instrument `rule_id` names.
    pub section_ref: String,
    pub passed: bool,
    /// Hash chain over prior verdicts plus this verdict's identifying fields.
    ///
    /// Commits each verdict to everything recorded before it, so removing or
    /// reordering one changes every subsequent hash. This is a local chain over
    /// this process's store; it is not a transparency-log inclusion proof, which
    /// is why `tlog_inclusion_proven` is false and no claim is made.
    pub verdict_hash: String,
    /// Always false here: there is no append-only log in this workspace to
    /// submit a leaf to. Carried explicitly so no consumer can mistake this hash
    /// for an inclusion proof.
    pub tlog_inclusion_proven: bool,
    pub evaluated_at: String,
}

/// Per-party compliance rollup.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartySummary {
    pub party_id: String,
    pub party_name: String,
    pub total_transmissions: u32,
    pub compliant: u32,
    pub non_compliant: u32,
}

/// A request to record a transmission.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecordTransmissionRequest {
    pub event_type: String,
    pub sender: String,
    pub recipient: String,
    /// The legal instrument relied on. An empty value is non-compliant, which
    /// `rules::evaluate_record` decides rather than this type.
    #[serde(default)]
    pub legal_basis: String,
    /// An optional caller-supplied payload digest. When absent the handler
    /// hashes the payload it actually received, so the stored record's digest is
    /// self-consistent.
    #[serde(default)]
    pub payload_hash: Option<String>,
    /// The transmitted body. Retained so the digest above covers real content.
    #[serde(default)]
    pub payload: serde_json::Value,
}
