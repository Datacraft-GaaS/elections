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

//! N4: BLAKE3 hash-chained admin-action ledger.
//!
//! Every mutating admin action is recorded as a [`LedgerEntry`] whose
//! `this_hash = BLAKE3(prev_hash_bytes || canonical_entry_json_without_this_hash)`,
//! mirroring the rfid-cover-custody hash chain. The chain is designed for
//! low/intermittent connectivity:
//!
//! * **Append-first**: recording is a local, in-memory + JSONL append with
//!   `fsync` ([`FileStore::append_and_flush`]) — zero network required.
//! * **Verifiable offline**: [`AuditChain::verify`] and [`FileStore::load`]
//!   replay the file locally; no anchoring or sync is needed to detect tamper.
//! * Anchoring/sync (if any) is opportunistic and never blocks recording.
//!
//! # Axum integration
//!
//! Use the provided middleware via `from_fn_with_state`:
//!
//! ```ignore
//! use eel_audit_chain::{AuditChain, ChainState, record_middleware};
//! use std::sync::{Arc, Mutex};
//!
//! let state = ChainState {
//!     chain: Arc::new(Mutex::new(AuditChain::new())),
//!     file: Some(Arc::new(eel_audit_chain::FileStore::new("/var/lib/eel/audit.jsonl"))),
//! };
//! let app = axum::Router::new()
//!     .route("/admin/action", axum::routing::post(handler))
//!     .layer(axum::middleware::from_fn_with_state(state.clone(), record_middleware))
//!     .with_state(state);
//! ```
//!
//! POST/PUT/PATCH/DELETE requests append one entry (actor from the
//! `x-client-spiffe-id` header, operation `"{method} {path}"`, params hash =
//! BLAKE3 of the raw query string — the body is never read). File-flush errors
//! are logged at warn level and never fail the request.
#![warn(missing_docs)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::{Request, State};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use serde::{Deserialize, Serialize};

/// One recorded admin action in the hash chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LedgerEntry {
    pub seq: u64,
    pub actor_spiffe: String,
    pub operation: String,
    pub params_hash_hex: String,
    pub prev_hash_hex: String,
    pub this_hash_hex: String,
    pub recorded_at_rfc3339: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("chain broken at seq {seq}: expected {expected}, actual {actual}")]
    Broken {
        seq: u64,
        expected: String,
        actual: String,
    },
    #[error("malformed ledger: {0}")]
    Malformed(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Canonical serialization of an entry excluding `this_hash_hex`.
#[derive(Serialize)]
struct CanonicalEntry<'a> {
    seq: u64,
    actor_spiffe: &'a str,
    operation: &'a str,
    params_hash_hex: &'a str,
    prev_hash_hex: &'a str,
    recorded_at_rfc3339: &'a str,
}

fn canonical_bytes(entry: &LedgerEntry) -> Vec<u8> {
    let c = CanonicalEntry {
        seq: entry.seq,
        actor_spiffe: &entry.actor_spiffe,
        operation: &entry.operation,
        params_hash_hex: &entry.params_hash_hex,
        prev_hash_hex: &entry.prev_hash_hex,
        recorded_at_rfc3339: &entry.recorded_at_rfc3339,
    };
    serde_json::to_vec(&c).unwrap_or_else(|_| b"{}".to_vec())
}

#[cfg(test)]
mod hash_compat_tests {
    use super::*;

    // Keep canonical_bytes exercised: it is the reference serialization used
    // by verify() via CanonicalEntry; assert it matches what append() hashed.
    #[test]
    fn canonical_bytes_match_appended_hashes() {
        let c = sample_entries(2);
        for e in c.entries() {
            let canonical = canonical_bytes(e);
            let mut hasher = blake3::Hasher::new();
            if !e.prev_hash_hex.is_empty() {
                hasher.update(&hex::decode(&e.prev_hash_hex).unwrap());
            }
            hasher.update(&canonical);
            assert_eq!(hex::encode(hasher.finalize().as_bytes()), e.this_hash_hex);
        }
    }

