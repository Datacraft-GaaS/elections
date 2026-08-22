//! CloudEvent type constants and envelope builders for telemetry ingest.

use serde_json::{json, Value};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Event type constants
// ---------------------------------------------------------------------------

pub const TELEMETRY_SIGNAL_V1: &str = "eel.field.telemetry_signal.v1";
pub const TELEMETRY_SESSION_OPENED_V1: &str = "eel.field.telemetry_session_opened.v1";
pub const TELEMETRY_SESSION_CLOSED_V1: &str = "eel.field.telemetry_session_closed.v1";
pub const TELEMETRY_ALERT_V1: &str = "eel.field.telemetry_alert.v1";

/// Source URI for events emitted by this crate.
const SOURCE: &str = "/eel/field-mobile-svc/telemetry-ingest";

// ---------------------------------------------------------------------------
// Envelope builders
// ---------------------------------------------------------------------------

/// Build the CloudEvent envelope for a signal ingested event.
pub fn telemetry_signal_data(signal_id: Uuid, device_id: Uuid, class: &str) -> Value {
    json!({
        "type": TELEMETRY_SIGNAL_V1,
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": device_id.to_string(),
        "data": {
            "signal_id": signal_id.to_string(),
            "device_id": device_id.to_string(),
            "class": class,
        }
    })
}

/// Build the CloudEvent envelope for a session-opened event.
pub fn telemetry_session_opened_data(session_id: Uuid, device_id: Uuid, actor_svid: &str) -> Value {
    json!({
        "type": TELEMETRY_SESSION_OPENED_V1,
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": device_id.to_string(),
        "data": {
            "session_id": session_id.to_string(),
            "device_id": device_id.to_string(),
            "actor_svid": actor_svid,
        }
    })
}

/// Build the CloudEvent envelope for a session-closed event.
pub fn telemetry_session_closed_data(
    session_id: Uuid,
    device_id: Uuid,
    exit_reason: &str,
) -> Value {
    json!({
        "type": TELEMETRY_SESSION_CLOSED_V1,
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": device_id.to_string(),
        "data": {
            "session_id": session_id.to_string(),
            "device_id": device_id.to_string(),
            "exit_reason": exit_reason,
        }
    })
}

/// Build the CloudEvent envelope for an alert raised event.
pub fn telemetry_alert_data(alert_id: Uuid, device_id: Uuid, class: &str, severity: &str) -> Value {
    json!({
        "type": TELEMETRY_ALERT_V1,
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": device_id.to_string(),
        "data": {
            "alert_id": alert_id.to_string(),
            "device_id": device_id.to_string(),
            "class": class,
            "severity": severity,
        }
    })
}
