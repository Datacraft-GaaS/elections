//! REST API stubs for seal-binding endpoints.
//!
//! POST /v1/custody/bind-seal
//! GET  /v1/custody/{id}/seal-verification
//!
//! Wire these routes into the inventory-svc Axum router.

use serde::{Deserialize, Serialize};

use crate::bind::{bind_custody_event, BoundCustodyEvent, CustodyEvent};
use crate::error::SealBindError;
use crate::seal_photo::SealPhotoSignature;

/// Request body for POST /v1/custody/bind-seal.
#[derive(Debug, Deserialize)]
pub struct BindSealRequest {
    pub event: CustodyEvent,
    pub photo_sig: SealPhotoSignature,
}

/// Response body for POST /v1/custody/bind-seal.
#[derive(Debug, Serialize)]
pub struct BindSealResponse {
    pub bound: BoundCustodyEvent,
}

/// Process a bind-seal request; returns the bound event or a rejection reason.
pub fn handle_bind_seal(req: BindSealRequest) -> Result<BindSealResponse, SealBindError> {
    let bound = bind_custody_event(req.event, req.photo_sig)?;
    Ok(BindSealResponse { bound })
}

/// Request parameters for GET /v1/custody/{id}/seal-verification.
#[derive(Debug, Deserialize)]
pub struct VerifySealParams {
    pub event_id: String,
}

/// Response body for GET /v1/custody/{id}/seal-verification.
#[derive(Debug, Serialize)]
pub struct VerifySealResponse {
    pub event_id: String,
    pub seal_id: String,
    pub photo_sha256: String,
    pub verified: bool,
    pub failure_reason: Option<String>,
}

impl VerifySealResponse {
    pub fn ok(bound: &BoundCustodyEvent) -> Self {
        Self {
            event_id: bound.event.event_id.clone(),
            seal_id: bound.event.seal_id.clone(),
            photo_sha256: bound.photo_sig.photo_sha256.clone(),
            verified: true,
            failure_reason: None,
        }
    }

    pub fn failed(event_id: &str, reason: &str) -> Self {
        Self {
            event_id: event_id.to_owned(),
            seal_id: String::new(),
            photo_sha256: String::new(),
            verified: false,
            failure_reason: Some(reason.to_owned()),
        }
    }
}
