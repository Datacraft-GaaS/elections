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
