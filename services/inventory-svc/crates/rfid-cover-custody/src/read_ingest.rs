//! Read-ingest pipeline: verify SVID, HMAC, record the read, raise alerts on tamper.

use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::domain::{CoverRead, ReaderType, Severity};
use crate::error::RfidError;
use crate::metrics;
use crate::reader::{HmacChallenger, ReaderRegistry};
use crate::replay_defence::NonceStore;
use crate::tamper_alert::{build_alert, classify_severity, RaiseAlertRequest};

// ---------------------------------------------------------------------------
// Request / response
// ---------------------------------------------------------------------------

/// Inbound read from a reader gate.
#[derive(Debug, Clone)]
pub struct IngestRequest {
    pub tag_id: Uuid,
    pub reader_id: String,
    pub reader_type: ReaderType,
    /// SPIFFE SVID presented by the reader in the request header.
    pub reader_svid: String,
    pub geo_lat: Option<f64>,
    pub geo_lon: Option<f64>,
    pub geo_accuracy_m: Option<f64>,
    /// 16-byte nonce the reader sent to the tag.
    pub tag_challenge: Option<Vec<u8>>,
    /// HMAC-SHA256 response returned by the tag.
    pub tag_response: Option<Vec<u8>>,
    /// Per-tag secret (in production fetched from HSM by `tag_secret_hsm_key_id`).
    pub tag_secret: Vec<u8>,
    pub tamper_indicator: bool,
    pub reader_signature: Option<Vec<u8>>,
    pub custody_event_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
}

/// Result of a successful ingest.
#[derive(Debug)]
pub struct IngestResult {
    pub read: CoverRead,
    /// Alert raised if tamper was detected; None on a clean read.
    pub alert: Option<crate::domain::CoverTamperAlert>,
}

// ---------------------------------------------------------------------------
// Ingest pipeline (pure logic; no DB; see `ingest_read_db` for the DB version)
// ---------------------------------------------------------------------------

/// Core ingest logic: verify SVID, verify HMAC, check replay, build the CoverRead.
///
/// DB writes and outbox emission are done by the caller (API handler / service layer)
/// to allow the whole operation to run inside a single transaction.
pub fn ingest_read_pure(
    req: &IngestRequest,
    registry: &ReaderRegistry,
    nonce_store: &mut dyn NonceStore,
) -> Result<IngestResult, RfidError> {
    // 1. Verify reader SVID.
    registry.verify_svid(&req.reader_id, &req.reader_svid)?;

    // 2. Replay defence.
    if let Some(nonce) = &req.tag_challenge {
        nonce_store.check_and_record(&req.tag_id.to_string(), nonce, Utc::now())?;
    }

    // 3. HMAC challenge/response verification.
    let challenge_valid = match (&req.tag_challenge, &req.tag_response) {
        (Some(nonce), Some(response)) => {
            let valid = HmacChallenger::verify_response(&req.tag_secret, nonce, response)?;
            if !valid {
                warn!(
                    tag_id = %req.tag_id,
                    reader_id = %req.reader_id,
                    "HMAC challenge verification failed"
                );
                metrics::inc_hmac_failures(req.reader_type.as_str());
            }
            valid
        }
        _ => {
            // No challenge/response provided; flag as unverified.
            warn!(tag_id = %req.tag_id, "read submitted without HMAC challenge; marking unverified");
            false
        }
    };

    // 4. Build the CoverRead.
    let read_id = Uuid::now_v7();
    let read = CoverRead {
        read_id,
        tag_id: req.tag_id,
        reader_id: req.reader_id.clone(),
        reader_type: req.reader_type.clone(),
        read_at: Utc::now(),
        geo_lat: req.geo_lat,
        geo_lon: req.geo_lon,
        geo_accuracy_m: req.geo_accuracy_m,
        tag_challenge: req.tag_challenge.clone(),
        tag_response: req.tag_response.clone(),
        challenge_valid,
        tamper_indicator: req.tamper_indicator,
        reader_signature: req.reader_signature.clone(),
        custody_event_id: req.custody_event_id,
        correlation_id: req.correlation_id,
    };

    metrics::inc_cover_reads(req.reader_type.as_str());

    info!(
        read_id = %read_id,
        tag_id = %req.tag_id,
        challenge_valid,
        tamper = req.tamper_indicator,
        "cover read ingested"
    );

    // 5. Raise alert on tamper or HMAC failure.
    let alert = if req.tamper_indicator || !challenge_valid {
        let severity = classify_severity(req.tamper_indicator, challenge_valid, false);
        metrics::inc_tamper_alerts(severity.as_str());

        let alert = build_alert(RaiseAlertRequest {
            tag_id: req.tag_id,
            triggering_read: Some(read_id),
            expected_state: None,
            observed_state: if req.tamper_indicator {
                Some("tamper_indicator_set".to_owned())
            } else {
                Some("hmac_failure".to_owned())
            },
            severity,
        });

        Some(alert)
    } else {
        None
    };

    Ok(IngestResult { read, alert })
}
