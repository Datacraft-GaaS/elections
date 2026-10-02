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

// ---------------------------------------------------------------------------
// Recall petition status FSM
// Valid transitions: Draft -> Active -> {Closed, LegalChallengeHold}
//   Closed -> {Upheld, Dismissed}
//   LegalChallengeHold -> {ElectionDeclared, PetitionDismissed}
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PetitionStatus {
    Draft,
    Active,
    LegalChallengeHold,
    Closed,
    Upheld,
    Dismissed,
}

impl Default for PetitionStatus {
    fn default() -> Self {
        Self::Draft
    }
}

// Recall grounds are a closed list per Art. 104(1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallGround {
    ConvictionOrSentence,
    ChapterSixViolation,
    AbuseOfOffice,
}

/// M48 aggregate: RecallPetition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecallPetition {
    pub id: String,
    pub constituency_id: String,
    pub mp_id: String,
    pub election_cycle_id: String,
    pub grounds: Vec<RecallGround>,
    pub status: PetitionStatus,
    /// Cached registered voter count for threshold computation (integer, no FP)
    pub registered_voters: u64,
    /// Derived from registered_voters * 30 (numerator for 30% test)
    pub signatory_target: u64,
    pub filed_at: String,
    pub window_closes_at: String,
    pub court_reference: Option<String>,
}

/// M48 aggregate: SignatoryRecord -- one row per verified signatory import
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignatoryRecord {
    pub id: String,
    pub petition_id: String,
    pub voter_id: String,
    pub ward: String,
    /// Confidence score from OCR engine (0.0 - 1.0); rows < 0.80 flagged for manual re-entry
    pub ocr_confidence: Option<f64>,
    /// VALID | NOT_ON_ROLL | DUPLICATE | FLAGGED
    pub verification_status: String,
    pub verified_at: Option<String>,
    pub verification_method: String,
}

/// M48 aggregate: ThresholdReport
///
/// All arithmetic is integer-only per FR-M48-008.
/// threshold_pct_numerator / threshold_pct_denominator represents the fraction;
/// meets_threshold is true iff verified_signatories * 100 >= registered_voters * 30.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThresholdReport {
    pub id: String,
    pub petition_id: String,
    pub total_signatories: u64,
    pub verified_signatories: u64,
    pub registered_voters: u64,
    /// verified_signatories * 100 (numerator; compare >= registered_voters * 30)
    pub numerator_x100: u64,
    /// registered_voters * 30 (denominator target)
    pub denominator_30pct: u64,
    pub meets_threshold: bool,
    /// Ward-level breakdown as JSON blob
    pub ward_breakdown: serde_json::Value,
    pub generated_at: String,
}

/// M48 aggregate: RecallElection -- created when petition is upheld, links to M50 ByElection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecallElection {
    pub id: String,
    pub petition_id: String,
    /// FK to M50 ByElection; set once M50 acknowledges creation
    pub by_election_id: Option<String>,
    /// PENDING | TRIGGERED | FAILED (retry up to 3x)
    pub trigger_status: String,
    pub trigger_attempts: u8,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// OCR import batch types
// ---------------------------------------------------------------------------

/// Single row from a signatory CSV/OCR batch
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignatoryImportRow {
    pub voter_id: String,
    pub name: String,
    pub ward: String,
    pub signature_date: String,
    pub ocr_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatoryImportResult {
    pub batch_id: String,
    pub total_rows: usize,
    pub accepted: usize,
    pub flagged_for_manual: usize,
    pub duplicates: usize,
    pub not_on_roll: usize,
    pub errors: Vec<RowError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowError {
    pub row_index: usize,
    pub voter_id: String,
    pub reason: String,
}
