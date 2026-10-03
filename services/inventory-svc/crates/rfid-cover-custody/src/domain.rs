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

//! Domain types for RFID cover custody.
//!
//! All enums implement `Display` + `FromStr` for SQL round-trips.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// All states a cover tag can occupy in its lifecycle.
///
/// The FSM is linear (MANUFACTURED -> ... -> STORAGE_ARCHIVE -> DESTROYED)
/// with QUARANTINE as an absorbing sink reachable from any state on tamper.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CoverState {
    Manufactured,
    Printer,
    Warehouse,
    Dispatched,
    InTransit,
    StationReceived,
    StationOpened,
    VotingActive,
    StationClosed,
    SealedReturn,
    TallyArrived,
    CountComplete,
    StorageArchive,
    Destroyed,
    Quarantine,
}

impl CoverState {
    /// Returns the string representation stored in Postgres.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manufactured => "MANUFACTURED",
            Self::Printer => "PRINTER",
            Self::Warehouse => "WAREHOUSE",
            Self::Dispatched => "DISPATCHED",
            Self::InTransit => "IN_TRANSIT",
            Self::StationReceived => "STATION_RECEIVED",
            Self::StationOpened => "STATION_OPENED",
            Self::VotingActive => "VOTING_ACTIVE",
            Self::StationClosed => "STATION_CLOSED",
            Self::SealedReturn => "SEALED_RETURN",
            Self::TallyArrived => "TALLY_ARRIVED",
            Self::CountComplete => "COUNT_COMPLETE",
            Self::StorageArchive => "STORAGE_ARCHIVE",
            Self::Destroyed => "DESTROYED",
            Self::Quarantine => "QUARANTINE",
        }
    }

    /// Parse from the Postgres string representation.
    ///
    /// Not `FromStr::from_str`: it returns an `Option` rather than a
    /// `Result`, and the name mirrors the database column representation.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "MANUFACTURED" => Some(Self::Manufactured),
            "PRINTER" => Some(Self::Printer),
            "WAREHOUSE" => Some(Self::Warehouse),
            "DISPATCHED" => Some(Self::Dispatched),
            "IN_TRANSIT" => Some(Self::InTransit),
            "STATION_RECEIVED" => Some(Self::StationReceived),
            "STATION_OPENED" => Some(Self::StationOpened),
            "VOTING_ACTIVE" => Some(Self::VotingActive),
            "STATION_CLOSED" => Some(Self::StationClosed),
            "SEALED_RETURN" => Some(Self::SealedReturn),
            "TALLY_ARRIVED" => Some(Self::TallyArrived),
            "COUNT_COMPLETE" => Some(Self::CountComplete),
            "STORAGE_ARCHIVE" => Some(Self::StorageArchive),
            "DESTROYED" => Some(Self::Destroyed),
            "QUARANTINE" => Some(Self::Quarantine),
            _ => None,
        }
    }

    /// Returns all non-terminal, non-absorbing states (for property tests).
    pub fn all_main() -> Vec<Self> {
        vec![
            Self::Manufactured,
            Self::Printer,
            Self::Warehouse,
            Self::Dispatched,
            Self::InTransit,
            Self::StationReceived,
            Self::StationOpened,
            Self::VotingActive,
            Self::StationClosed,
            Self::SealedReturn,
            Self::TallyArrived,
            Self::CountComplete,
            Self::StorageArchive,
            Self::Destroyed,
            Self::Quarantine,
        ]
    }

    /// True if no further transitions are allowed except QUARANTINE.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Destroyed | Self::Quarantine)
    }
}

impl std::fmt::Display for CoverState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// RFID reader hardware type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReaderType {
    WarehouseGate,
    TransitCheckpoint,
    StationArrival,
    KitHandheld,
    RoDesktop,
    ArchiveGate,
    AirlineCargo,
    MfaCustoms,
}

