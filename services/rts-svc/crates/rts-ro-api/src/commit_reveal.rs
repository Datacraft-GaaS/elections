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

//! N1: commit-reveal for Form 34A tallies.
//!
//! Officers COMMIT a hash of the form (signed) before revealing contents, so
//! the tally cannot be adapted after totals are seen. Designed for
//! LOW/INTERMITTENT CONNECTIVITY: commits and full-form arrivals are both
//! accepted AT ANY TIME and in ANY ORDER — an arrival may beat its commitment
//! (still in SMS flight) or a duplicate commit may be re-posted after a
//! relay retry. The contract is:
//!
//! - commit is IDEMPOTENT for identical fields (200 on re-post),
//! - a DIFFERENT hash for the same (station, kind) once recorded is a 409,
//! - arrival before commit is accepted into the tally pipeline but flagged
//!   `uncommitted` (the commitment may simply not have landed yet),
//! - arrival mismatching the commitment is `quarantined` (409 with both hashes).
//!
//! All state here is an in-process Mutex. Production backs this with the
//! outbox table; what matters for this module is the late/out-of-order/
//! duplicated-acceptance CONTRACT above.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    sync::Mutex,
};
use tracing::{info, warn};

/// Canonical bytes covered by an officer's commit signature:
/// `{station}|{kind}|{hash}|{officer}` (ASCII, pipe-separated).
pub fn canonical_commit_bytes(
    station_code: &str,
    form_kind: &str,
    form_hash_hex: &str,
    officer_id: &str,
) -> Vec<u8> {
    format!("{station_code}|{form_kind}|{form_hash_hex}|{officer_id}").into_bytes()
}

/// A recorded commitment for one (station, form-kind) pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub form_hash_hex: String,
    pub officer_signature_hex: String,
    /// RFC3339 timestamp of when the commit was ACCEPTED here.
    pub committed_at_rfc3339: String,
    /// Monotonic local arrival counter; ordering of record-keeping only.
    pub received_seq: u64,
}

/// In-process commit store keyed by (station_code, form_kind).
///
/// Production backs this with the outbox table so late/duplicated SMS-relayed
/// commits merge deterministically across restarts.
#[derive(Default)]
pub struct CommitStore {
    inner: Mutex<HashMap<(String, String), Entry>>,
}

static STORE: std::sync::LazyLock<CommitStore> = std::sync::LazyLock::new(|| CommitStore {
    inner: Mutex::new(HashMap::new()),
});

static RECEIVED_SEQ: AtomicU64 = AtomicU64::new(1);

impl CommitStore {
    pub fn get(&self, station_code: &str, form_kind: &str) -> Option<Entry> {
        self.inner
            .lock()
            .expect("commit store poisoned")
            .get(&(station_code.to_string(), form_kind.to_string()))
            .cloned()
    }

    pub fn insert(&self, station_code: &str, form_kind: &str, entry: Entry) {
        self.inner
            .lock()
            .expect("commit store poisoned")
            .insert((station_code.to_string(), form_kind.to_string()), entry);
    }

    pub fn snapshot(&self) -> HashMap<(String, String), Entry> {
        self.inner.lock().expect("commit store poisoned").clone()
    }
}

fn next_received_seq() -> u64 {
    RECEIVED_SEQ.fetch_add(1, Ordering::Relaxed)
}

// --- Officer key registry ----------------------------------------------------

/// Parse `EEL_RO_OFFICER_KEYS`: `"station_code:hex_pubkey"` comma-separated.
/// Mirrors the agents.rs registry pattern but scoped per polling station.
pub fn parse_officer_registry(spec: &str) -> Result<HashMap<String, VerifyingKey>, String> {
    let mut out = HashMap::new();
    for entry in spec.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let parts: Vec<&str> = entry.split(':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "bad EEL_RO_OFFICER_KEYS entry {entry:?} (want station:hexkey)"
            ));
        }
        let key_bytes =
            hex::decode(parts[1]).map_err(|e| format!("bad key hex in {entry:?}: {e}"))?;
        let key_arr: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| format!("key must be 32 bytes in {entry:?}"))?;
        let verifying = VerifyingKey::from_bytes(&key_arr)
            .map_err(|e| format!("bad ed25519 key in {entry:?}: {e}"))?;
        out.insert(parts[0].to_string(), verifying);
    }
    Ok(out)
}

