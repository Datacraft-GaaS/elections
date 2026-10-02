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

//! CloudEvent type constants for the diaspora module (WP-5.8, M24).
//!
//! All event names follow the eel.diaspora.<noun>.<verb>.v1 convention.

/// A Form 34A was captured at an embassy station and HSM-signed.
pub const FORM_CAPTURED: &str = "eel.diaspora.form_captured.v1";

/// A Form 34A was successfully transmitted to Nairobi HQ.
pub const FORM_TRANSMITTED: &str = "eel.diaspora.form_transmitted.v1";

/// A transmission batch could not be sent; delivery deferred.
pub const TRANSMISSION_DEFERRED: &str = "eel.diaspora.transmission_deferred.v1";

/// A pouch was sealed for physical transport.
pub const POUCH_SEALED: &str = "eel.diaspora.pouch_sealed.v1";

/// A pouch custody was transferred to a new officer.
pub const POUCH_TRANSFERRED: &str = "eel.diaspora.pouch_transferred.v1";

/// A diaspora roll was frozen (no further changes permitted).
pub const ROLL_FROZEN: &str = "eel.diaspora.roll_frozen.v1";

/// SPIFFE source URI for the diaspora service.
pub const SOURCE_URI: &str = "spiffe://eel.local/service/diaspora";

/// Default Kafka topic for diaspora events.
pub const TOPIC: &str = "eel.diaspora";

/// Build an OutboxEnvelope for the given event type, stream id, and payload.
/// All eel_* extension fields are set to safe defaults.
pub fn make_envelope(
    event_type: &str,
    stream_id: uuid::Uuid,
    subject: Option<&str>,
    data: serde_json::Value,
) -> eel_outbox_rs::OutboxEnvelope {
    use chrono::Utc;
    let now = Utc::now();
    eel_outbox_rs::OutboxEnvelope {
        id: uuid::Uuid::now_v7().to_string(),
        event_type: event_type.to_owned(),
        source: SOURCE_URI.to_owned(),
        subject: subject.map(ToOwned::to_owned),
        time: now.to_rfc3339(),
        eel_stream_id: stream_id.to_string(),
        eel_seq: 0,
        eel_hlc: now.timestamp_millis(),
        eel_prev_hash: None,
        eel_correlation_id: None,
        eel_causation_id: None,
        eel_producer_key_id: None,
        eel_signature: None,
        eel_election_id: None,
        eel_data_class: Some("B".to_owned()),
        data,
        topic: TOPIC.to_owned(),
        kafka_key: stream_id.to_string(),
        aggregate_type: "diaspora".to_owned(),
        aggregate_id: stream_id.to_string(),
    }
}
