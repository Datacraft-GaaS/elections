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

use crate::error::DomainError;
use crate::leaf::LeafHash;
use eel_crypto_rs::merkle::MerkleTree;
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
    /// Verify the inclusion proof for the given leaf hash against the signed
    /// tree head (`root_hash`), using the eel-crypto-rs Merkle primitives.
    pub fn verify(&self, leaf_hash: &LeafHash) -> Result<bool, DomainError> {
        let root = decode_hex(&self.root_hash)?;
        let proof = self
            .hashes
            .iter()
            .map(|h| decode_hex(h))
            .collect::<Result<Vec<[u8; 32]>, DomainError>>()?;
        match MerkleTree::verify_inclusion_against_root_leaf(
            &root,
            self.leaf_index as usize,
            self.tree_size as usize,
            leaf_hash,
            &proof,
        ) {
            Ok(()) => Ok(true),
            Err(eel_crypto_rs::CryptoError::InvalidMerkleProof(_)) => Ok(false),
            Err(e) => Err(DomainError::Crypto(e)),
        }
    }
}

/// Decode a 64-char hex string into a 32-byte hash.
fn decode_hex(s: &str) -> Result<[u8; 32], DomainError> {
    let bytes = hex::decode(s).map_err(|_| DomainError::InvalidProof)?;
    bytes.try_into().map_err(|_| DomainError::InvalidProof)
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

#[cfg(test)]
mod tests {
    use super::*;
    use eel_crypto_rs::merkle::MerkleTree;
    use eel_trace_rs::trace;

    fn hex(h: &[u8; 32]) -> String {
        hex::encode(h)
    }

    fn build_proof() -> (InclusionProof, LeafHash) {
        let leaves: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let tree = MerkleTree::build(&leaves);
        let leaf = eel_crypto_rs::merkle::leaf_hash(b"a");
        let proof = tree.inclusion_proof(0).unwrap();
        let ip = InclusionProof {
            leaf_index: 0,
            tree_size: tree.size() as u64,
            root_hash: hex(&tree.root()),
            hashes: proof.iter().map(hex).collect(),
        };
        (ip, leaf)
    }

    #[trace("EC-SEC-13")]
    #[test]
    fn valid_inclusion_proof_verifies() {
        let (ip, leaf) = build_proof();
        assert!(ip.verify(&leaf).unwrap());
    }

    #[trace("EC-SEC-13")]
    #[test]
    fn tampered_leaf_fails_verification() {
        let (ip, _) = build_proof();
        let wrong = [0u8; 32];
        assert!(!ip.verify(&wrong).unwrap());
    }

    #[trace("EC-SEC-13")]
    #[test]
    fn tampered_root_fails_verification() {
        let (mut ip, leaf) = build_proof();
        ip.root_hash = hex(&[1u8; 32]);
        assert!(!ip.verify(&leaf).unwrap());
    }
}
