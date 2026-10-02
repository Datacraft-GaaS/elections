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

//! Core sequencer: append leaf, compute Merkle tree, return inclusion proof.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tlog_domain::{Domain, InclusionProof, LeafFormat};

use crate::rate::SequencerRateLimiter;

/// Result of a successful leaf append.
#[derive(Debug)]
pub struct AppendResult {
    pub leaf_index: u64,
    pub tree_size: u64,
    pub proof: InclusionProof,
}

/// The sequencer engine.
pub struct Sequencer {
    pool: Arc<PgPool>,
    limiter: Arc<SequencerRateLimiter>,
}

impl Sequencer {
    pub fn new(pool: Arc<PgPool>, max_leaves_per_sec: u32) -> Self {
        Self {
            pool,
            limiter: Arc::new(SequencerRateLimiter::new(max_leaves_per_sec)),
        }
    }

    /// Append a leaf to the log.
    ///
    /// Returns `None` if the rate limiter is saturated (caller should respond 429).
    pub async fn append(
        &self,
        domain: Domain,
        canonical_event_json: &[u8],
        producer_spiffe_id: &str,
        source_event_id: &str,
        source_topic: &str,
        event_time: DateTime<Utc>,
        producer_signature: &[u8],
    ) -> Result<Option<AppendResult>, crate::SequencerError> {
        let _permit = match self.limiter.try_acquire() {
            Some(p) => p,
            None => return Ok(None),
        };

        // Deduplicate by (topic, event_id).
        if crate::db::leaf_exists(&self.pool, source_topic, source_event_id).await? {
            let tree_sz = crate::db::tree_size(&self.pool).await? as u64;
            return Ok(Some(AppendResult {
                leaf_index: 0,
                tree_size: tree_sz,
                proof: InclusionProof {
                    leaf_index: 0,
                    tree_size: tree_sz,
                    root_hash: String::new(),
                    hashes: vec![],
                },
            }));
        }

        let seq = crate::db::tree_size(&self.pool).await? as u64;
        let leaf_hash_bytes =
            LeafFormat::hash(canonical_event_json, producer_spiffe_id, event_time, seq);

        let index = crate::db::append_leaf(
            &self.pool,
            domain,
            &leaf_hash_bytes,
            producer_spiffe_id,
            source_event_id,
            source_topic,
            event_time,
            producer_signature,
        )
        .await? as u64;

        let new_size = index + 1;

        // Compute the real inclusion proof against the full tree.
        let hashes = crate::proofs::load_leaf_hashes(&self.pool).await?;
        let tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(hashes);
        let path = tree
            .inclusion_proof(index as usize)
            .map_err(|e| crate::SequencerError::Proof(e.to_string()))?;

        Ok(Some(AppendResult {
            leaf_index: index,
            tree_size: new_size,
            proof: InclusionProof {
                leaf_index: index,
                tree_size: new_size,
                root_hash: hex::encode(tree.root()),
                hashes: path.iter().map(|h| hex::encode(h)).collect(),
            },
        }))
    }
}
