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

//! A stored record, its hash chain, and chain verification.

use chrono::{DateTime, Utc};
use eel_crypto_rs::{blake3, ed25519, rfc8785};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::keystore::KeyStore;
use crate::provenance::Provenance;

/// Identifier of a stored record.
pub type RecordId = Uuid;

/// Why a store failed its integrity check.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TamperError {
    /// A record's hash does not match its contents.
    #[error("record {id} content does not match its recorded hash")]
    ContentAltered {
        /// The offending record.
        id: String,
    },
    /// A record's hash does not commit to its predecessor.
    #[error("record {id} does not chain to its predecessor")]
    ChainBroken {
        /// The offending record.
        id: String,
    },
    /// A record's signature did not verify under the instance key.
    #[error("record {id} signature is invalid")]
    SignatureInvalid {
        /// The offending record.
        id: String,
    },
    /// A record could not be decrypted.
    #[error("record {id} could not be decrypted")]
    Undecryptable {
        /// The offending record.
        id: String,
    },
}

/// A single captured datum: its value, its provenance, and its integrity proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    /// This record's id.
    pub record_id: RecordId,
    /// Name of the data item, matching the workflow's `data_items[].name`.
    pub data_item: String,
    /// PII tier, `A` through `E`.
    pub data_class: String,
    /// The captured value. Encrypted at rest; see [`Record::seal`].
    pub value: Value,
    /// Where, when, who, why, how.
    pub provenance: Provenance,
    /// Hash of the previous record, empty for the first.
    pub prev_hash: String,
    /// BLAKE3 hash of this record's content, chained to `prev_hash`.
    pub this_hash: String,
    /// Ed25519 signature over `this_hash`, by the instance key.
    pub signature: String,
    /// Whether the server has acknowledged receipt.
    pub synced: bool,
}

impl Record {
    /// The bytes covered by this record's hash: everything that must not change
    /// without detection. `synced` is excluded on purpose -- sync state is
    /// mutable bookkeeping, not captured content, and folding it in would make
    /// acknowledging a record mutate its integrity proof.
    fn preimage(&self) -> Vec<u8> {
        let body = serde_json::json!({
            "record_id": self.record_id.to_string(),
            "data_item": self.data_item,
            "data_class": self.data_class,
            "value": &self.value,
            "provenance": &self.provenance,
            "prev_hash": self.prev_hash,
        });
        rfc8785::canonicalize(&body).unwrap_or_default()
    }

    /// Computes the chained hash for a record's current content.
    pub fn compute_hash(&self) -> String {
        let canonical = self.preimage();
        // Chain by hashing the predecessor into this record's content hash.
        let mut chained = Vec::with_capacity(canonical.len() + 64);
        chained.extend_from_slice(self.prev_hash.as_bytes());
        chained.extend_from_slice(&canonical);
        hex_encode(&blake3::hash(&chained))
    }

    /// Recomputes `this_hash`, signs it, and returns the updated record.
    pub fn seal(mut self, keys: &impl KeyStore) -> Result<Self, eel_crypto_rs::CryptoError> {
        self.this_hash = self.compute_hash();
        let signature = keys.sign(self.this_hash.as_bytes())?;
        self.signature =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature);
        Ok(self)
    }

    /// Recomputes the hash and compares it to the recorded value.
    pub fn verify_content(&self) -> Result<(), TamperError> {
        if self.compute_hash() != self.this_hash {
            return Err(TamperError::ContentAltered {
                id: self.record_id.to_string(),
            });
        }
        Ok(())
    }

    /// Verifies the record's signature against `public_key`.
    pub fn verify_signature(&self, public_key: &[u8; 32]) -> Result<(), TamperError> {
        let bad = || TamperError::SignatureInvalid {
            id: self.record_id.to_string(),
        };
        let raw =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &self.signature)
                .map_err(|_| bad())?;
        let sig = ed25519::signature_from_bytes(&raw.try_into().map_err(|_| bad())?)
            .map_err(|_| bad())?;
        let key = ed25519::verifying_key_from_bytes(public_key).map_err(|_| bad())?;
        ed25519::verify(&key, self.this_hash.as_bytes(), &sig).map_err(|_| bad())
    }

    /// Returns `true` when the record has been acknowledged by the server.
    pub fn is_synced(&self) -> bool {
        self.synced
    }

    /// Captured instant, for ordering and reporting.
    pub fn captured_at(&self) -> DateTime<Utc> {
        self.provenance.captured_at
    }
}

/// Verifies the content hash, the chain linkage, and the signature of each
/// record in order.
///
/// Checks all three properties for every record rather than stopping at the
/// first fault, so an auditor gets the full picture of an altered store in one
/// pass. An empty slice is valid: a store with no records is not a tampered one.
pub fn verify_chain(records: &[Record], public_key: &[u8; 32]) -> Result<(), Vec<TamperError>> {
    let mut errors = Vec::new();
    let mut expected_prev = String::new();

    for record in records {
        if let Err(e) = record.verify_content() {
            errors.push(e);
        }
        if record.prev_hash != expected_prev {
            errors.push(TamperError::ChainBroken {
                id: record.record_id.to_string(),
            });
        }
        if let Err(e) = record.verify_signature(public_key) {
            errors.push(e);
        }
        expected_prev = record.this_hash.clone();
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Encodes bytes as lowercase hex.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
