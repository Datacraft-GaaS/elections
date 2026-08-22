//! HotStuff protocol messages.
//!
//! Five message types per Yin-Malkhi-Reiter-Gueta-Abraham PODC 2019:
//!   NewView, Prepare, PreCommit, Commit, Decide.
//!
//! Each message carries: view_number, block_hash, optional block payload,
//! and an optional justify QuorumCertificate from the prior phase.

use serde::{Deserialize, Serialize};

use crate::certificate::QuorumCertificate;
use crate::replica::ReplicaId;

/// A tally block: the unit of agreement in HotStuff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// Sequential height within the chain.
    pub height: u64,
    /// Blake3 hash of the payload.
    pub hash: [u8; 32],
    /// Hash of the parent block (zero for genesis).
    pub parent_hash: [u8; 32],
    /// Opaque tally payload (serialised Form34A aggregate).
    pub payload: Vec<u8>,
}

impl Block {
    /// Compute the Blake3 hash of height + parent_hash + payload.
    pub fn compute_hash(height: u64, parent_hash: &[u8; 32], payload: &[u8]) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(&height.to_le_bytes());
        h.update(parent_hash);
        h.update(payload);
        *h.finalize().as_bytes()
    }

    /// Build a new block and auto-compute its hash.
    pub fn new(height: u64, parent_hash: [u8; 32], payload: Vec<u8>) -> Self {
        let hash = Self::compute_hash(height, &parent_hash, &payload);
        Self {
            height,
            hash,
            parent_hash,
            payload,
        }
    }

    pub fn genesis() -> Self {
        Self {
            height: 0,
            hash: [0u8; 32],
            parent_hash: [0u8; 32],
            payload: vec![],
        }
    }
}

/// The five HotStuff message types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    NewView,
    Prepare,
    PreCommit,
    Commit,
    Decide,
}

/// A HotStuff protocol message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotStuffMessage {
    pub msg_type: MessageType,
    /// Current view number.
    pub view_number: u64,
    /// Hash of the block this message refers to.
    pub block_hash: [u8; 32],
    /// Block payload (present in Prepare messages from leader).
    pub block: Option<Block>,
    /// Justify: the highest QC the sender has seen.
    pub justify: Option<QuorumCertificate>,
    /// Sender replica ID.
    pub sender: ReplicaId,
    /// Ed25519 signature over (msg_type || view_number || block_hash).
    pub signature: Vec<u8>,
}

impl HotStuffMessage {
    /// Bytes to sign: big-endian encoding of msg_type index || view_number || block_hash.
    pub fn signing_bytes(
        msg_type: &MessageType,
        view_number: u64,
        block_hash: &[u8; 32],
    ) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1 + 8 + 32);
        buf.push(msg_type_index(msg_type));
        buf.extend_from_slice(&view_number.to_be_bytes());
        buf.extend_from_slice(block_hash);
        buf
    }
}

fn msg_type_index(t: &MessageType) -> u8 {
    match t {
        MessageType::NewView => 0,
        MessageType::Prepare => 1,
        MessageType::PreCommit => 2,
        MessageType::Commit => 3,
        MessageType::Decide => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_hash_deterministic() {
        let b1 = Block::new(1, [0u8; 32], b"payload".to_vec());
        let b2 = Block::new(1, [0u8; 32], b"payload".to_vec());
        assert_eq!(b1.hash, b2.hash);
    }

    #[test]
    fn block_hash_changes_with_payload() {
        let b1 = Block::new(1, [0u8; 32], b"payload_a".to_vec());
        let b2 = Block::new(1, [0u8; 32], b"payload_b".to_vec());
        assert_ne!(b1.hash, b2.hash);
    }

    #[test]
    fn signing_bytes_different_per_type() {
        let h = [0u8; 32];
        let a = HotStuffMessage::signing_bytes(&MessageType::Prepare, 1, &h);
        let b = HotStuffMessage::signing_bytes(&MessageType::Commit, 1, &h);
        assert_ne!(a, b);
    }
}
