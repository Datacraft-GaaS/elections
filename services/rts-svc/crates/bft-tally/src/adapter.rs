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

//! BftTallyAdapter: drop-in replacement for the Kafka-transactional tally consumer.
//!
//! The adapter wraps a replica and drives it through one HotStuff view per
//! Kafka message batch. Every committed block requires a valid QC before its
//! payload is forwarded to downstream aggregation.
//!
//! The interface mirrors the TallyConsumer in rts-svc/crates/rts-tally.

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::certificate::QuorumCertificate;
use crate::message::Block;
use crate::{BftError, QUORUM};

/// A raw Kafka message payload (opaque bytes).
#[derive(Debug, Clone)]
pub struct KafkaMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub payload: Vec<u8>,
}

/// Result of processing one Kafka message through the BFT layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BftCommitResult {
    /// Block height in the BFT chain.
    pub height: u64,
    /// Blake3 hash of the committed block.
    pub block_hash: [u8; 32],
    /// Number of signatures in the QC.
    pub qc_signatures: usize,
    /// The committed payload (forwarded to downstream tally aggregator).
    pub payload: Vec<u8>,
}

/// Simulates the network layer for tests: collects votes from N local replicas.
pub struct LocalNetwork {
    /// How many replicas are functioning (simulates crash faults).
    pub healthy: usize,
}

impl LocalNetwork {
    pub fn new(healthy: usize) -> Self {
        Self { healthy }
    }

    /// Gather QUORUM votes from the healthy replicas.
    ///
    /// In production this would be replaced by actual network I/O.
    pub fn gather_votes(
        &self,
        signing_keys: &[ed25519_dalek::SigningKey],
        msg_type: &crate::message::MessageType,
        view: u64,
        block_hash: &[u8; 32],
    ) -> Result<Vec<crate::certificate::Vote>, BftError> {
        use ed25519_dalek::Signer;

        let available = self.healthy.min(signing_keys.len());
        if available < QUORUM {
            return Err(BftError::LivenessLost {
                faulty: signing_keys.len() - available,
                f: crate::F,
            });
        }

        let signing_bytes =
            crate::message::HotStuffMessage::signing_bytes(msg_type, view, block_hash);
        let votes = signing_keys[..available]
            .iter()
            .enumerate()
            .map(|(i, sk)| {
                let sig = sk.sign(&signing_bytes);
                crate::certificate::Vote {
                    replica_id: i as u64,
                    signature: sig.to_bytes().to_vec(),
                }
            })
            .collect();

        Ok(votes)
    }
}

/// BFT tally adapter. Drives a one-shot consensus round per message batch.
pub struct BftTallyAdapter {
    pub height: u64,
    pub last_hash: [u8; 32],
}

impl BftTallyAdapter {
    pub fn new() -> Self {
        Self {
            height: 0,
            last_hash: [0u8; 32],
        }
    }

    /// Process one Kafka message: build a block, simulate consensus, return commit result.
    ///
    /// `qc` must be a valid QC for the block (provided by the network layer in production).
    pub fn process(
        &mut self,
        msg: &KafkaMessage,
        qc: QuorumCertificate,
    ) -> Result<BftCommitResult, BftError> {
        self.height += 1;
        let block = Block::new(self.height, self.last_hash, msg.payload.clone());

        // Verify the block hash matches what the QC covers.
        if qc.block_hash != block.hash {
            return Err(BftError::HashMismatch);
        }

        if qc.votes.len() < QUORUM {
            return Err(BftError::InsufficientSignatures {
                got: qc.votes.len(),
                required: QUORUM,
            });
        }

        self.last_hash = block.hash;
        info!(height = self.height, "BftTallyAdapter: block committed");

        Ok(BftCommitResult {
            height: block.height,
            block_hash: block.hash,
            qc_signatures: qc.votes.len(),
            payload: block.payload,
        })
    }
}

impl Default for BftTallyAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::Vote;
    use crate::message::{Block, MessageType};

    fn make_qc(block_hash: [u8; 32], n_votes: usize) -> QuorumCertificate {
        let votes: Vec<Vote> = (0..n_votes)
            .map(|i| Vote {
                replica_id: i as u64,
                signature: vec![0u8; 64],
            })
            .collect();
        QuorumCertificate {
            msg_type: MessageType::Decide,
            view_number: 0,
            block_hash,
            votes,
        }
    }

    #[test]
    fn adapter_accepts_valid_qc() {
        let mut adapter = BftTallyAdapter::new();
        let payload = b"tally_payload".to_vec();
        let block = Block::new(1, [0u8; 32], payload.clone());
        let qc = make_qc(block.hash, QUORUM);
        let msg = KafkaMessage {
            topic: "eel.rts.form34a.v1".into(),
            partition: 0,
            offset: 0,
            payload,
        };
        let result = adapter.process(&msg, qc).unwrap();
        assert_eq!(result.height, 1);
        assert_eq!(result.qc_signatures, QUORUM);
    }

    #[test]
    fn adapter_rejects_wrong_hash() {
        let mut adapter = BftTallyAdapter::new();
        let payload = b"tally_payload".to_vec();
        let wrong_hash = [1u8; 32];
        let qc = make_qc(wrong_hash, QUORUM);
        let msg = KafkaMessage {
            topic: "eel.rts.form34a.v1".into(),
            partition: 0,
            offset: 0,
            payload,
        };
        let err = adapter.process(&msg, qc).unwrap_err();
        assert!(matches!(err, BftError::HashMismatch));
    }

    #[test]
    fn adapter_rejects_insufficient_qc() {
        let mut adapter = BftTallyAdapter::new();
        let payload = b"data".to_vec();
        let block = Block::new(1, [0u8; 32], payload.clone());
        let qc = make_qc(block.hash, QUORUM - 1);
        let msg = KafkaMessage {
            topic: "eel.rts.form34a.v1".into(),
            partition: 0,
            offset: 0,
            payload,
        };
        let err = adapter.process(&msg, qc).unwrap_err();
        assert!(matches!(err, BftError::InsufficientSignatures { .. }));
    }
}
