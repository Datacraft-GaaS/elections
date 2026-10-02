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

use crate::domain::Domain;
use blake3::Hasher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A 32-byte leaf hash (BLAKE3).
pub type LeafHash = [u8; 32];

/// Canonical leaf format per ADR-0017.
///
/// `hash( canonical_json(event) || producer_spiffe_id || timestamp || sequence )`
pub struct LeafFormat;

impl LeafFormat {
    /// Compute the leaf hash from its components.
    pub fn hash(
        canonical_event_json: &[u8],
        producer_spiffe_id: &str,
        timestamp: DateTime<Utc>,
        sequence: u64,
    ) -> LeafHash {
        let mut h = Hasher::new();
        h.update(canonical_event_json);
        h.update(producer_spiffe_id.as_bytes());
        h.update(&timestamp.timestamp_micros().to_le_bytes());
        h.update(&sequence.to_le_bytes());
        *h.finalize().as_bytes()
    }
}

/// A single entry in the transparency log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leaf {
    /// Leaf index in the append-only tree (0-based).
    pub index: u64,
    /// Domain tag.
    pub domain: Domain,
    /// Pre-computed leaf hash (hex-encoded for storage/transport).
    pub hash: String,
    /// SPIFFE URI of the producing service.
    pub producer_spiffe_id: String,
    /// Source event id (CloudEvents `id` field).
    pub source_event_id: String,
    /// Source Kafka topic.
    pub source_topic: String,
    /// Wall-clock time the event was produced (NTP-authenticated).
    pub event_time: DateTime<Utc>,
    /// Time the log sequencer appended this leaf.
    pub appended_at: DateTime<Utc>,
    /// Ed25519 signature from the producer (hex-encoded).
    pub producer_signature: String,
}

impl Leaf {
    /// Decode the hash bytes from the hex-encoded field.
    pub fn hash_bytes(&self) -> Result<LeafHash, crate::error::DomainError> {
        let bytes = hex::decode(&self.hash).map_err(|_| crate::error::DomainError::HashMismatch)?;
        bytes
            .try_into()
            .map_err(|_| crate::error::DomainError::HashMismatch)
    }
}
