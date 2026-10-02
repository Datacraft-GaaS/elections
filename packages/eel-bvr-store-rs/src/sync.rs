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

//! Periodic sync: batching, Merkle roots, and acknowledgement.

use serde::{Deserialize, Serialize};

use crate::identity::InstanceIdentity;
use crate::record::Record;

/// How much of a store has been acknowledged by the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncState {
    /// Records held in total.
    pub total: usize,
    /// Records the server has acknowledged.
    pub synced: usize,
    /// Records still waiting to be sent.
    pub pending: usize,
}

/// A unit of sync: the unsynced records plus a Merkle root over them.
///
/// The root is the point of the batch. Once the server anchors it in the
/// transparency log, anyone can later prove that a given record was in the
/// batch, and that the batch has not been quietly rewritten -- which is what
/// upgrades local tamper evidence into third-party-verifiable evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Batch {
    /// The station that produced the batch.
    pub instance_id: InstanceIdentity,
    /// Records awaiting acknowledgement, in capture order.
    pub records: Vec<Record>,
    /// BLAKE3 Merkle root over the record hashes, hex encoded.
    pub merkle_root: String,
}

impl Batch {
    /// Builds a batch and computes its Merkle root.
    pub fn new(instance: InstanceIdentity, records: Vec<Record>) -> Self {
        let leaves: Vec<eel_crypto_rs::merkle::Hash> = records
            .iter()
            .map(|r| {
                let bytes = hex_decode(&r.this_hash);
                let mut leaf = [0u8; 32];
                leaf.copy_from_slice(&bytes);
                leaf
            })
            .collect();
        let tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(leaves);
        Self { instance_id: instance, records, merkle_root: hex_encode(&tree.root()) }
    }

    /// Number of records in the batch.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns the inclusion proof for the record at `index`.
    pub fn inclusion_proof(&self, index: usize) -> Option<Vec<[u8; 32]>> {
        if index >= self.records.len() {
            return None;
        }
        let leaves: Vec<eel_crypto_rs::merkle::Hash> = self
            .records
            .iter()
            .map(|r| {
                let bytes = hex_decode(&r.this_hash);
                let mut leaf = [0u8; 32];
                leaf.copy_from_slice(&bytes);
                leaf
            })
            .collect();
        let tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(leaves);
        tree.inclusion_proof(index).ok()
    }
}

/// Decodes lowercase hex into bytes.
fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

/// Encodes bytes as lowercase hex.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
