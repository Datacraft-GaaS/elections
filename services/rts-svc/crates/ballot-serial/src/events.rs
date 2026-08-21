//! Ballot-serial domain event constants and payloads (WP-3.3, M63).

use serde::{Deserialize, Serialize};

pub const EVENT_SERIAL_ASSIGNED: &str = "eel.ballot_serial.serial_assigned.v1";
pub const EVENT_BALLOT_ISSUED: &str = "eel.ballot_serial.serial_issued.v1";
pub const EVENT_BALLOT_SPOILT: &str = "eel.ballot_serial.ballot_spoilt.v1";
pub const EVENT_RECONCILIATION_STARTED: &str = "eel.ballot_serial.reconciliation_started.v1";
pub const EVENT_RECONCILIATION_COMPLETE: &str = "eel.ballot_serial.reconciliation_cleared.v1";
pub const EVENT_RECONCILIATION_DISCREPANCY: &str = "eel.ballot_serial.reconciliation_held.v1";

// ---- payloads ---------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialAssignedPayload {
    pub assignment_id: String,
    pub lot_id: String,
    pub station_code: String,
    pub serial_low: u64,
    pub serial_high: u64,
    pub assigned_at: String,
    pub assigned_by: String,
    /// Reference to the M39 manifest (FR-M63-003).
    pub manifest_ref: String,
    /// Ed25519 signature of the manifest (FR-M63-003).
    pub manifest_sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotIssuedPayload {
    pub receipt_id: String,
    pub assignment_id: String,
    pub station_code: String,
    pub clerk_id: String,
    pub received_at: String,
    pub receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotSpoiltPayload {
    pub station_code: String,
    pub spoilt_serial: u64,
    pub replacement_serial: Option<u64>,
    pub reason: String,
    pub recorded_at: String,
    pub clerk_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationStartedPayload {
    pub reconciliation_id: String,
    pub station_code: String,
    pub election_id: String,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationCompletePayload {
    pub reconciliation_id: String,
    pub station_code: String,
    pub issued_count: u64,
    pub spoilt_count: u64,
    pub cast_count: u64,
    pub unused_count: u64,
    pub completed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationDiscrepancyPayload {
    pub reconciliation_id: String,
    pub station_code: String,
    pub flag_reason: String,
    /// Blocks M10 result publication for this station (FR-M63-025).
    pub result_hold: bool,
    pub raised_at: String,
}