    fn sample_entries(n: u64) -> AuditChain {
        let mut c = AuditChain::new();
        for i in 0..n {
            c.append("spiffe://x", &format!("op-{i}"), &serde_json::json!(i));
        }
        c
    }
}

fn compute_this_hash(prev_hash_hex: &str, entry_without_hash: &CanonicalEntry<'_>) -> [u8; 32] {
    let canonical = serde_json::to_vec(entry_without_hash).unwrap_or_else(|_| b"{}".to_vec());
    compute_this_hash_from_canonical(prev_hash_hex, &canonical)
}

fn compute_this_hash_from_canonical(prev_hash_hex: &str, canonical_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    if !prev_hash_hex.is_empty() {
        // Bad hex cannot occur on entries we produced; verify() rejects it as
        // Malformed before hashing. Here we fall back to raw bytes of the hex
        // string only if decoding fails (defensive).
        match hex::decode(prev_hash_hex) {
            Ok(b) => hasher.update(&b),
            Err(_) => hasher.update(prev_hash_hex.as_bytes()),
        };
    }
    hasher.update(canonical_bytes);
    *hasher.finalize().as_bytes()
}

pub fn params_hash_hex(params_json: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(params_json).unwrap_or_default();
    hex::encode(blake3::hash(&bytes).as_bytes())
}

/// In-memory hash chain. Genesis entry links from the empty prev hash `""`.
pub struct AuditChain {
    entries: Vec<LedgerEntry>,
}

impl Default for AuditChain {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditChain {
    /// New empty chain; the first appended entry links from genesis (`""`).
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(
        &mut self,
        actor_spiffe: &str,
        operation: &str,
        params_json: &serde_json::Value,
    ) -> &LedgerEntry {
        self.append_with_params_hash(actor_spiffe, operation, params_hash_hex(params_json))
    }

    /// Append with a caller-supplied params hash (middleware uses BLAKE3 of the
    /// raw query string here).
    pub fn append_with_params_hash(
        &mut self,
        actor_spiffe: &str,
        operation: &str,
        params_hash_hex: String,
    ) -> &LedgerEntry {
        let seq = self.entries.len() as u64;
        let prev_hash_hex = match self.entries.last() {
            Some(e) => e.this_hash_hex.clone(),
            None => String::new(),
        };
        // recorded_at participates in the hash; compute it before hashing.
        let recorded_at_rfc3339 = chrono::Utc::now().to_rfc3339();
        let canonical = CanonicalEntry {
            seq,
            actor_spiffe,
            operation,
            params_hash_hex: &params_hash_hex,
            prev_hash_hex: &prev_hash_hex,
            recorded_at_rfc3339: &recorded_at_rfc3339,
        };
        let this_hash_hex = hex::encode(compute_this_hash(&prev_hash_hex, &canonical));
        let entry = LedgerEntry {
            seq,
            actor_spiffe: actor_spiffe.to_string(),
            operation: operation.to_string(),
            params_hash_hex,
            prev_hash_hex,
            this_hash_hex,
            recorded_at_rfc3339,
        };
        self.entries.push(entry);
        self.entries.last().expect("just pushed")
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }
}

/// Verify chain integrity by full local replay from genesis (`""`).
///
/// Checks sequence continuity, hash-link continuity between consecutive
/// entries, and recomputes each `this_hash`.
pub fn verify(entries: &[LedgerEntry]) -> Result<(), ChainError> {
    let mut prev_this: Option<&str> = None;
    for (i, e) in entries.iter().enumerate() {
        if e.seq != i as u64 {
            return Err(ChainError::Malformed(format!(
                "expected seq {}, found {}",
                i, e.seq
            )));
        }
        if let Some(p) = prev_this {
            if e.prev_hash_hex != p {
                return Err(ChainError::Broken {
                    seq: e.seq,
                    expected: p.to_string(),
                    actual: e.prev_hash_hex.clone(),
                });
            }
        } else if !e.prev_hash_hex.is_empty() {
            return Err(ChainError::Broken {
                seq: e.seq,
                expected: String::new(),
                actual: e.prev_hash_hex.clone(),
            });
        }

        let expected = hex::encode(compute_this_hash_from_canonical(
            &e.prev_hash_hex,
            &canonical_bytes(e),
        ));
        if expected != e.this_hash_hex {
            return Err(ChainError::Broken {
                seq: e.seq,
                expected,
                actual: e.this_hash_hex.clone(),
            });
        }
        prev_this = Some(&e.this_hash_hex);
    }
    Ok(())
}

/// Durable offline store: one JSON line per entry, append + fsync.
///
/// Recording works with zero network; verification is a pure local replay of
/// the file.
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Append ONE JSON line and durably flush (flush + sync_all).
    pub fn append_and_flush(&self, entry: &LedgerEntry) -> std::io::Result<()> {
        let mut line = serde_json::to_string(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        line.push('\n');
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        f.write_all(line.as_bytes())?;
        f.flush()?;
        f.sync_all()
    }

    /// Load and fully verify a ledger file. Any corrupt line fails with the
    /// offending line number; any broken link fails with its seq.
    pub fn load(path: impl Into<PathBuf>) -> Result<AuditChain, ChainError> {
        let path = path.into();
        let data = std::fs::read_to_string(&path)?;
        let mut entries = Vec::new();
        for (n, line) in data.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let entry: LedgerEntry = serde_json::from_str(line).map_err(|e| {
                ChainError::Malformed(format!(
                    "{}:{}: invalid ledger line: {}",
                    path.display(),
                    n + 1,
                    e
                ))
            })?;
            entries.push(entry);
        }
        verify(&entries)?;
        Ok(AuditChain { entries })
    }
}

