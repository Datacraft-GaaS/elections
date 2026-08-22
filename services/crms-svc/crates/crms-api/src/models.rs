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
    pub id_number_hash: String,
    pub name: String,
    pub party_id: Option<Uuid>,
    pub position: Position,
    pub constituency_code: String,
    pub gender: Gender,
    pub date_of_birth: String,
    pub is_independent: bool,
}

/// Open a nomination for an existing candidate.
#[derive(Debug, Deserialize)]
pub struct CreateNomination {
    pub candidate_id: Uuid,
}

/// Append a statutory document to a nomination (EC-DAT-07).
#[derive(Debug, Deserialize)]
pub struct AddDocument {
    pub doc_id: Uuid,
    pub doc_type: String,
    pub storage_ref: String,
}

/// Record an RO clearance decision against a nomination.
#[derive(Debug, Deserialize)]
pub struct CreateClearance {
    pub nomination_id: Uuid,
    pub source: ClearanceSource,
    pub verdict: ClearanceVerdict,
    pub evidence_ref: Option<String>,
    pub officer_id: String,
    pub override_reason: Option<String>,
}

/// One cleared candidate row fed to the quota engine.
#[derive(Debug, Deserialize)]
pub struct QuotaCandidate {
    pub gender: Gender,
    pub is_pwd: bool,
    pub date_of_birth: String,
}

/// Evaluate the two-thirds gender rule for a roster.
#[derive(Debug, Deserialize)]
pub struct EvaluateQuota {
    pub scope: String,
    pub position: Position,
    pub nomination_date: String,
    pub candidates: Vec<QuotaCandidate>,
}

/// Advisory clearance from a statutory source (KRA / EACC).
#[derive(Debug, Deserialize)]
pub struct AdvisoryClearance {
    pub source: ClearanceSource,
    pub id_number_hash: String,
}

/// Build a gazette notice.
#[derive(Debug, Deserialize)]
pub struct BuildGazette {
    pub election_id: String,
    pub position_code: String,
    pub entries: Vec<GazetteEntry>,
    pub officer_a_id: String,
    pub officer_b_id: String,
}

/// Build a ballot proof for a position in a constituency.
#[derive(Debug, Deserialize)]
pub struct BuildBallot {
    pub election_id: String,
    pub position_code: String,
    pub constituency_code: String,
    pub positions: Vec<BallotPosition>,
}

/// Approval carries the printer's signature for a ballot proof.
#[derive(Debug, Deserialize)]
pub struct ApproveBallot {
    pub printer_sig: String,
}

/// Register a political party (M45).
#[derive(Debug, Deserialize)]
pub struct CreateParty {
    pub name_en: String,
    pub name_sw: String,
    pub abbreviation: String,
    pub hq_address: String,
}

/// Registrar approval of a party with its registration number.
#[derive(Debug, Deserialize)]
pub struct ApproveParty {
    pub registration_number: String,
    pub gazette_url: String,
}

/// Register a coalition of parties.
#[derive(Debug, Deserialize)]
pub struct CreateCoalition {
    pub name: String,
    pub member_party_ids: Vec<Uuid>,
}

/// Confirm a member party's membership of a coalition.
#[derive(Debug, Deserialize)]
pub struct ConfirmMember {
    pub party_id: Uuid,
}

/// Submit a party primary result (M46).
#[derive(Debug, Deserialize)]
pub struct SubmitPrimaryResult {
    pub party_id: Uuid,
    pub constituency_code: String,
    pub position: String,
    pub submitted_by: String,
    pub candidate_tallies: Vec<CandidateTally>,
    pub spoilt_ballots: u32,
    pub total_turnout: u32,
}

/// File a dispute against a primary result.
#[derive(Debug, Deserialize)]
pub struct CreatePrimaryDispute {
    pub result_id: Uuid,
    pub complainant_name: String,
    pub complainant_id: String,
    pub grounds: String,
    pub evidence_urls: Vec<String>,
}

/// Declare a nomination fee schedule for a position.
#[derive(Debug, Deserialize)]
pub struct CreateFeeSchedule {
    pub election_cycle: String,
    pub position: Position,
    pub amount_cents: u64,
    pub due_at: DateTime<Utc>,
}

/// Record a nomination fee payment.
#[derive(Debug, Deserialize)]
pub struct RecordPayment {
    pub schedule_id: Uuid,
    pub candidate_id: Uuid,
    pub amount_cents: u64,
}

/// Confirm a pending payment via M-Pesa reference.
#[derive(Debug, Deserialize)]
pub struct ConfirmPayment {
    pub mpesa_ref: String,
}

/// Register a party symbol and colour (M67).
#[derive(Debug, Deserialize)]
pub struct RegisterSymbol {
    pub party_id: Uuid,
    pub symbol_name: String,
    pub symbol_url: String,
    pub colour_hex: String,
}
