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

//! Cover tag issuance and pairing to a ballot box body.

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::domain::{CoverTag, TamperState};
use crate::error::RfidError;

// ---------------------------------------------------------------------------
// Issuance
// ---------------------------------------------------------------------------

/// Request to issue a new cover RFID tag.
#[derive(Debug, Clone)]
pub struct IssueTagRequest {
    pub manufacturer: String,
    pub lot_id: Uuid,
    pub chip_family: String,
    /// Opaque HSM key-slot identifier provisioned at manufacture.
    pub tag_secret_hsm_key_id: String,
}

/// Generate a new 96-bit EPC for this tag.
///
/// EPC layout (12 bytes):
///   [0]   Header = 0x30 (SGTIN-96)
///   [1]   Filter + partition
///   [2..9] Company prefix + item reference (derived from lot_id bytes 0..8)
///   [10..11] Serial counter (bytes 6..8 of tag_id)
fn generate_epc(tag_id: Uuid, lot_id: Uuid) -> [u8; 12] {
    let tag_bytes = tag_id.as_bytes();
    let lot_bytes = lot_id.as_bytes();
    let mut epc = [0u8; 12];
    epc[0] = 0x30; // SGTIN-96 header
    epc[1] = 0x34; // Filter=3 (ballot material), partition=4
                   // Bytes 2..10: mix of lot prefix and tag serial
    epc[2..10].copy_from_slice(&lot_bytes[0..8]);
    // XOR with tag identity bytes to make it unique per tag
    for i in 2..10 {
        epc[i] ^= tag_bytes[i - 2];
    }
    epc[10] = tag_bytes[14];
    epc[11] = tag_bytes[15];
    epc
}

/// Issue a new cover RFID tag, generating a UUID7 tag_id and 96-bit EPC.
///
/// In production, `tag_secret_hsm_key_id` is set by the HSM provisioning
/// step that runs at the printer facility. The per-tag secret never leaves
/// the HSM; only the key-slot identifier is stored.
pub fn issue_tag(req: IssueTagRequest) -> Result<CoverTag, RfidError> {
    let tag_id = Uuid::now_v7();
    let epc_bytes = generate_epc(tag_id, req.lot_id);

    let tag = CoverTag {
        tag_id,
        epc: epc_bytes.to_vec(),
        manufacturer: req.manufacturer,
        lot_id: req.lot_id,
        chip_family: req.chip_family,
        tag_secret_hsm_key_id: req.tag_secret_hsm_key_id,
        issued_at: Utc::now(),
        paired_with_box: None,
        paired_at: None,
        tamper_state: TamperState::Intact,
        tamper_detected_at: None,
        tamper_detected_by: None,
        retired_at: None,
    };

    info!(
        tag_id = %tag.tag_id,
        epc = hex::encode(&tag.epc),
        lot_id = %tag.lot_id,
        "cover RFID tag issued"
    );

    Ok(tag)
}

// ---------------------------------------------------------------------------
// Pairing
// ---------------------------------------------------------------------------

/// Request to pair a cover tag to a specific ballot box body.
#[derive(Debug, Clone)]
pub struct PairToBoxRequest {
    /// The UUID of the box body (from `assets.asset_id`).
    pub box_body_id: Uuid,
    /// Lot ID of the box body; must match the tag's lot_id.
    pub box_lot_id: Uuid,
    /// EPC of the box-body tag scanned during dual-tag-scan validation.
    pub box_body_epc: Vec<u8>,
}

/// Bind a cover tag to a box body, enforcing:
/// - tag must not be already paired
/// - tag must not be retired
/// - lot IDs must match (cover and box body from the same print run)
pub fn pair_to_box(tag: CoverTag, req: PairToBoxRequest) -> Result<CoverTag, RfidError> {
    if tag.retired_at.is_some() {
        return Err(RfidError::TagRetired(tag.tag_id.to_string()));
    }

    if let Some(existing_box) = tag.paired_with_box {
        return Err(RfidError::AlreadyPaired {
            tag_id: tag.tag_id.to_string(),
            box_id: existing_box.to_string(),
        });
    }

    if tag.lot_id != req.box_lot_id {
        return Err(RfidError::LotMismatch {
            tag_lot: tag.lot_id.to_string(),
            box_lot: req.box_lot_id.to_string(),
        });
    }

    let paired = CoverTag {
        paired_with_box: Some(req.box_body_id),
        paired_at: Some(Utc::now()),
        ..tag
    };

    info!(
        tag_id = %paired.tag_id,
        box_body_id = %req.box_body_id,
        "cover RFID tag paired to box body"
    );

    Ok(paired)
}
