//! Domain types for mobile telemetry ingest.
//!
//! All enums implement `Display` + string conversion for SQL round-trips.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Signal class as stored in eel_telemetry_signals.class.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalClass {
    Location,
    Time,
    Activity,
    ProximalRfid,
    Movement,
    LoginLogout,
    Read,
    Write,
    Battery,
    LockUnlock,
    Drop,
    Network,
    Attestation,
}

impl SignalClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Location => "location",
            Self::Time => "time",
            Self::Activity => "activity",
            Self::ProximalRfid => "proximal_rfid",
            Self::Movement => "movement",
            Self::LoginLogout => "login_logout",
            Self::Read => "read",
            Self::Write => "write",
            Self::Battery => "battery",
            Self::LockUnlock => "lock_unlock",
            Self::Drop => "drop",
            Self::Network => "network",
            Self::Attestation => "attestation",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "location" => Some(Self::Location),
            "time" => Some(Self::Time),
            "activity" => Some(Self::Activity),
            "proximal_rfid" => Some(Self::ProximalRfid),
            "movement" => Some(Self::Movement),
            "login_logout" => Some(Self::LoginLogout),
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "battery" => Some(Self::Battery),
            "lock_unlock" => Some(Self::LockUnlock),
            "drop" => Some(Self::Drop),
            "network" => Some(Self::Network),
            "attestation" => Some(Self::Attestation),
            _ => None,
        }
    }
}

impl std::fmt::Display for SignalClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Alert class as stored in eel_telemetry_alerts.class.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertClass {
    Drop,
    GeoFenceBreach,
    SessionHijack,
    BatteryLow,
    RfidAbsence,
    LoginTimeAnomaly,
    ReadHeavy,
    CoordinatedAnomaly,
    ChassisTamper,
}

impl AlertClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Drop => "drop",
            Self::GeoFenceBreach => "geo_fence_breach",
            Self::SessionHijack => "session_hijack",
            Self::BatteryLow => "battery_low",
            Self::RfidAbsence => "rfid_absence",
            Self::LoginTimeAnomaly => "login_time_anomaly",
            Self::ReadHeavy => "read_heavy",
            Self::CoordinatedAnomaly => "coordinated_anomaly",
            Self::ChassisTamper => "chassis_tamper",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "drop" => Some(Self::Drop),
            "geo_fence_breach" => Some(Self::GeoFenceBreach),
            "session_hijack" => Some(Self::SessionHijack),
            "battery_low" => Some(Self::BatteryLow),
            "rfid_absence" => Some(Self::RfidAbsence),
            "login_time_anomaly" => Some(Self::LoginTimeAnomaly),
            "read_heavy" => Some(Self::ReadHeavy),
            "coordinated_anomaly" => Some(Self::CoordinatedAnomaly),
            "chassis_tamper" => Some(Self::ChassisTamper),
            _ => None,
        }
    }
}

impl std::fmt::Display for AlertClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Alert severity level.
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

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "info" => Some(Self::Info),
            "warn" => Some(Self::Warn),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Session exit reason.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    Normal,
    Timeout,
    Revoked,
    Duress,
    Crash,
}

impl ExitReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Timeout => "timeout",
            Self::Revoked => "revoked",
            Self::Duress => "duress",
            Self::Crash => "crash",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "timeout" => Some(Self::Timeout),
            "revoked" => Some(Self::Revoked),
            "duress" => Some(Self::Duress),
            "crash" => Some(Self::Crash),
            _ => None,
        }
    }
}

impl std::fmt::Display for ExitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Motion classification for activity frames.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionClass {
    Stationary,
    Walking,
    Running,
    Vehicle,
    Unknown,
}

impl MotionClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stationary => "stationary",
            Self::Walking => "walking",
            Self::Running => "running",
            Self::Vehicle => "vehicle",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "stationary" => Some(Self::Stationary),
            "walking" => Some(Self::Walking),
            "running" => Some(Self::Running),
            "vehicle" => Some(Self::Vehicle),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

impl std::fmt::Display for MotionClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Row structs
// ---------------------------------------------------------------------------

/// Row in eel_telemetry_devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryDevice {
    pub device_id: Uuid,
    pub kit_id: String,
    pub app_id: String,
    pub persona_role: String,
    pub spiffe_id: Option<String>,
    pub hmac_key_hsm_ref: String,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
}

/// Row in eel_telemetry_sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySession {
    pub session_id: Uuid,
    pub device_id: Uuid,
    pub actor_svid: String,
    pub entered_at: DateTime<Utc>,
    pub exited_at: Option<DateTime<Utc>>,
    pub exit_reason: Option<ExitReason>,
    pub signals_count: i64,
}

/// Row in eel_telemetry_signals (partition table).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySignal {
    pub signal_id: Uuid,
    pub device_id: Uuid,
    pub session_id: Option<Uuid>,
    pub class: SignalClass,
    pub payload: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
    pub ingested_at: DateTime<Utc>,
    pub monotonic_counter: i64,
    pub prev_hash: Option<Vec<u8>>,
    pub this_hash: Vec<u8>,
    pub hmac_signature: Vec<u8>,
    pub server_signature: Option<Vec<u8>>,
    pub trace_id: Option<Vec<u8>>,
    pub span_id: Option<Vec<u8>>,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
}

/// Row in eel_telemetry_activity_frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryActivityFrame {
    pub frame_id: Uuid,
    pub device_id: Uuid,
    pub second_bucket: DateTime<Utc>,
    pub accel_x_avg: f32,
    pub accel_y_avg: f32,
    pub accel_z_avg: f32,
    pub accel_max_g: f32,
    pub gyro_x_avg: f32,
    pub gyro_y_avg: f32,
    pub gyro_z_avg: f32,
    pub motion_class: MotionClass,
    pub battery_pct: i16,
    pub battery_temp_c: f32,
    pub network_class: Option<String>,
    pub lat_coarse: Option<f32>,
    pub lon_coarse: Option<f32>,
}

/// Row in eel_telemetry_geo_dwells.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryGeoDwell {
    pub dwell_id: Uuid,
    pub device_id: Uuid,
    pub polygon_ref: String,
    pub entered_at: DateTime<Utc>,
    pub exited_at: Option<DateTime<Utc>>,
    pub dwell_seconds: Option<i32>,
}

/// Row in eel_telemetry_alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryAlert {
    pub alert_id: Uuid,
    pub device_id: Uuid,
    pub class: AlertClass,
    pub severity: Severity,
    pub detected_at: DateTime<Utc>,
    pub triggering_signal: Option<Uuid>,
    pub payload: serde_json::Value,
    pub ecc_dispatched: bool,
    pub ecc_dispatched_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_note: Option<String>,
}

/// Row in eel_telemetry_snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub snapshot_id: Uuid,
    pub snapshot_date: NaiveDate,
    pub device_id: Option<Uuid>,
    pub device_merkle_root: Option<Vec<u8>>,
    pub snapshot_taken_at: DateTime<Utc>,
    pub tlog_leaf_index: Option<i64>,
    pub tlog_root_hash: Option<Vec<u8>>,
    pub inclusion_proof: Option<Vec<u8>>,
}