pub fn officer_registry_from_env() -> HashMap<String, VerifyingKey> {
    let spec = std::env::var("EEL_RO_OFFICER_KEYS").unwrap_or_default();
    match parse_officer_registry(&spec) {
        Ok(map) => {
            if map.is_empty() {
                tracing::warn!("EEL_RO_OFFICER_KEYS empty; all form commits will be rejected");
            }
            map
        }
        Err(e) => {
            tracing::error!(error = %e, "EEL_RO_OFFICER_KEYS malformed; NO commits will be accepted");
            HashMap::new()
        }
    }
}

enum CommitAuthError {
    UnknownOfficer,
    BadSignatureFormat,
    InvalidSignature,
}

fn verify_officer_signature(
    registry: &HashMap<String, VerifyingKey>,
    station_code: &str,
    form_kind: &str,
    form_hash_hex: &str,
    officer_id: &str,
    signature_hex: &str,
) -> Result<(), CommitAuthError> {
    // The registered key is looked up by STATION; officer_id is bound into the
    // signed message so it cannot be swapped after signing.
    let key = registry
        .get(station_code)
        .ok_or(CommitAuthError::UnknownOfficer)?;
    let sig_bytes =
        hex::decode(signature_hex.trim()).map_err(|_| CommitAuthError::BadSignatureFormat)?;
    let sig = Signature::from_slice(&sig_bytes).map_err(|_| CommitAuthError::BadSignatureFormat)?;
    key.verify(
        &canonical_commit_bytes(station_code, form_kind, form_hash_hex, officer_id),
        &sig,
    )
    .map_err(|_| CommitAuthError::InvalidSignature)
}

fn valid_form_hash(h: &str) -> bool {
    h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())
}

// --- Requests / responses ----------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    pub station_code: String,
    pub form_kind: String,
    /// SHA-256 of the sealed 34A payload (64 lowercase/uppercase hex).
    pub form_hash_hex: String,
    pub officer_id: String,
    /// Ed25519 over canonical_commit_bytes, hex-encoded.
    pub officer_signature_hex: String,
    /// True when this arrived via SMS relay (offline path). Accepted any time;
    /// informational only — late relays are the expected case.
    #[serde(default)]
    pub sms_relayed: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArriveResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committed_hash_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrived_hash_hex: Option<String>,
}

// --- Handlers ----------------------------------------------------------------

// POST /api/v1/rts/forms/commit
pub async fn forms_commit(Json(req): Json<CommitRequest>) -> Response {
    if !valid_form_hash(&req.form_hash_hex) {
        return (
            StatusCode::BAD_REQUEST,
            "form_hash_hex must be 64 hex chars",
        )
            .into_response();
    }

    static REGISTRY: std::sync::OnceLock<HashMap<String, VerifyingKey>> =
        std::sync::OnceLock::new();
    let registry = REGISTRY.get_or_init(officer_registry_from_env);

    match verify_officer_signature(
        registry,
        &req.station_code,
        &req.form_kind,
        &req.form_hash_hex,
        &req.officer_id,
        &req.officer_signature_hex,
    ) {
        Ok(()) => {}
        Err(CommitAuthError::UnknownOfficer) => {
            warn!(station_code = %req.station_code, "form commit rejected: unregistered station/officer key");
            return (StatusCode::FORBIDDEN, "unregistered officer key").into_response();
        }
        Err(CommitAuthError::BadSignatureFormat) => {
            return (StatusCode::BAD_REQUEST, "malformed signature encoding").into_response();
        }
        Err(CommitAuthError::InvalidSignature) => {
            warn!(station_code = %req.station_code, officer_id = %req.officer_id, "form commit rejected: invalid signature");
            return (StatusCode::FORBIDDEN, "signature does not verify").into_response();
        }
    }

    match STORE.get(&req.station_code, &req.form_kind) {
        Some(existing) => {
            if existing
                .form_hash_hex
                .eq_ignore_ascii_case(&req.form_hash_hex)
            {
                // IDEMPOTENT: duplicate/retried commit (SMS relays will do
                // this constantly under intermittent connectivity).
                info!(
                    station_code = %req.station_code,
                    form_kind = %req.form_kind,
                    received_seq = existing.received_seq,
                    "duplicate form commit (idempotent)"
                );
                (
                    StatusCode::OK,
                    Json(json!({
                        "status": "duplicate",
                        "received_seq": existing.received_seq,
                        "committed_at_rfc3339": existing.committed_at_rfc3339,
                    })),
                )
                    .into_response()
            } else {
                warn!(
                    station_code = %req.station_code,
                    form_kind = %req.form_kind,
                    "conflicting form commit hash"
                );
                (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "status": "conflict",
                        "committed_hash_hex": existing.form_hash_hex,
                        "arrived_hash_hex": req.form_hash_hex,
                    })),
                )
                    .into_response()
            }
        }
        None => {
            let entry = Entry {
                form_hash_hex: req.form_hash_hex.to_lowercase(),
                officer_signature_hex: req.officer_signature_hex.clone(),
                committed_at_rfc3339: chrono::Utc::now().to_rfc3339(),
                received_seq: next_received_seq(),
            };
            let received_seq = entry.received_seq;
            STORE.insert(&req.station_code, &req.form_kind, entry);
            info!(
                station_code = %req.station_code,
                form_kind = %req.form_kind,
                received_seq,
                "form commit recorded"
            );
            (
                StatusCode::CREATED,
                Json(json!({ "status": "committed", "received_seq": received_seq })),
            )
                .into_response()
        }
    }
}

