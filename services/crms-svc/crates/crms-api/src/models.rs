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

//! Request DTOs for the CRMS API. Responses are the crms-domain aggregates
//! directly (all are `Serialize`), so no response wrappers are needed.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crms_domain::ballot::BallotPosition;
use crms_domain::candidate::{Gender, Position};
use crms_domain::clearance::{ClearanceSource, ClearanceVerdict};
use crms_domain::gazette::GazetteEntry;
use crms_domain::primaries::CandidateTally;

/// Create a candidate aggregate (FR-M12-001).
#[derive(Debug, Deserialize)]
pub struct CreateCandidate {
    /// SHA-256 hash of the candidate's national ID number.
    pub id_number_hash: String,
    /// Candidate's full legal name.
    pub name: String,
    /// Sponsoring party id, or `None` for independent candidates.
    pub party_id: Option<Uuid>,
    /// Seat the candidate is contesting.
    pub position: Position,
    /// Constituency code the candidate is registered in.
    pub constituency_code: String,
    /// Candidate's gender.
    pub gender: Gender,
    /// ISO date of birth.
    pub date_of_birth: String,
    /// Whether the candidate is standing as an independent.
    pub is_independent: bool,
}

/// Open a nomination for an existing candidate.
#[derive(Debug, Deserialize)]
pub struct CreateNomination {
    /// Id of the candidate being nominated.
    pub candidate_id: Uuid,
}

/// Append a statutory document to a nomination (EC-DAT-07).
#[derive(Debug, Deserialize)]
pub struct AddDocument {
    /// Document identifier assigned by the document store.
    pub doc_id: Uuid,
    /// Document type (e.g. degree, ID copy, tax clearance).
    pub doc_type: String,
    /// Storage reference to the document blob.
    pub storage_ref: String,
}

/// Record an RO clearance decision against a nomination.
#[derive(Debug, Deserialize)]
pub struct CreateClearance {
    /// Nomination under review.
    pub nomination_id: Uuid,
    /// Statutory source of the clearance.
    pub source: ClearanceSource,
    /// Clearance verdict.
    pub verdict: ClearanceVerdict,
    /// Optional reference to supporting evidence.
    pub evidence_ref: Option<String>,
    /// Officer recording the decision.
    pub officer_id: String,
    /// Reason when the verdict overrides the source's finding.
    pub override_reason: Option<String>,
}

/// One cleared candidate row fed to the quota engine.
#[derive(Debug, Deserialize)]
pub struct QuotaCandidate {
    /// Candidate's gender.
    pub gender: Gender,
    /// Whether the candidate is a person with disability.
    pub is_pwd: bool,
    /// ISO date of birth used for age-based quota.
    pub date_of_birth: String,
}

/// Evaluate the two-thirds gender rule for a roster.
#[derive(Debug, Deserialize)]
pub struct EvaluateQuota {
    /// Quota scope (national, county, or constituency).
    pub scope: String,
    /// Seat under evaluation.
    pub position: Position,
    /// ISO date nominations close.
    pub nomination_date: String,
    /// Cleared candidates to evaluate.
    pub candidates: Vec<QuotaCandidate>,
}

/// Advisory clearance from a statutory source (KRA / EACC).
#[derive(Debug, Deserialize)]
pub struct AdvisoryClearance {
    /// Statutory source issuing the advisory.
    pub source: ClearanceSource,
    /// Hash of the national ID under advisory review.
    pub id_number_hash: String,
}

/// Build a gazette notice.
#[derive(Debug, Deserialize)]
pub struct BuildGazette {
    /// Election the notice belongs to.
    pub election_id: String,
    /// Position code the notice covers.
    pub position_code: String,
    /// Entries to publish in the notice.
    pub entries: Vec<GazetteEntry>,
    /// First gazetting officer.
    pub officer_a_id: String,
    /// Second gazetting officer.
    pub officer_b_id: String,
}

