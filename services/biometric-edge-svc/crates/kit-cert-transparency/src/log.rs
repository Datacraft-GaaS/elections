//! KitCertTransparencyLog: RFC 6962-style Merkle append-only log for kit certificates.
//!
//! Uses `eel-crypto-rs` Merkle primitives (SHA-256, 0x00/0x01 domain separation).
//! The log is append-only in memory; persistence is the caller's responsibility.
//!
//! Reference: Laurie, Langley, Kasper, "Certificate Transparency", RFC 6962 (2013).

use std::sync::{Arc, Mutex};

use eel_crypto_rs::merkle::MerkleTree;
use serde::{Deserialize, Serialize};
use sha2::Digest as _;
use thiserror::Error;

use crate::entry::KitCertEntry;

#[derive(Debug, Error)]
pub enum LogError {
    #[error("entry index {0} is out of range (tree size {1})")]
    IndexOutOfRange(usize, usize),
    #[error("inclusion proof verification failed")]
    BadInclusionProof,
    #[error("consistency proof verification failed")]
    BadConsistencyProof,
    #[error("log is empty")]
    EmptyLog,
}

/// Signed Tree Head (STH) as defined in RFC 6962 §3.5.
///
/// Signing is done externally; this struct carries the pre-image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTreeHead {
    /// Number of leaves at the time of signing.
    pub tree_size: usize,
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// Hex-encoded SHA-256 Merkle root.
    pub sha256_root_hash: String,
    /// Opaque signature bytes (hex). Empty until signed externally.
    pub tree_head_signature: String,
}

/// An inclusion proof for one log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    pub leaf_index: usize,
    pub tree_size: usize,
    /// Sibling hashes from leaf to root, hex-encoded.
    pub audit_path: Vec<String>,
}

/// An in-memory append-only CT log for kit certificates.
#[derive(Clone)]
pub struct KitCertTransparencyLog {
    inner: Arc<Mutex<Vec<KitCertEntry>>>,
}

impl KitCertTransparencyLog {
    /// Create an empty log.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Append a new entry. Returns the leaf index assigned.
    pub fn append(&self, entry: KitCertEntry) -> usize {
        let mut g = self.inner.lock().unwrap();
        let idx = g.len();
        g.push(entry);
        idx
    }

    /// Return the current tree size (number of leaves).
    pub fn tree_size(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// Return a clone of the entry at `index`, or `None` if out of range.
    pub fn entry_at(&self, index: usize) -> Option<KitCertEntry> {
        self.inner.lock().unwrap().get(index).cloned()
    }

    /// Return the current Signed Tree Head (unsigned).
    pub fn current_sth(&self) -> SignedTreeHead {
        let g = self.inner.lock().unwrap();
        let root = if g.is_empty() {
            // RFC 6962 §2.1: empty tree hash = SHA256("").
            let hash = sha2::Sha256::new().finalize();
            hex::encode(hash)
        } else {
            let raw: Vec<Vec<u8>> = g.iter().map(|e| e.to_leaf_bytes()).collect();
            let refs: Vec<&[u8]> = raw.iter().map(|v| v.as_slice()).collect();
            hex::encode(MerkleTree::build(&refs).root())
        };
        SignedTreeHead {
            tree_size: g.len(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            sha256_root_hash: root,
            tree_head_signature: String::new(),
        }
    }

    /// Generate an inclusion proof for the entry at `leaf_index`.
    pub fn inclusion_proof(&self, leaf_index: usize) -> Result<InclusionProof, LogError> {
        let g = self.inner.lock().unwrap();
        let n = g.len();
        if leaf_index >= n {
            return Err(LogError::IndexOutOfRange(leaf_index, n));
        }
        let raw: Vec<Vec<u8>> = g.iter().map(|e| e.to_leaf_bytes()).collect();
        let refs: Vec<&[u8]> = raw.iter().map(|v| v.as_slice()).collect();
        let tree = MerkleTree::build(&refs);
        let proof = tree
            .inclusion_proof(leaf_index)
            .map_err(|_| LogError::IndexOutOfRange(leaf_index, n))?;
        Ok(InclusionProof {
            leaf_index,
            tree_size: n,
            audit_path: proof.iter().map(hex::encode).collect(),
        })
    }

    /// Verify an inclusion proof for `entry` at `leaf_index`.
    pub fn verify_inclusion(
        &self,
        leaf_index: usize,
        entry: &KitCertEntry,
        proof: &InclusionProof,
    ) -> Result<(), LogError> {
        let g = self.inner.lock().unwrap();
        let n = g.len();
        if leaf_index >= n {
            return Err(LogError::IndexOutOfRange(leaf_index, n));
        }
        let raw: Vec<Vec<u8>> = g.iter().map(|e| e.to_leaf_bytes()).collect();
        let refs: Vec<&[u8]> = raw.iter().map(|v| v.as_slice()).collect();
        let tree = MerkleTree::build(&refs);

        let audit_path: Vec<[u8; 32]> = proof
            .audit_path
            .iter()
            .map(|s| {
                let v = hex::decode(s).map_err(|_| LogError::BadInclusionProof)?;
                let mut arr = [0u8; 32];
                if v.len() != 32 {
                    return Err(LogError::BadInclusionProof);
                }
                arr.copy_from_slice(&v);
                Ok(arr)
            })
            .collect::<Result<_, _>>()?;

        tree.verify_inclusion(leaf_index, &entry.to_leaf_bytes(), &audit_path)
            .map_err(|_| LogError::BadInclusionProof)
    }

    /// Generate a consistency proof between STHs at `old_size` and `new_size`.
    pub fn consistency_proof(
        &self,
        old_size: usize,
        new_size: usize,
    ) -> Result<Vec<String>, LogError> {
        let g = self.inner.lock().unwrap();
        let n = g.len();
        if new_size > n || old_size > new_size {
            return Err(LogError::IndexOutOfRange(new_size, n));
        }
        if old_size == 0 || old_size == new_size {
            return Ok(vec![]);
        }
        let raw: Vec<Vec<u8>> = g[..new_size].iter().map(|e| e.to_leaf_bytes()).collect();
        let refs: Vec<&[u8]> = raw.iter().map(|v| v.as_slice()).collect();
        let tree = MerkleTree::build(&refs);
        let path = tree
            .consistency_proof(old_size)
            .map_err(|_| LogError::BadConsistencyProof)?;
        Ok(path.iter().map(hex::encode).collect())
    }

    /// Return the root hash for a tree of `size` leaves.
    ///
    /// Used by verifiers to confirm an old STH root against newly appended data.
    pub fn root_at(&self, size: usize) -> Result<String, LogError> {
        let g = self.inner.lock().unwrap();
        let n = g.len();
        if size > n {
            return Err(LogError::IndexOutOfRange(size, n));
        }
        if size == 0 {
            let hash = sha2::Sha256::new().finalize();
            return Ok(hex::encode(hash));
        }
        let raw: Vec<Vec<u8>> = g[..size].iter().map(|e| e.to_leaf_bytes()).collect();
        let refs: Vec<&[u8]> = raw.iter().map(|v| v.as_slice()).collect();
        Ok(hex::encode(MerkleTree::build(&refs).root()))
    }
}

impl Default for KitCertTransparencyLog {
    fn default() -> Self {
        Self::new()
    }
}
