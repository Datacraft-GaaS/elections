//! IssuanceHook: intercepts SPIRE + step-ca kit cert issuances and appends to
//! the KitCertTransparencyLog before the cert is delivered to the requester.
//!
//! The hook is synchronous with cert delivery: if the log append fails, the
//! cert is NOT returned. This prevents certs from existing outside the log.

use thiserror::Error;
use tracing::info;

use crate::{
    entry::KitCertEntry,
    log::{KitCertTransparencyLog, LogError},
};

#[derive(Debug, Error)]
pub enum HookError {
    #[error("log append failed: {0}")]
    LogAppend(#[from] LogError),
    #[error("cert DER is empty")]
    EmptyCert,
    #[error("kit_id is empty")]
    EmptyKitId,
    #[error("spiffe_id is empty")]
    EmptySpiffeId,
}

/// Request to issue and log a kit certificate.
#[derive(Debug, Clone)]
pub struct IssuanceRequest {
    pub kit_id: String,
    pub cert_der: Vec<u8>,
    pub issuer_ca: String,
    pub spiffe_id: String,
    pub revocation_ref: Option<String>,
}

/// Response from a successful issuance hook call.
#[derive(Debug)]
pub struct IssuanceReceipt {
    /// The leaf index assigned to this entry in the log.
    pub leaf_index: usize,
    /// The cert, unchanged, now guaranteed to be in the log.
    pub cert_der: Vec<u8>,
}

/// Intercept a cert issuance, append to the log, and return the receipt.
///
/// The cert is only returned if the log append succeeds. Call this
/// synchronously inside the CA issuance path before returning the cert to
/// the requester.
pub fn intercept(
    log: &KitCertTransparencyLog,
    req: IssuanceRequest,
) -> Result<IssuanceReceipt, HookError> {
    if req.kit_id.is_empty() {
        return Err(HookError::EmptyKitId);
    }
    if req.cert_der.is_empty() {
        return Err(HookError::EmptyCert);
    }
    if req.spiffe_id.is_empty() {
        return Err(HookError::EmptySpiffeId);
    }

    let entry = KitCertEntry {
        kit_id: req.kit_id.clone(),
        cert_der: req.cert_der.clone(),
        issued_at: chrono::Utc::now(),
        issuer_ca: req.issuer_ca.clone(),
        spiffe_id: req.spiffe_id.clone(),
        revocation_ref: req.revocation_ref.clone(),
    };

    let leaf_index = log.append(entry);

    info!(
        kit_id = %req.kit_id,
        spiffe_id = %req.spiffe_id,
        issuer_ca = %req.issuer_ca,
        leaf_index,
        "kit cert appended to transparency log"
    );

    Ok(IssuanceReceipt {
        leaf_index,
        cert_der: req.cert_der,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_req() -> IssuanceRequest {
        IssuanceRequest {
            kit_id: "kit-001".to_string(),
            cert_der: vec![0x30, 0x82],
            issuer_ca: "eel-step-ca".to_string(),
            spiffe_id: "spiffe://eel.internal/kit/001".to_string(),
            revocation_ref: None,
        }
    }

    #[test]
    fn intercept_appends_and_returns_cert() {
        let log = KitCertTransparencyLog::new();
        let req = sample_req();
        let cert_copy = req.cert_der.clone();
        let receipt = intercept(&log, req).unwrap();
        assert_eq!(receipt.leaf_index, 0);
        assert_eq!(receipt.cert_der, cert_copy);
        assert_eq!(log.tree_size(), 1);
    }

    #[test]
    fn intercept_rejects_empty_kit_id() {
        let log = KitCertTransparencyLog::new();
        let mut req = sample_req();
        req.kit_id = String::new();
        assert!(matches!(intercept(&log, req), Err(HookError::EmptyKitId)));
    }

    #[test]
    fn intercept_rejects_empty_cert() {
        let log = KitCertTransparencyLog::new();
        let mut req = sample_req();
        req.cert_der = vec![];
        assert!(matches!(intercept(&log, req), Err(HookError::EmptyCert)));
    }

    #[test]
    fn intercept_rejects_empty_spiffe_id() {
        let log = KitCertTransparencyLog::new();
        let mut req = sample_req();
        req.spiffe_id = String::new();
        assert!(matches!(
            intercept(&log, req),
            Err(HookError::EmptySpiffeId)
        ));
    }

    #[test]
    fn second_issuance_gets_index_1() {
        let log = KitCertTransparencyLog::new();
        intercept(&log, sample_req()).unwrap();
        let req2 = IssuanceRequest {
            kit_id: "kit-002".to_string(),
            cert_der: vec![0x30, 0x83],
            issuer_ca: "eel-step-ca".to_string(),
            spiffe_id: "spiffe://eel.internal/kit/002".to_string(),
            revocation_ref: None,
        };
        let receipt = intercept(&log, req2).unwrap();
        assert_eq!(receipt.leaf_index, 1);
    }
}