/// Build a ballot proof for a position in a constituency.
#[derive(Debug, Deserialize)]
pub struct BuildBallot {
    /// Election the ballot belongs to.
    pub election_id: String,
    /// Position code.
    pub position_code: String,
    /// Constituency code.
    pub constituency_code: String,
    /// Ordered list of candidates on the ballot.
    pub positions: Vec<BallotPosition>,
}

/// Approval carries the printer's signature for a ballot proof.
#[derive(Debug, Deserialize)]
pub struct ApproveBallot {
    /// Printer's digital signature over the proof.
    pub printer_sig: String,
}

/// Register a political party (M45).
#[derive(Debug, Deserialize)]
pub struct CreateParty {
    /// English party name.
    pub name_en: String,
    /// Swahili party name.
    pub name_sw: String,
    /// Party abbreviation.
    pub abbreviation: String,
    /// Party headquarters address.
    pub hq_address: String,
}

/// Registrar approval of a party with its registration number.
#[derive(Debug, Deserialize)]
pub struct ApproveParty {
    /// Registrar-assigned registration number.
    pub registration_number: String,
    /// URL of the published gazette notice.
    pub gazette_url: String,
}

/// Register a coalition of parties.
#[derive(Debug, Deserialize)]
pub struct CreateCoalition {
    /// Coalition name.
    pub name: String,
    /// Member party ids.
    pub member_party_ids: Vec<Uuid>,
}

/// Confirm a member party's membership of a coalition.
#[derive(Debug, Deserialize)]
pub struct ConfirmMember {
    /// Party confirming membership.
    pub party_id: Uuid,
}

/// Submit a party primary result (M46).
#[derive(Debug, Deserialize)]
pub struct SubmitPrimaryResult {
    /// Party running the primary.
    pub party_id: Uuid,
    /// Constituency the primary was held in.
    pub constituency_code: String,
    /// Seat being contested.
    pub position: String,
    /// Officer submitting the result.
    pub submitted_by: String,
    /// Per-candidate tallies.
    pub candidate_tallies: Vec<CandidateTally>,
    /// Spoilt ballot count.
    pub spoilt_ballots: u32,
    /// Total turnout.
    pub total_turnout: u32,
}

/// File a dispute against a primary result.
#[derive(Debug, Deserialize)]
pub struct CreatePrimaryDispute {
    /// Result being disputed.
    pub result_id: Uuid,
    /// Complainant's name.
    pub complainant_name: String,
    /// Complainant's national ID.
    pub complainant_id: String,
    /// Grounds for the dispute.
    pub grounds: String,
    /// URLs to supporting evidence.
    pub evidence_urls: Vec<String>,
}

/// Declare a nomination fee schedule for a position.
#[derive(Debug, Deserialize)]
pub struct CreateFeeSchedule {
    /// Election cycle the schedule applies to.
    pub election_cycle: String,
    /// Seat the fee covers.
    pub position: Position,
    /// Fee amount in cents.
    pub amount_cents: u64,
    /// Deadline for payment.
    pub due_at: DateTime<Utc>,
}

/// Record a nomination fee payment.
#[derive(Debug, Deserialize)]
pub struct RecordPayment {
    /// Fee schedule being paid against.
    pub schedule_id: Uuid,
    /// Candidate making the payment.
    pub candidate_id: Uuid,
    /// Amount paid in cents.
    pub amount_cents: u64,
}

/// Confirm a pending payment via M-Pesa reference.
#[derive(Debug, Deserialize)]
pub struct ConfirmPayment {
    /// M-Pesa transaction reference.
    pub mpesa_ref: String,
}

/// Register a party symbol and colour (M67).
#[derive(Debug, Deserialize)]
pub struct RegisterSymbol {
    /// Party registering the symbol.
    pub party_id: Uuid,
    /// Symbol name.
    pub symbol_name: String,
    /// URL of the symbol image.
    pub symbol_url: String,
    /// Symbol colour as a hex value.
    pub colour_hex: String,
}
