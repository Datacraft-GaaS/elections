//! Request and response types mirroring the contract schemas
//! (`packages/openapi-contracts/bvi/openapi.yaml`). Field names are the wire
//! names, so no serde renaming.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct SessionCreate {
    pub station_code: String,
    pub device_serial: String,
    pub election_id: String,
}

/// BviSession contract schema; also the store row shape.
#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub session_id: String,
    pub station_code: String,
    pub device_serial: String,
    pub election_id: String,
    pub status: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub identifications_count: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CloseSessionRequest {
    pub closed_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Identification
// ---------------------------------------------------------------------------

/// Contract decisions (IdentificationCreate.decision).
pub const DECISIONS: [&str; 4] = ["match", "no_match", "fallback_used", "ambiguous"];

/// Modalities accepted by the identification_events mirror (V3 CHECK).
/// The contract omits a modality field, so devices may extend; the mirror
/// table enforces the domain, defaulting to fingerprint.
pub const MODALITIES: [&str; 4] = ["fingerprint", "iris", "face", "nfc"];

#[derive(Debug, Clone, Deserialize)]
pub struct IdentificationCreate {
    pub session_id: String,
    pub voter_id: String,
    pub biometric_ref: String,
    pub match_score: f64,
    pub decision: String,
    pub identified_at: DateTime<Utc>,
    pub device_serial: String,
    /// Optional extension: biometric modality used. Defaults to fingerprint.
    pub modality: Option<String>,
}

/// Identification contract schema; also the store row shape.
#[derive(Debug, Clone, Serialize)]
pub struct Identification {
    pub identification_id: String,
    pub session_id: String,
    pub voter_id: String,
    pub biometric_ref: String,
    pub match_score: f64,
    pub decision: String,
    pub identified_at: DateTime<Utc>,
    pub device_serial: String,
}

/// Map a contract decision onto the mirror table outcome domain.
pub fn decision_to_outcome(decision: &str) -> Option<&'static str> {
    match decision {
        "match" => Some("matched"),
        "no_match" => Some("no_match"),
        "fallback_used" => Some("fallback_used"),
        "ambiguous" => Some("ambiguous"),
        _ => None,
    }
}

/// Inverse of [decision_to_outcome] for responses.
pub fn outcome_to_decision(outcome: &str) -> &str {
    match outcome {
        "matched" => "match",
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Telemetry / distress
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryCreate {
    pub device_serial: String,
    pub station_code: String,
    pub battery_pct: i32,
    pub signal_strength_dbm: Option<f64>,
    pub temperature_c: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistressCreate {
    pub station_code: String,
    pub device_serial: String,
    pub distress_type: String,
    pub details: Option<String>,
    pub raised_at: DateTime<Utc>,
}

pub const DISTRESS_TYPES: [&str; 6] = [
    "officer_duress",
    "device_tamper",
    "power_failure",
    "network_failure",
    "physical_intrusion",
    "other",
];

/// Distress contract schema; also the store row shape.
#[derive(Debug, Clone, Serialize)]
pub struct Distress {
    pub distress_id: String,
    pub station_code: String,
    pub device_serial: String,
    pub distress_type: String,
    pub details: Option<String>,
    pub raised_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Station config
// ---------------------------------------------------------------------------

/// Station config row from kit_configs (V2 + V14 delivery fields).
#[derive(Debug, Clone)]
pub struct StationConfig {
    pub station_code: String,
    pub version: i32,
    pub bundle_id: String,
    pub match_threshold: f64,
    pub allowed_modalities: Vec<String>,
    pub timeout_s: i32,
    pub fallback_allowed: bool,
}

/// BviConfig contract schema.
#[derive(Debug, Clone, Serialize)]
pub struct StationConfigResponse {
    pub station_code: String,
    pub election_id: String,
    pub match_threshold: f64,
    pub allowed_modalities: Vec<String>,
    pub timeout_s: i32,
    pub fallback_allowed: bool,
}

// ---------------------------------------------------------------------------
// Kits
// ---------------------------------------------------------------------------

pub const REVOKE_REASONS: [&str; 5] =
    ["stolen", "tampered", "compromised", "defective", "reassigned"];

#[derive(Debug, Clone, Deserialize)]
pub struct RevokeKitRequest {
    pub reason: String,
    pub officer_id: String,
}

/// Revoke response body (inline schema on revokeKit).
#[derive(Debug, Clone, Serialize)]
pub struct RevokedKit {
    pub serial: String,
    pub revoked_at: DateTime<Utc>,
    pub reason: String,
}

/// KitHealth contract schema.
#[derive(Debug, Clone, Serialize)]
pub struct KitHealth {
    pub serial: String,
    pub station_code: Option<String>,
    pub battery_pct: Option<i32>,
    pub online: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub identification_count_today: i64,
    pub error_count_today: i64,
}

/// KitHealthPage contract schema.
#[derive(Debug, Clone, Serialize)]
pub struct KitHealthPage {
    pub items: Vec<KitHealth>,
    pub total: i64,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct KitsHealthQuery {
    pub station_code: Option<String>,
    pub online: Option<bool>,
    pub cursor: Option<String>,
    pub page_size: u32,
}

impl KitsHealthQuery {
    pub fn effective_page_size(&self) -> u32 {
        self.page_size.clamp(1, 200).max(1)
    }
}

// ---------------------------------------------------------------------------
// Voter station lookup
// ---------------------------------------------------------------------------

/// VoterStation contract schema; also the store row shape.
#[derive(Debug, Clone, Serialize)]
pub struct VoterStation {
    pub voter_id: String,
    pub station_code: String,
    pub polling_station_name: String,
    pub constituency_code: String,
    pub county_code: String,
}

/// Audit context for voter station lookups (V7 voter_lookup_audit).
#[derive(Debug, Clone)]
pub struct LookupAuditCtx {
    pub actor_session_hash: String,
    pub ip_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_outcome_round_trip() {
        assert_eq!(decision_to_outcome("match"), Some("matched"));
        assert_eq!(outcome_to_decision("matched"), "match");
        assert_eq!(outcome_to_decision("fallback_used"), "fallback_used");
        assert_eq!(decision_to_outcome("nonsense"), None);
    }

    #[test]
    fn page_size_clamped() {
        let mut q = KitsHealthQuery { page_size: 999, ..Default::default() };
        assert_eq!(q.effective_page_size(), 200);
        q.page_size = 0;
        assert_eq!(q.effective_page_size(), 1);
    }
}
