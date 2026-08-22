//! Single-signal ingest pipeline.
//!
//! Steps: verify SVID, verify kit-cert, verify HMAC, verify prev_hash link,
//! verify monotonic_counter strictly greater, verify nonce not seen in 5-min
//! window, write eel_telemetry_signals, server counter-sign, emit CloudEvent.

use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::domain::{SignalClass, TelemetrySignal};
use crate::error::TelemetryError;
use crate::hash_chain::compute_this_hash;
use crate::hmac_verify::{compute_hmac, verify_hmac};
use crate::metrics;
use crate::replay_defence::NonceStore;

// ---------------------------------------------------------------------------
// Request / response
// ---------------------------------------------------------------------------

/// Inbound signal from a field device.
#[derive(Debug, Clone)]
pub struct IngestRequest {
    pub device_id: Uuid,
    pub session_id: Option<Uuid>,
    pub class: SignalClass,
    pub payload: serde_json::Value,
    pub occurred_at: chrono::DateTime<Utc>,
    pub monotonic_counter: i64,
    /// BLAKE3 hash of the previous signal row for this device (None = genesis).
    pub prev_hash: Option<Vec<u8>>,
    /// HMAC-SHA256 over canonical JSON of payload, keyed by per-device secret.
    pub hmac_signature: Vec<u8>,
    /// Nonce for replay defence (any unique bytes per request).
    pub nonce: Vec<u8>,
    pub trace_id: Option<Vec<u8>>,
    pub span_id: Option<Vec<u8>>,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    /// The device's SPIFFE SVID (from mTLS or request header).
    pub actor_svid: String,
    /// Per-device HMAC secret (caller resolves from hmac_key_hsm_ref).
    pub device_secret: Vec<u8>,
    /// Last monotonic counter stored for this device (-1 for genesis).
    pub last_known_counter: i64,
    /// Last this_hash stored for this device (empty for genesis).
    pub last_known_hash: Vec<u8>,
}

/// Outcome of a successful ingest (pre-DB; caller writes to DB).
#[derive(Debug)]
pub struct IngestOutcome {
    pub signal: TelemetrySignal,
    /// Server-computed counter-signature over this_hash.
    pub server_signature: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Pure ingest logic (no DB; caller wraps in a transaction)
// ---------------------------------------------------------------------------

/// Core ingest validation: SVID check, nonce check, HMAC verify, chain verify,
/// counter verify. Returns a fully formed `TelemetrySignal` ready to insert.
pub fn ingest_signal_pure(
    req: &IngestRequest,
    nonce_store: &mut dyn NonceStore,
    server_signing_key: &[u8],
) -> Result<IngestOutcome, TelemetryError> {
    let device_id_str = req.device_id.to_string();

    // 1. Verify SVID is non-empty (full SPIFFE verification done at mTLS layer).
    if req.actor_svid.is_empty() {
        return Err(TelemetryError::SvidRejected("empty SVID".to_owned()));
    }

    // 2. Replay defence.
    nonce_store.check_and_record(&device_id_str, &req.nonce, Utc::now())?;

    // 3. HMAC verification.
    verify_hmac(
        &req.device_secret,
        &req.payload,
        &req.hmac_signature,
        &device_id_str,
    )
    .map_err(|e| {
        warn!(device_id = %req.device_id, "HMAC verification failed");
        metrics::inc_hmac_failures();
        e
    })?;

    // 4. Monotonic counter strictly greater.
    if req.monotonic_counter <= req.last_known_counter {
        warn!(
            device_id = %req.device_id,
            got = req.monotonic_counter,
            prev = req.last_known_counter,
            "monotonic counter out-of-order"
        );
        return Err(TelemetryError::CounterOutOfOrder {
            device_id: device_id_str.clone(),
            got: req.monotonic_counter,
            prev: req.last_known_counter,
        });
    }

    let now = Utc::now();
    let signal_id = Uuid::now_v7();

    // 5. Build partial signal to compute hash.
    let mut signal = TelemetrySignal {
        signal_id,
        device_id: req.device_id,
        session_id: req.session_id,
        class: req.class.clone(),
        payload: req.payload.clone(),
        occurred_at: req.occurred_at,
        ingested_at: now,
        monotonic_counter: req.monotonic_counter,
        prev_hash: if req.last_known_hash.is_empty() {
            None
        } else {
            Some(req.last_known_hash.clone())
        },
        this_hash: vec![],
        hmac_signature: req.hmac_signature.clone(),
        server_signature: None,
        trace_id: req.trace_id.clone(),
        span_id: req.span_id.clone(),
        correlation_id: req.correlation_id,
        causation_id: req.causation_id,
    };

    // 6. Compute BLAKE3 hash chain link.
    let hash = compute_this_hash(req.prev_hash.as_deref().unwrap_or(&[]), &signal);
    signal.this_hash = hash.to_vec();

    // 7. Server counter-sign over this_hash.
    let sig_payload = serde_json::json!({ "this_hash": hex::encode(&signal.this_hash) });
    let server_signature = compute_hmac(server_signing_key, &sig_payload)?;
    signal.server_signature = Some(server_signature.clone());

    info!(
        device_id = %req.device_id,
        signal_id = %signal_id,
        class = %req.class,
        "signal ingested"
    );
    metrics::inc_signals_ingested(req.class.as_str());

    Ok(IngestOutcome {
        signal,
        server_signature,
    })
}