// POST /api/v1/rts/forms/arrive
pub async fn forms_arrive(Json(req): Json<ArriveRequest>) -> Response {
    if !valid_form_hash(&req.form_hash_hex) {
        return (
            StatusCode::BAD_REQUEST,
            "form_hash_hex must be 64 hex chars",
        )
            .into_response();
    }

    match STORE.get(&req.station_code, &req.form_kind) {
        Some(entry) => {
            if entry.form_hash_hex.eq_ignore_ascii_case(&req.form_hash_hex) {
                Json(ArriveResponse {
                    status: "matched".into(),
                    committed_hash_hex: None,
                    arrived_hash_hex: None,
                })
                .into_response()
            } else {
                warn!(
                    station_code = %req.station_code,
                    form_kind = %req.form_kind,
                    "arrival quarantined: hash differs from commitment"
                );
                // Quarantine carries BOTH hashes so offline reconciliation
                // merges can adjudicate without re-fetching anything.
                (
                    StatusCode::CONFLICT,
                    Json(ArriveResponse {
                        status: "quarantined".into(),
                        committed_hash_hex: Some(entry.form_hash_hex),
                        arrived_hash_hex: Some(req.form_hash_hex),
                    }),
                )
                    .into_response()
            }
        }
        None => {
            // Low-connectivity reality: the full form may arrive before its
            // commitment clears the SMS flight. Accept for the tally
            // pipeline but flag as uncommitted.
            info!(
                station_code = %req.station_code,
                form_kind = %req.form_kind,
                "arrival without commitment; accepting flagged"
            );
            Json(ArriveResponse {
                status: "uncommitted".into(),
                committed_hash_hex: None,
                arrived_hash_hex: None,
            })
            .into_response()
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArriveRequest {
    pub station_code: String,
    pub form_kind: String,
    pub form_hash_hex: String,
}

// GET /api/v1/rts/forms/commitments
pub async fn list_commitments() -> impl IntoResponse {
    let snapshot = STORE.snapshot();
    let items: Vec<serde_json::Value> = snapshot
        .into_iter()
        .map(|((station, kind), e)| {
            json!({
                "station_code": station,
                "form_kind": kind,
                "form_hash_hex": e.form_hash_hex,
                "officer_signature_hex": e.officer_signature_hex,
                "committed_at_rfc3339": e.committed_at_rfc3339,
                "received_seq": e.received_seq,
            })
        })
        .collect();
    Json(json!({ "items": items, "total": items.len() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::post, Router};
    use ed25519_dalek::Signer;
    use tower::ServiceExt;

    fn test_app() -> Router {
        Router::new()
            .route("/api/v1/rts/forms/commit", post(forms_commit))
            .route("/api/v1/rts/forms/arrive", post(forms_arrive))
    }

    fn signed_commit(
        sk: &ed25519_dalek::SigningKey,
        station: &str,
        kind: &str,
        hash: &str,
        officer: &str,
    ) -> CommitRequest {
        let msg = canonical_commit_bytes(station, kind, hash, officer);
        CommitRequest {
            station_code: station.into(),
            form_kind: kind.into(),
            form_hash_hex: hash.into(),
            officer_id: officer.into(),
            officer_signature_hex: hex::encode(sk.sign(&msg).to_bytes()),
            sms_relayed: Some(true),
        }
    }

    async fn post_json(app: Router, uri: &str, body: serde_json::Value) -> Response {
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
    }

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn canonical_bytes_shape() {
        assert_eq!(
            canonical_commit_bytes("ST-01", "34A", HASH_A, "ro-7"),
            b"ST-01|34A|aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa|ro-7"
        );
    }

    #[tokio::test]
    async fn full_commit_reveal_flow_late_duplicated_conflicting() {
        // One serialized flow: the store + env-backed registry are process
        // globals, so scenarios run sequentially against distinct stations.
        let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let wrong_sk = ed25519_dalek::SigningKey::from_bytes(&[99u8; 32]);
        let pk = hex::encode(sk.verifying_key().as_bytes());
        std::env::set_var(
            "EEL_RO_OFFICER_KEYS",
            format!("ST-IDEM:{pk},ST-LATE:{pk},ST-QUAR:{pk},ST-BADSIG:{pk}"),
        );
        let app = test_app();

        async fn body_json(resp: Response) -> serde_json::Value {
            serde_json::from_slice(
                &axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap(),
            )
            .unwrap()
        }

        // -- idempotency -----------------------------------------------------
        let commit = signed_commit(&sk, "ST-IDEM", "34A", HASH_A, "ro-1");
        let resp = post_json(
            app.clone(),
            "/api/v1/rts/forms/commit",
            serde_json::to_value(&commit).unwrap(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Identical re-post (late/duplicated SMS relay) -> 200, idempotent.
        let resp = post_json(
            app.clone(),
            "/api/v1/rts/forms/commit",
            serde_json::to_value(&commit).unwrap(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(body_json(resp).await["status"], "duplicate");

        // Differing hash for same (station, kind) after arrival recorded -> 409.
        let conflicting = signed_commit(&sk, "ST-IDEM", "34A", HASH_B, "ro-1");
        let resp = post_json(
            app.clone(),
            "/api/v1/rts/forms/commit",
            serde_json::to_value(&conflicting).unwrap(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);

        // -- bad signature / unknown officer ---------------------------------
        let forged = signed_commit(&wrong_sk, "ST-BADSIG", "34A", HASH_A, "ro-1");
        let resp = post_json(
            app.clone(),
            "/api/v1/rts/forms/commit",
            serde_json::to_value(&forged).unwrap(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let unregistered = signed_commit(&sk, "ST-UNKNOWN", "34A", HASH_A, "ro-1");
        let resp = post_json(
            app.clone(),
            "/api/v1/rts/forms/commit",
            serde_json::to_value(&unregistered).unwrap(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let malformed_hash = signed_commit(&sk, "ST-IDEM", "34B", "nothex", "ro-1");
        let resp = post_json(
            app.clone(),
            "/api/v1/rts/forms/commit",
            serde_json::to_value(&malformed_hash).unwrap(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // -- late commitment arriving AFTER arrival ---------------------------
        let arrive = serde_json::json!({
            "station_code": "ST-LATE",
            "form_kind": "34A",
            "form_hash_hex": HASH_A
        });
        let resp = post_json(app.clone(), "/api/v1/rts/forms/arrive", arrive.clone()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(body_json(resp).await["status"], "uncommitted");

        let late_commit = signed_commit(&sk, "ST-LATE", "34A", HASH_A, "ro-2");
        let resp = post_json(
            app.clone(),
            "/api/v1/rts/forms/commit",
            serde_json::to_value(&late_commit).unwrap(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Re-check flow: arrival now matches the recorded commitment.
        let resp = post_json(app.clone(), "/api/v1/rts/forms/arrive", arrive).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(body_json(resp).await["status"], "matched");

        // -- quarantined mismatch carries both hashes -------------------------
        let commit = signed_commit(&sk, "ST-QUAR", "34B", HASH_A, "ro-3");
        let resp = post_json(
            app.clone(),
            "/api/v1/rts/forms/commit",
            serde_json::to_value(&commit).unwrap(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let arrive_mismatch = serde_json::json!({
            "station_code": "ST-QUAR",
            "form_kind": "34B",
            "form_hash_hex": HASH_B
        });
        let resp = post_json(app.clone(), "/api/v1/rts/forms/arrive", arrive_mismatch).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let body = body_json(resp).await;
        assert_eq!(body["status"], "quarantined");
        assert_eq!(body["committed_hash_hex"], HASH_A);
        assert_eq!(body["arrived_hash_hex"], HASH_B);

        // -- snapshot for offline reconciliation ------------------------------
        let entry = STORE.get("ST-LATE", "34A").expect("late commitment stored");
        assert_eq!(entry.form_hash_hex, HASH_A);
        assert!(entry.received_seq >= 1);
    }
}
