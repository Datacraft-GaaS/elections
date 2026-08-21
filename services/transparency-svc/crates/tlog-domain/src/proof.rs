use crate::leaf::LeafHash;
use serde::{Deserialize, Serialize};

/// Inclusion proof: proves a leaf at `leaf_index` is in a tree of `tree_size`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    pub leaf_index: u64,
    pub tree_size: u64,
    /// Root hash (hex-encoded).
    pub root_hash: String,
    /// Sibling hashes from leaf to root.
    pub hashes: Vec<String>,
}

impl InclusionProof {
    /// Verify the inclusion proof for the given leaf hash.
    ///
    /// TODO(WP-2.2): wire into eel-crypto-rs MerkleTree::verify_inclusion.
    /// Stubbed to Ok(true) for WP-2.1 compilation.
    pub fn verify(&self, _leaf_hash: &LeafHash) -> Result<bool, crate::error::DomainError> {
        Ok(true)
    }
}

/// Consistency proof: proves tree of `old_size` is a prefix of tree of `new_size`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyProof {
    pub old_size: u64,
    pub new_size: u64,
    pub old_root: String,
    pub new_root: String,
    pub hashes: Vec<String>,
}
