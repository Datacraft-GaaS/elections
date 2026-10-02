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

//! Event-type constants and payload structs for M27 intent preservation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const EVENT_PREMARK_VALIDATION: &str = "eel.support.intent.premark_validation.v1";
pub const EVENT_SPOILT_BALLOT: &str = "eel.support.intent.spoilt_ballot.v1";
pub const EVENT_ADJUDICATION: &str = "eel.support.intent.adjudication.v1";
pub const EVENT_REJECTION_RATE_ALERT: &str = "eel.support.intent.rejection_rate_alert.v1";
pub const EVENT_ADJUDICATION_DISABLE_TOGGLED: &str =
    "eel.support.intent.adjudication_disable_toggled.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreMarkValidationPayload {
    pub event_id: Uuid,
    pub station_code: String,
    pub contest_id: String,
    /// Inference outcome; image MUST be deleted <= 1 s after this event.
    pub inference_outcome: String,
    pub model_version: String,
    pub device_id: String,
    /// Elapsed ms from capture to delete; must be <= 1000.
    pub image_delete_elapsed_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoiltBallotPayload {
    pub event_id: Uuid,
    pub station_code: String,
    pub contest_id: String,
    pub reason_code: String,
    pub officer_id: String,
    pub witness_ids: Vec<String>,
    pub replacement_issued: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjudicationPayload {
    pub event_id: Uuid,
    pub ballot_id: String,
    pub station_code: String,
    pub contest_id: String,
    pub adjudicator_id: String,
    pub reason_code: String,
    pub decision: String,
    pub photo_evidence_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionRateAlertPayload {
    pub alert_id: Uuid,
    pub station_code: String,
    pub contest_id: String,
    pub rejection_rate: f64,
    pub national_mean: f64,
    pub z_score: f64,
    pub alerted_at: String,
}