/// Cloneable middleware state holding the in-memory chain and optional durable
/// store.
#[derive(Clone)]
pub struct ChainState {
    pub chain: Arc<Mutex<AuditChain>>,
    pub file: Option<Arc<FileStore>>,
}

const MUTATING_METHODS: [Method; 4] = [Method::POST, Method::PUT, Method::PATCH, Method::DELETE];

/// Axum middleware recording every mutating request into the audit chain.
///
/// Actor is taken from the `x-client-spiffe-id` header (fallback `"unknown"`),
/// operation is `"{method} {path}"`, and the params hash is BLAKE3 of the raw
/// query string only — the request body is never read. File-flush errors are
/// best-effort: they are logged at warn and never fail the request. See the
/// crate doc-comment for registration via `from_fn_with_state`.
pub async fn record_middleware(
    State(state): State<ChainState>,
    req: Request,
    next: Next,
) -> Response {
    let mutating = MUTATING_METHODS.contains(req.method());
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();
    let actor = req
        .headers()
        .get("x-client-spiffe-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let resp = next.run(req).await;

    if mutating {
        let params_hash = hex::encode(blake3::hash(query.as_bytes()).as_bytes());
        let operation = format!("{} {}", method, path);
        let entry = {
            let mut chain = state.chain.lock().expect("audit chain mutex poisoned");
            chain
                .append_with_params_hash(&actor, &operation, params_hash)
                .clone()
        };
        if let Some(store) = &state.file {
            if let Err(e) = store.append_and_flush(&entry) {
                tracing::warn!(error = %e, seq = entry.seq, "audit chain file flush failed");
            }
        }
    }

    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use axum::routing::post;
    use axum::Router;
    use tower::ServiceExt;

    fn sample_entries(n: u64) -> AuditChain {
        let mut c = AuditChain::new();
        for i in 0..n {
            c.append(
                "spiffe://eel/admin",
                &format!("op-{i}"),
                &serde_json::json!({ "i": i }),
            );
        }
        c
    }

    #[test]
    fn append_verify_round_trip() {
        let c = sample_entries(5);
        assert_eq!(c.entries().len(), 5);
        assert_eq!(c.entries()[0].seq, 0);
        assert!(c.entries()[0].prev_hash_hex.is_empty());
        assert_eq!(c.entries()[1].prev_hash_hex, c.entries()[0].this_hash_hex);
        verify(c.entries()).expect("fresh chain must verify");
    }

    #[test]
    fn tamper_middle_entry_is_detected_with_seq() {
        let mut entries = sample_entries(4).entries().to_vec();
        let middle = entries[2].clone();
        entries[2].operation = "tampered-op".to_string();
        match verify(&entries) {
            Err(ChainError::Broken {
                seq,
                expected,
                actual,
            }) => {
                assert_eq!(seq, 2);
                assert_eq!(actual, middle.this_hash_hex);
                assert_ne!(expected, actual);
            }
            other => panic!("expected Broken, got {:?}", other.map(|_| ())),
        }
        // Untampered prefix still verifies.
        verify(&entries[..2]).expect("prefix should verify");
    }

    #[test]
    fn broken_link_is_detected() {
        let mut entries = sample_entries(3).entries().to_vec();
        entries[2].prev_hash_hex = hex::encode([0u8; 32]);
        match verify(&entries) {
            Err(ChainError::Broken { seq, .. }) => assert_eq!(seq, 2),
            other => panic!("expected Broken, got {:?}", other.map(|_| ())),
        }
    }

    #[test]
    fn malformed_seq_is_rejected() {
        let mut entries = sample_entries(3).entries().to_vec();
        entries[1].seq = 99;
        assert!(matches!(verify(&entries), Err(ChainError::Malformed(_))));
    }

    #[test]
    fn file_store_round_trip_reload_verify() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("audit.jsonl");
        let store = FileStore::new(&path);

        let chain = sample_entries(3);
        for e in chain.entries() {
            store.append_and_flush(e).expect("append_and_flush");
        }

        let reloaded = FileStore::load(&path).expect("reload");
        assert_eq!(reloaded.entries().len(), 3);
        assert_eq!(reloaded.entries(), chain.entries());
        verify(reloaded.entries()).expect("reloaded chain verifies offline");

        // Appending continues seamlessly after reload.
        let mut reloaded = reloaded;
        reloaded.append("spiffe://eel/admin", "post-reload", &serde_json::json!({}));
        assert_eq!(reloaded.entries().len(), 4);
        verify(reloaded.entries()).expect("extended chain verifies");
    }

    #[test]
    fn corrupt_line_load_fails_naming_line() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("corrupt.jsonl");
        let store = FileStore::new(&path);
        for e in sample_entries(2).entries() {
            store.append_and_flush(e).expect("append_and_flush");
        }
        // Corrupt the second line in place.
        let data = std::fs::read_to_string(&path).unwrap();
        let mut lines: Vec<String> = data.lines().map(str::to_string).collect();
        lines[1] = "{ not json".to_string();
        std::fs::write(&path, lines.join("\n") + "\n").unwrap();

        match FileStore::load(&path) {
            Err(ChainError::Malformed(msg)) => assert!(msg.contains("2:"), "{}", msg),
            other => panic!("expected Malformed, got {:?}", other.map(|_| ())),
        }
    }

    #[tokio::test]
    async fn middleware_records_post_but_not_get() {
        let state = ChainState {
            chain: Arc::new(Mutex::new(AuditChain::new())),
            file: None,
        };

        async fn ok_handler() -> StatusCode {
            StatusCode::OK
        }

        let app: Router<()> = Router::new()
            .route("/admin/action", post(ok_handler).get(ok_handler))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                record_middleware,
            ))
            .with_state(state.clone());

        let post_req = HttpRequest::builder()
            .method("POST")
            .uri("/admin/action?target=tag-42")
            .header("x-client-spiffe-id", "spiffe://eel/admin/alice")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(post_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let get_req = HttpRequest::builder()
            .uri("/admin/action")
            .body(Body::empty())
            .unwrap();
        let app2: Router<()> = Router::new()
            .route("/admin/action", post(ok_handler).get(ok_handler))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                record_middleware,
            ))
            .with_state(state.clone());
        let resp = app2.oneshot(get_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let chain = state.chain.lock().unwrap();
        assert_eq!(chain.entries().len(), 1, "only the POST must be recorded");
        let e = &chain.entries()[0];
        assert_eq!(e.actor_spiffe, "spiffe://eel/admin/alice");
        assert_eq!(e.operation, "POST /admin/action");
        assert_eq!(
            e.params_hash_hex,
            hex::encode(blake3::hash(b"target=tag-42").as_bytes())
        );
    }
}
