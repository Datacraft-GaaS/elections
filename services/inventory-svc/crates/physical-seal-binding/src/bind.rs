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

//! bind_custody_event: binds a SealPhotoSignature to a custody event,
//! producing a BoundCustodyEvent with cryptographic attestation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::SealBindError;
use crate::seal_photo::SealPhotoSignature;
use crate::verify::verify_kit_signature;

/// The minimal custody-event fields needed for seal binding.
/// Production code passes the full domain::CustodyEvent via Into<CustodyEventRef>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEvent {
    pub event_id: String,
    /// The seal serial associated with this custody transfer.
    pub seal_id: String,
    /// The kit ID initiating the transfer.
    pub kit_id: String,
    /// HLC timestamp (milliseconds since Unix epoch).
    pub ts_hlc: i64,
    pub from_custodian: String,
    pub to_custodian: String,
    pub location: String,
}

/// A custody event with cryptographic seal-photo attestation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundCustodyEvent {
    pub event: CustodyEvent,
    pub photo_sig: SealPhotoSignature,
    pub bound_at: DateTime<Utc>,
}

/// Bind `event` to `photo_sig`, enforcing:
/// - seal_id match
/// - kit_id match
/// - kit_hsm_signature verifies over the binding message
/// - event timestamp >= photo captured_at (no retroactive photos)
pub fn bind_custody_event(
    event: CustodyEvent,
    photo_sig: SealPhotoSignature,
) -> Result<BoundCustodyEvent, SealBindError> {
    // seal_id consistency
    if event.seal_id != photo_sig.seal_id {
        return Err(SealBindError::SealIdMismatch {
            event: event.seal_id,
            photo: photo_sig.seal_id,
        });
    }

    // kit_id consistency
    if event.kit_id != photo_sig.kit_id {
        return Err(SealBindError::KitIdMismatch {
            event: event.kit_id,
            photo: photo_sig.kit_id,
        });
    }

    // Temporal order: custody event must not precede the photo
    let captured_ms = photo_sig.captured_at.timestamp_millis();
    if event.ts_hlc < captured_ms {
        return Err(SealBindError::TimestampOutOfOrder {
            event_ts: event.ts_hlc,
            captured_at: captured_ms,
        });
    }

    // Verify kit HSM signature
    verify_kit_signature(&photo_sig)?;

    let bound_at = Utc::now();
    info!(
        event_id = %event.event_id,
        seal_id = %event.seal_id,
        "seal photo bound to custody event"
    );

    Ok(BoundCustodyEvent {
        event,
        photo_sig,
        bound_at,
    })
}