impl ReaderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WarehouseGate => "warehouse_gate",
            Self::TransitCheckpoint => "transit_checkpoint",
            Self::StationArrival => "station_arrival",
            Self::KitHandheld => "kit_handheld",
            Self::RoDesktop => "ro_desktop",
            Self::ArchiveGate => "archive_gate",
            Self::AirlineCargo => "airline_cargo",
            Self::MfaCustoms => "mfa_customs",
        }
    }

    /// Parse from the snake_case string representation.
    ///
    /// Not `FromStr::from_str`: it returns an `Option` rather than a `Result`,
    /// and the name mirrors the database column representation.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "warehouse_gate" => Some(Self::WarehouseGate),
            "transit_checkpoint" => Some(Self::TransitCheckpoint),
            "station_arrival" => Some(Self::StationArrival),
            "kit_handheld" => Some(Self::KitHandheld),
            "ro_desktop" => Some(Self::RoDesktop),
            "archive_gate" => Some(Self::ArchiveGate),
            "airline_cargo" => Some(Self::AirlineCargo),
            "mfa_customs" => Some(Self::MfaCustoms),
            _ => None,
        }
    }
}

impl std::fmt::Display for ReaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Physical tamper state of the tag antenna.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TamperState {
    Intact,
    CoverLifted,
    AntennaBroken,
    CloneDetected,
    Killed,
}

impl TamperState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Intact => "intact",
            Self::CoverLifted => "cover_lifted",
            Self::AntennaBroken => "antenna_broken",
            Self::CloneDetected => "clone_detected",
            Self::Killed => "killed",
        }
    }
}

impl std::fmt::Display for TamperState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Alert severity for tamper events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warn,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Critical => "critical",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Row structs
// ---------------------------------------------------------------------------

/// One manufactured cover RFID tag. Row in `eel_rfid_cover_tags`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverTag {
    pub tag_id: Uuid,
    /// 96-bit EPC as raw bytes (12 bytes).
    pub epc: Vec<u8>,
    pub manufacturer: String,
    pub lot_id: Uuid,
    pub chip_family: String,
    /// Identifier of the HSM key slot holding the per-tag secret.
    pub tag_secret_hsm_key_id: String,
    pub issued_at: DateTime<Utc>,
    pub paired_with_box: Option<Uuid>,
    pub paired_at: Option<DateTime<Utc>>,
    pub tamper_state: TamperState,
    pub tamper_detected_at: Option<DateTime<Utc>>,
    pub tamper_detected_by: Option<String>,
    pub retired_at: Option<DateTime<Utc>>,
}

/// One RFID read event. Row in `eel_rfid_cover_reads`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverRead {
    pub read_id: Uuid,
    pub tag_id: Uuid,
    pub reader_id: String,
    pub reader_type: ReaderType,
    pub read_at: DateTime<Utc>,
    pub geo_lat: Option<f64>,
    pub geo_lon: Option<f64>,
    pub geo_accuracy_m: Option<f64>,
    /// 16-byte nonce sent to the tag.
    pub tag_challenge: Option<Vec<u8>>,
    /// Tag HMAC-SHA256 response.
    pub tag_response: Option<Vec<u8>>,
    pub challenge_valid: bool,
    pub tamper_indicator: bool,
    pub reader_signature: Option<Vec<u8>>,
    pub custody_event_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
}

/// One lifecycle FSM transition. Row in `eel_cover_state_transitions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverStateTransition {
    pub transition_id: Uuid,
    pub tag_id: Uuid,
    pub from_state: CoverState,
    pub to_state: CoverState,
    pub triggered_by_read: Option<Uuid>,
    pub officer_id: Option<Uuid>,
    pub officer_svid: Option<String>,
    pub officer_signature: Option<Vec<u8>>,
    /// BLAKE3 hash of the preceding transition (None for the first row).
    pub prev_hash: Option<Vec<u8>>,
    /// BLAKE3(prev_hash || canonical_json(this_transition)).
    pub this_hash: Vec<u8>,
    pub transitioned_at: DateTime<Utc>,
}

/// One tamper alert. Row in `eel_cover_tamper_alerts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverTamperAlert {
    pub alert_id: Uuid,
    pub tag_id: Uuid,
    pub detected_at: DateTime<Utc>,
    pub triggering_read: Option<Uuid>,
    pub expected_state: Option<String>,
    pub observed_state: Option<String>,
    pub severity: Severity,
    pub ecc_dispatched: bool,
    pub ecc_dispatched_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_note: Option<String>,
}
