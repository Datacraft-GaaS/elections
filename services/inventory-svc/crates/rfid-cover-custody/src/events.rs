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

//! CloudEvent type constants and envelope builders for RFID cover custody.

use serde_json::{json, Value};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Event type constants
// ---------------------------------------------------------------------------

pub const COVER_TAG_ISSUED_V1: &str = "eel.inventory.cover_tag_issued.v1";
pub const COVER_PAIRED_V1: &str = "eel.inventory.cover_paired.v1";
pub const COVER_READ_RECORDED_V1: &str = "eel.inventory.cover_read_recorded.v1";
pub const COVER_STATE_TRANSITIONED_V1: &str = "eel.inventory.cover_state_transitioned.v1";
pub const COVER_TAMPER_V1: &str = "eel.inventory.cover_tamper.v1";

/// Source URI for events emitted by this crate.
const SOURCE: &str = "/eel/inventory-svc/rfid-cover-custody";

// ---------------------------------------------------------------------------
// Envelope builders
// ---------------------------------------------------------------------------

/// Build the CloudEvent data payload for a tag-issued event.
pub fn cover_tag_issued_data(tag_id: Uuid, lot_id: Uuid, epc_hex: &str) -> Value {
    json!({
        "type": COVER_TAG_ISSUED_V1,
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": tag_id.to_string(),
        "data": {
            "tag_id": tag_id.to_string(),
            "lot_id": lot_id.to_string(),
            "epc": epc_hex,
        }
    })
}

/// Build the CloudEvent data payload for a cover-paired event.
pub fn cover_paired_data(tag_id: Uuid, box_body_id: Uuid) -> Value {
    json!({
        "type": COVER_PAIRED_V1,
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": tag_id.to_string(),
        "data": {
            "tag_id": tag_id.to_string(),
            "box_body_id": box_body_id.to_string(),
        }
    })
}

/// Build the CloudEvent data payload for a read-recorded event.
pub fn cover_read_recorded_data(
    read_id: Uuid,
    tag_id: Uuid,
    reader_id: &str,
    challenge_valid: bool,
) -> Value {
    json!({
        "type": COVER_READ_RECORDED_V1,
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": tag_id.to_string(),
        "data": {
            "read_id": read_id.to_string(),
            "tag_id": tag_id.to_string(),
            "reader_id": reader_id,
            "challenge_valid": challenge_valid,
        }
    })
}

/// Build the CloudEvent data payload for a state-transition event.
pub fn cover_state_transitioned_data(
    transition_id: Uuid,
    tag_id: Uuid,
    from_state: &str,
    to_state: &str,
) -> Value {
    json!({
        "type": COVER_STATE_TRANSITIONED_V1,
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": tag_id.to_string(),
        "data": {
            "transition_id": transition_id.to_string(),
            "tag_id": tag_id.to_string(),
            "from_state": from_state,
            "to_state": to_state,
        }
    })
}

/// Build the CloudEvent data payload for a tamper event.
pub fn cover_tamper_data(alert_id: Uuid, tag_id: Uuid, severity: &str) -> Value {
    json!({
        "type": COVER_TAMPER_V1,
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": tag_id.to_string(),
        "data": {
            "alert_id": alert_id.to_string(),
            "tag_id": tag_id.to_string(),
            "severity": severity,
        }
    })
}
