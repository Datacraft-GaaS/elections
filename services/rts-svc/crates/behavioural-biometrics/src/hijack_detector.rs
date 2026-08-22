//! Score above threshold emits eel.rts.session_hijack_suspected.v1 event.

use serde::Serialize;

use crate::session::SessionProfile;

#[derive(Debug, Clone, Serialize)]
pub struct HijackEvent {
    pub event_type: &'static str,
    pub officer_id: String,
    pub session_id: String,
    pub mahalanobis_distance: f64,
    pub threshold: f64,
    pub timestamp: String,
}

pub struct HijackDetector {
    pub threshold: f64,
}

impl HijackDetector {
    /// Construct a detector. Default threshold is 5.0.
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Returns Some(HijackEvent) if distance exceeds the threshold.
    pub fn check(&self, profile: &SessionProfile, distance: f64) -> Option<HijackEvent> {
        if distance <= self.threshold {
            return None;
        }
        Some(HijackEvent {
            event_type: "eel.rts.session_hijack_suspected.v1",
            officer_id: profile.officer_id.clone(),
            session_id: profile.session_id.clone(),
            mahalanobis_distance: distance,
            threshold: self.threshold,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Serialise the event to a JSON string.
    pub fn emit_event(event: &HijackEvent) -> String {
        serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string())
    }
}
