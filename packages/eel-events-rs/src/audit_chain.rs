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

//! HARD-21: cross-service audit-shadow prev_hash chain.
//!
//! Every service that mutates a resource appends an AuditEntry whose `this_hash`
//! is BLAKE3(prev_hash || canonical_json(payload) || entry_id).  A verifier
//! reconstructs the chain and rejects any gap or silent rewrite.
//!
//! ## Migration guide for existing per-service audit_shadow tables
//!
//! 1. Add two columns to your `audit_shadow` table (or equivalent):
//!    ```sql
//!    ALTER TABLE audit_shadow
//!      ADD COLUMN prev_hash BYTEA NOT NULL DEFAULT '\x'::BYTEA,
//!      ADD COLUMN this_hash BYTEA NOT NULL DEFAULT '\x'::BYTEA;
//!    ```
//! 2. For the very first row per resource, set `prev_hash = [0u8; 32]`.
//! 3. Compute `this_hash` application-side via `AuditEntry::compute_hash` and
//!    store it atomically with the resource mutation inside the same transaction.
//! 4. Verification: read all rows ordered by `appended_at ASC` and call
//!    `ChainVerifier::verify_all`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single immutable audit entry in the cross-service chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// UUID v7 — monotonically ordered by creation time.
    pub entry_id: String,
    /// Hash of the previous entry in this resource's chain (all zeros for first).
    pub prev_hash: [u8; 32],
    /// Service that performed the action (e.g. "eel.rts-svc").
    pub service: String,
    /// Action name (e.g. "form34a_submitted").
    pub action: String,
    /// SPIFFE ID or operator ID of the actor.
    pub actor: String,
    /// Canonical resource identifier (e.g. "station:47-001").
    pub resource: String,
    /// Timestamp of the action.
    pub at: DateTime<Utc>,
    /// Arbitrary structured payload (kept small; large blobs belong in object storage).
    pub payload: Value,
    /// BLAKE3(prev_hash || canonical_json(payload) || entry_id_utf8).
    pub this_hash: [u8; 32],
}

impl AuditEntry {
    /// Compute the hash that `this_hash` must equal.
    ///
    /// Canonical JSON: `serde_json::to_vec` with sorted keys is achieved by
    /// using a BTreeMap for the payload.  In practice, callers should normalise
    /// their payloads before passing them in, or use `canonicalize_payload`.
    pub fn compute_hash(prev_hash: &[u8; 32], payload: &Value, entry_id: &str) -> [u8; 32] {
        let canonical = canonicalize_payload(payload);
        let mut hasher = blake3::Hasher::new();
        hasher.update(prev_hash);
        hasher.update(&canonical);
        hasher.update(entry_id.as_bytes());
        *hasher.finalize().as_bytes()
    }

    /// Returns true iff `this_hash` matches the recomputed value.
    pub fn is_self_consistent(&self) -> bool {
        let expected = Self::compute_hash(&self.prev_hash, &self.payload, &self.entry_id);
        self.this_hash == expected
    }
}

/// Produce deterministic (sorted-key) JSON bytes for a serde_json Value.
pub fn canonicalize_payload(v: &Value) -> Vec<u8> {
    // Sort object keys recursively for determinism.
    fn sort_value(v: &Value) -> Value {
        match v {
            Value::Object(m) => {
                let sorted: serde_json::Map<String, Value> = m
                    .iter()
                    .map(|(k, v)| (k.clone(), sort_value(v)))
                    .collect::<std::collections::BTreeMap<_, _>>()
                    .into_iter()
                    .collect();
                Value::Object(sorted)
            }
            Value::Array(a) => Value::Array(a.iter().map(sort_value).collect()),
            other => other.clone(),
        }
    }
    serde_json::to_vec(&sort_value(v)).unwrap_or_default()
}

/// Trait for services to plug in their own storage back-end.
pub trait ChainedAudit {
    type Error: std::fmt::Debug;

    /// The hash of the most-recently stored entry for `resource`, or `[0u8; 32]`
    /// if no entries exist yet.  Must be read inside the same transaction as
    /// `append`.
    fn tip_hash(&self, resource: &str) -> Result<[u8; 32], Self::Error>;

    /// Persist the entry atomically with the resource mutation.
    fn persist(&mut self, entry: &AuditEntry) -> Result<(), Self::Error>;

    /// Build and persist a new AuditEntry, computing `this_hash` automatically.
    // The fields are the entry itself; a struct parameter would only add a
    // wrapper around the same audit-chain ordering.
    #[allow(clippy::too_many_arguments)]
    fn append(
        &mut self,
        entry_id: String,
        service: String,
        action: String,
        actor: String,
        resource: String,
        at: DateTime<Utc>,
        payload: Value,
    ) -> Result<AuditEntry, Self::Error> {
        let prev_hash = self.tip_hash(&resource)?;
        let this_hash = AuditEntry::compute_hash(&prev_hash, &payload, &entry_id);
        let entry = AuditEntry {
            entry_id,
            prev_hash,
            service,
            action,
            actor,
            resource,
            at,
            payload,
            this_hash,
        };
        self.persist(&entry)?;
        Ok(entry)
    }
}

/// In-memory implementation of `ChainedAudit` for testing.
#[derive(Debug, Default)]
pub struct InMemoryChain {
    entries: Vec<AuditEntry>,
}

impl InMemoryChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }
}

impl ChainedAudit for InMemoryChain {
    type Error = ChainError;

    fn tip_hash(&self, resource: &str) -> Result<[u8; 32], ChainError> {
        Ok(self
            .entries
            .iter()
            .rfind(|e| e.resource == resource)
            .map(|e| e.this_hash)
            .unwrap_or([0u8; 32]))
    }

    fn persist(&mut self, entry: &AuditEntry) -> Result<(), ChainError> {
        self.entries.push(entry.clone());
        Ok(())
    }
}

/// Errors from chain operations.
#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("chain broken at entry {entry_id}: hash mismatch")]
    HashMismatch { entry_id: String },
    #[error("missing entry between {prev} and {next}")]
    MissingEntry { prev: String, next: String },
    #[error("empty chain")]
    EmptyChain,
}

/// Stateless chain verifier.  Call `verify_all` on an ordered slice of entries.
pub struct ChainVerifier;

impl ChainVerifier {
    /// Verify a sequence of AuditEntries for one resource, ordered oldest first.
    ///
    /// Checks:
    /// 1. Each entry's `this_hash` matches the recomputed value.
    /// 2. Each entry's `prev_hash` equals the previous entry's `this_hash`.
    /// 3. The first entry has `prev_hash == [0u8; 32]`.
    pub fn verify_all(entries: &[AuditEntry]) -> Result<(), ChainError> {
        if entries.is_empty() {
            return Err(ChainError::EmptyChain);
        }
        let mut expected_prev = [0u8; 32];
        for entry in entries {
            // Self-consistency
            if !entry.is_self_consistent() {
                return Err(ChainError::HashMismatch {
                    entry_id: entry.entry_id.clone(),
                });
            }
            // Chain continuity
            if entry.prev_hash != expected_prev {
                return Err(ChainError::HashMismatch {
                    entry_id: entry.entry_id.clone(),
                });
            }
            expected_prev = entry.this_hash;
        }
        Ok(())
    }
}
