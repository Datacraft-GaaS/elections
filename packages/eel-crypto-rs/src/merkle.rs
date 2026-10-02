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

//! RFC 6962 Certificate Transparency Merkle tree primitives.
//!
//! Implements leaf hashing, internal node hashing, inclusion proofs, and
//! consistency proofs as specified in RFC 6962 §2.
//!
//! Hash function: SHA-256.
//! Leaf hash:     `SHA256(0x00 || data)`
//! Node hash:     `SHA256(0x01 || left || right)`
//!
//! # Example
//!
//! ```
//! use eel_crypto_rs::merkle::{leaf_hash, MerkleTree};
//! let entries: Vec<&[u8]> = vec![b"alpha", b"beta", b"gamma"];
//! let tree = MerkleTree::build(&entries);
//! let proof = tree.inclusion_proof(0).unwrap();
//! assert!(tree.verify_inclusion(0, b"alpha", &proof).is_ok());
//! ```

use sha2::{Digest, Sha256};

use crate::CryptoError;

/// A 32-byte SHA-256 hash value.
pub type Hash = [u8; 32];

// ---------------------------------------------------------------------------
// Hash functions
// ---------------------------------------------------------------------------

/// Compute the RFC 6962 leaf hash: `SHA256(0x00 || data)`.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::merkle::leaf_hash;
/// let h = leaf_hash(b"data");
/// assert_eq!(h.len(), 32);
/// ```
pub fn leaf_hash(data: &[u8]) -> Hash {
    let mut h = Sha256::new();
    h.update([0x00]);
    h.update(data);
    h.finalize().into()
}

/// Compute the RFC 6962 internal node hash: `SHA256(0x01 || left || right)`.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::merkle::node_hash;
/// let l = [0u8; 32];
/// let r = [1u8; 32];
/// let h = node_hash(&l, &r);
/// assert_eq!(h.len(), 32);
/// ```
pub fn node_hash(left: &Hash, right: &Hash) -> Hash {
    let mut h = Sha256::new();
    h.update([0x01]);
    h.update(left);
    h.update(right);
    h.finalize().into()
}

// ---------------------------------------------------------------------------
// MerkleTree
// ---------------------------------------------------------------------------

/// An in-memory RFC 6962 Merkle tree.
///
/// Built from a list of leaf entries.  Supports inclusion and consistency
/// proof generation and verification.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::merkle::MerkleTree;
/// let tree = MerkleTree::build(&[b"a" as &[u8], b"b", b"c"]);
/// assert_eq!(tree.size(), 3);
/// ```
pub struct MerkleTree {
    leaves: Vec<Hash>,
    /// Full hash tree stored in level-order.  Index 0 is the root.
    nodes: Vec<Hash>,
    size: usize,
}

impl MerkleTree {
    /// Build a Merkle tree from a slice of leaf data entries.
    ///
    /// An empty slice produces a tree whose root is `SHA256("")`.
    ///
    /// # Example
    ///
    /// ```
    /// use eel_crypto_rs::merkle::MerkleTree;
    /// let tree = MerkleTree::build(&[b"x" as &[u8]]);
    /// assert_eq!(tree.size(), 1);
    /// ```
    pub fn build(entries: &[&[u8]]) -> Self {
        let leaves: Vec<Hash> = entries.iter().map(|e| leaf_hash(e)).collect();
        let size = leaves.len();
        let nodes = build_tree(&leaves);
        MerkleTree {
            leaves,
            nodes,
            size,
        }
    }

    /// Build a Merkle tree from pre-computed leaf hashes.
    ///
    /// Unlike [`MerkleTree::build`], the entries are treated as final leaf
    /// hashes and are NOT hashed again. Use this when the caller already
    /// holds domain-defined leaf hashes (e.g. transparency-log leaves stored
    /// in a database) so that [`MerkleTree::verify_inclusion_against_root_leaf`]
    /// verifies against those exact hashes.
    ///
    /// # Example
    ///
    /// ```
    /// use eel_crypto_rs::merkle::{leaf_hash, MerkleTree};
    /// let hashes = vec![leaf_hash(b"a"), leaf_hash(b"b")];
    /// let tree = MerkleTree::from_leaf_hashes(hashes);
    /// assert_eq!(tree.size(), 2);
    /// ```
    pub fn from_leaf_hashes(leaves: Vec<Hash>) -> Self {
        let size = leaves.len();
        let nodes = build_tree(&leaves);
        MerkleTree { leaves, nodes, size }
    }

    /// Return the number of leaves in the tree.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Return the tree root hash.
    ///
    /// # Example
    ///
    /// ```
    /// use eel_crypto_rs::merkle::MerkleTree;
    /// let t1 = MerkleTree::build(&[b"a" as &[u8]]);
    /// let t2 = MerkleTree::build(&[b"a" as &[u8]]);
    /// assert_eq!(t1.root(), t2.root());
    /// ```
    pub fn root(&self) -> Hash {
        if self.nodes.is_empty() {
            // Empty tree.
            let h = Sha256::new();
            h.finalize().into()
        } else {
            self.nodes[0]
        }
    }

    /// Generate an inclusion proof for leaf at `index`.
    ///
    /// Returns the ordered list of sibling hashes from leaf to root.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::InvalidMerkleProof`] if `index >= size`.
    ///
    /// # Example
    ///
    /// ```
    /// use eel_crypto_rs::merkle::MerkleTree;
    /// let t = MerkleTree::build(&[b"a" as &[u8], b"b", b"c"]);
    /// let proof = t.inclusion_proof(1).unwrap();
    /// assert!(!proof.is_empty());
    /// ```
    pub fn inclusion_proof(&self, index: usize) -> Result<Vec<Hash>, CryptoError> {
        if index >= self.size {
            return Err(CryptoError::InvalidMerkleProof(format!(
                "index {index} out of range (size {})",
                self.size
            )));
        }
        Ok(compute_inclusion_proof(&self.leaves, index))
    }

    /// Verify that `data` appears at position `index` using `proof`.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::InvalidMerkleProof`] if verification fails.
    ///
    /// # Example
    ///
    /// ```
    /// use eel_crypto_rs::merkle::MerkleTree;
    /// let t = MerkleTree::build(&[b"a" as &[u8], b"b"]);
    /// let proof = t.inclusion_proof(0).unwrap();
    /// assert!(t.verify_inclusion(0, b"a", &proof).is_ok());
    /// assert!(t.verify_inclusion(0, b"wrong", &proof).is_err());
    /// ```
    pub fn verify_inclusion(
        &self,
        index: usize,
        data: &[u8],
        proof: &[Hash],
    ) -> Result<(), CryptoError> {
        if index >= self.size {
            return Err(CryptoError::InvalidMerkleProof(format!(
                "index {index} out of range"
            )));
        }
        let computed = recompute_root(index, self.size, data, proof)?;
        if crate::constant_time::ct_eq(&computed, &self.root()) {
            Ok(())
        } else {
            Err(CryptoError::InvalidMerkleProof(
                "root hash mismatch".to_owned(),
            ))
        }
    }

    /// Verify an inclusion proof against a known root hash, without holding
    /// the full tree. Used by verifiers that only possess a signed tree head.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::InvalidMerkleProof`] if the proof does not
    /// recompute to `root`.
    pub fn verify_inclusion_against_root(
        root: &Hash,
        index: usize,
        tree_size: usize,
        data: &[u8],
        proof: &[Hash],
    ) -> Result<(), CryptoError> {
        if index >= tree_size {
            return Err(CryptoError::InvalidMerkleProof(format!(
                "index {index} out of range"
            )));
        }
        let computed = recompute_root(index, tree_size, data, proof)?;
        if crate::constant_time::ct_eq(&computed, root) {
            Ok(())
        } else {
            Err(CryptoError::InvalidMerkleProof(
                "root hash mismatch".to_owned(),
            ))
        }
    }

    /// Verify an inclusion proof against a known root hash, starting from a
    /// pre-computed leaf hash. Used by verifiers that hold the leaf hash
    /// (e.g. a transparency-log inclusion proof) rather than the raw data.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::InvalidMerkleProof`] if the proof does not
    /// recompute to `root`.
    pub fn verify_inclusion_against_root_leaf(
        root: &Hash,
        index: usize,
        tree_size: usize,
        leaf: &Hash,
        proof: &[Hash],
    ) -> Result<(), CryptoError> {
        if index >= tree_size {
            return Err(CryptoError::InvalidMerkleProof(format!(
                "index {index} out of range"
            )));
        }
        let computed = recompute_root_from_leaf(index, tree_size, *leaf, proof)?;
        if crate::constant_time::ct_eq(&computed, root) {
            Ok(())
        } else {
            Err(CryptoError::InvalidMerkleProof(
                "root hash mismatch".to_owned(),
            ))
        }
    }

    /// Generate a consistency proof between a tree of `old_size` leaves and
    /// this tree.
    ///
    /// Returns the list of hashes that prove the old tree is a prefix of the
    /// current tree.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::InvalidMerkleProof`] if `old_size > self.size`.
    ///
    /// # Example
    ///
    /// ```
    /// use eel_crypto_rs::merkle::MerkleTree;
    /// let old_leaves: Vec<&[u8]> = vec![b"a", b"b"];
    /// let new_leaves: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
    /// let new_tree = MerkleTree::build(&new_leaves);
    /// let old_tree = MerkleTree::build(&old_leaves);
    /// let proof = new_tree.consistency_proof(old_tree.size()).unwrap();
    /// assert!(new_tree.verify_consistency(old_tree.root(), old_tree.size(), &proof).is_ok());
    /// ```
    pub fn consistency_proof(&self, old_size: usize) -> Result<Vec<Hash>, CryptoError> {
        if old_size > self.size {
            return Err(CryptoError::InvalidMerkleProof(format!(
                "old_size {old_size} > current size {}",
                self.size
            )));
        }
        if old_size == 0 || old_size == self.size {
            return Ok(vec![]);
        }
        Ok(compute_consistency_proof(&self.leaves, old_size, self.size))
    }

    /// Verify a consistency proof between `old_root` (of `old_size` leaves)
    /// and this tree's root.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::InvalidMerkleProof`] if verification fails.
    pub fn verify_consistency(
        &self,
        old_root: Hash,
        old_size: usize,
        proof: &[Hash],
    ) -> Result<(), CryptoError> {
        if old_size == 0 {
            return Ok(());
        }
        if old_size == self.size {
            if crate::constant_time::ct_eq(&old_root, &self.root()) {
                return Ok(());
            }
            return Err(CryptoError::InvalidMerkleProof(
                "old root must equal new root when sizes match".to_owned(),
            ));
        }
        verify_consistency_proof(&old_root, old_size, &self.root(), self.size, proof)
    }
}

// ---------------------------------------------------------------------------
// Internal tree building
// ---------------------------------------------------------------------------

fn build_tree(leaves: &[Hash]) -> Vec<Hash> {
    if leaves.is_empty() {
        return vec![];
    }
    // Store as a flat array: nodes at depth d start at index 2^d - 1.
    // Use a simpler approach: build level by level bottom-up.
    let mut current = leaves.to_vec();
    let mut all_levels: Vec<Vec<Hash>> = vec![current.clone()];

    while current.len() > 1 {
        let mut next = Vec::with_capacity((current.len() + 1) / 2);
        let mut i = 0;
        while i < current.len() {
            if i + 1 < current.len() {
                next.push(node_hash(&current[i], &current[i + 1]));
            } else {
                // Odd leaf: promote without hashing (per RFC 6962).
                next.push(current[i]);
            }
            i += 2;
        }
        all_levels.push(next.clone());
        current = next;
    }

    // Root is in the last level.  Build a lookup: level -> hashes.
    // We don't store the full tree structure for now; root is levels.last()[0].
    // For proof generation we use the recursive approach below.
    let _ = all_levels;
    current // only root
}

/// Compute inclusion proof (sibling hashes, leaf to root order).
fn compute_inclusion_proof(leaves: &[Hash], index: usize) -> Vec<Hash> {
    let mut proof = Vec::new();
    rfc6962_path(leaves, index, leaves.len(), &mut proof);
    proof
}

fn rfc6962_path(leaves: &[Hash], index: usize, n: usize, proof: &mut Vec<Hash>) {
    if n <= 1 {
        return;
    }
    let k = largest_power_of_two_less_than(n);
    if index < k {
        // Target is in the left subtree.  Recurse first so the leaf-level
        // sibling is first in the proof (leaf-to-root order), matching
        // what recompute_root expects.
        rfc6962_path(&leaves[..k], index, k, proof);
        proof.push(merkle_root(&leaves[k..n]));
    } else {
        // Target is in the right subtree.
        rfc6962_path(&leaves[k..n], index - k, n - k, proof);
        proof.push(merkle_root(&leaves[..k]));
    }
}

/// Recompute the root from an inclusion proof.
///
/// The proof is in leaf-to-root order (as generated by `rfc6962_path`).
/// We reconstruct the path structure top-down, then replay it bottom-up
/// so each proof element is consumed at the correct tree level.
fn recompute_root(
    index: usize,
    tree_size: usize,
    data: &[u8],
    proof: &[Hash],
) -> Result<Hash, CryptoError> {
    recompute_root_from_leaf(index, tree_size, leaf_hash(data), proof)
}

/// Recompute the root from an inclusion proof, starting from a pre-computed
/// leaf hash (used by verifiers that hold the leaf hash, not the raw data).
fn recompute_root_from_leaf(
    index: usize,
    tree_size: usize,
    leaf: Hash,
    proof: &[Hash],
) -> Result<Hash, CryptoError> {
    let mut current = leaf;

    // Build the sequence of (subtree_size, is_right_child) from root down to leaf.
    // is_right_child == true means the target sits in the RIGHT subtree at that level.
    let mut path: Vec<(usize, bool)> = Vec::new();
    {
        let mut sz = tree_size;
        let mut idx = index;
        while sz > 1 {
            let k = largest_power_of_two_less_than(sz);
            if idx < k {
                path.push((sz, false));
                sz = k;
            } else {
                path.push((sz, true));
                idx -= k;
                sz -= k;
            }
        }
    }

    // The path is root-to-leaf; reverse it so we apply siblings leaf-to-root,
    // matching the proof order that `rfc6962_path` produces.
    let mut proof_iter = proof.iter();
    for &(_level_sz, is_right) in path.iter().rev() {
        let sibling = proof_iter
            .next()
            .ok_or_else(|| CryptoError::InvalidMerkleProof("proof too short".to_owned()))?;
        if is_right {
            // target is in right subtree; sibling is the left subtree hash
            current = node_hash(sibling, &current);
        } else {
            // target is in left subtree; sibling is the right subtree hash
            current = node_hash(&current, sibling);
        }
    }

    if proof_iter.next().is_some() {
        return Err(CryptoError::InvalidMerkleProof("proof too long".to_owned()));
    }
    Ok(current)
}

/// Compute the Merkle root of a slice of leaf hashes using RFC 6962 rules.
fn merkle_root(leaves: &[Hash]) -> Hash {
    match leaves.len() {
        0 => {
            let h = Sha256::new();
            h.finalize().into()
        }
        1 => leaves[0],
        n => {
            let k = largest_power_of_two_less_than(n);
            node_hash(&merkle_root(&leaves[..k]), &merkle_root(&leaves[k..]))
        }
    }
}

/// Largest power of two strictly less than `n` (used in RFC 6962 proofs).
fn largest_power_of_two_less_than(n: usize) -> usize {
    assert!(n > 1);
    let mut k = 1;
    while k * 2 < n {
        k *= 2;
    }
    k
}

// ---------------------------------------------------------------------------
// Consistency proofs
// ---------------------------------------------------------------------------

fn compute_consistency_proof(leaves: &[Hash], old_size: usize, new_size: usize) -> Vec<Hash> {
    let mut proof = Vec::new();
    rfc6962_consistency(&leaves[..new_size], old_size, new_size, true, &mut proof);
    proof
}

fn rfc6962_consistency(
    leaves: &[Hash],
    old: usize,
    new: usize,
    is_root: bool,
    proof: &mut Vec<Hash>,
) {
    if old == new {
        if !is_root {
            proof.push(merkle_root(leaves));
        }
        return;
    }
    let k = largest_power_of_two_less_than(new);
    if old <= k {
        rfc6962_consistency(&leaves[..k], old, k, is_root, proof);
        proof.push(merkle_root(&leaves[k..new]));
    } else {
        rfc6962_consistency(&leaves[k..new], old - k, new - k, false, proof);
        proof.push(merkle_root(&leaves[..k]));
    }
}

fn verify_consistency_proof(
    old_root: &Hash,
    old_size: usize,
    new_root: &Hash,
    new_size: usize,
    proof: &[Hash],
) -> Result<(), CryptoError> {
    let mut proof_iter = proof.iter();
    let (computed_old, computed_new) =
        reconstruct_consistency(old_size, new_size, true, old_root, &mut proof_iter)?;
    if proof_iter.next().is_some() {
        return Err(CryptoError::InvalidMerkleProof(
            "consistency proof too long".to_owned(),
        ));
    }
    if !crate::constant_time::ct_eq(&computed_old, old_root) {
        return Err(CryptoError::InvalidMerkleProof(
            "old root mismatch in consistency proof".to_owned(),
        ));
    }
    if !crate::constant_time::ct_eq(&computed_new, new_root) {
        return Err(CryptoError::InvalidMerkleProof(
            "new root mismatch in consistency proof".to_owned(),
        ));
    }
    Ok(())
}

/// Recursively reconstruct both the old and new Merkle roots from a
/// consistency proof using the same structural decomposition as
/// `rfc6962_consistency`.
///
/// `b = true` means this is the top-level call where `old_root` was not
/// pushed into the proof (it is known externally).  When `b = false`, the
/// hash for an equal old/new subtree is taken from the proof iterator.
///
/// Returns `(old_root_for_subtree, new_root_for_subtree)`.
fn reconstruct_consistency<'a>(
    old: usize,
    new: usize,
    b: bool,
    external_old_root: &Hash,
    proof: &mut impl Iterator<Item = &'a Hash>,
) -> Result<(Hash, Hash), CryptoError> {
    if old == new {
        if b {
            // This subtree IS the entire old tree.  Its hash was not pushed
            // into the proof; use the externally provided old_root.
            return Ok((*external_old_root, *external_old_root));
        }
        // b=false: this subtree's hash was pushed into the proof.
        let h = proof.next().ok_or_else(|| {
            CryptoError::InvalidMerkleProof("consistency proof too short".to_owned())
        })?;
        return Ok((*h, *h));
    }

    let k = largest_power_of_two_less_than(new);
    if old <= k {
        // Old tree fits entirely within the left k-leaf subtree.
        let (oh, nh) = reconstruct_consistency(old, k, b, external_old_root, proof)?;
        let right = proof.next().ok_or_else(|| {
            CryptoError::InvalidMerkleProof("consistency proof too short".to_owned())
        })?;
        Ok((oh, node_hash(&nh, right)))
    } else {
        // Old tree spans both subtrees; right portion's proof elements come first.
        let (oh, nh) = reconstruct_consistency(old - k, new - k, false, external_old_root, proof)?;
        let left = proof.next().ok_or_else(|| {
            CryptoError::InvalidMerkleProof("consistency proof too short".to_owned())
        })?;
        Ok((node_hash(left, &oh), node_hash(left, &nh)))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // --- known values --------------------------------------------------------

    #[test]
    fn single_leaf_root() {
        let entries: Vec<&[u8]> = vec![b"data"];
        let tree = MerkleTree::build(&entries);
        assert_eq!(tree.root(), leaf_hash(b"data"));
    }

    #[test]
    fn two_leaf_root() {
        let entries: Vec<&[u8]> = vec![b"a", b"b"];
        let tree = MerkleTree::build(&entries);
        let expected = node_hash(&leaf_hash(b"a"), &leaf_hash(b"b"));
        assert_eq!(tree.root(), expected);
    }

    // --- inclusion proof -----------------------------------------------------

    #[test]
    fn inclusion_proof_2_leaves() {
        let entries: Vec<&[u8]> = vec![b"a", b"b"];
        let tree = MerkleTree::build(&entries);

        let proof0 = tree.inclusion_proof(0).unwrap();
        tree.verify_inclusion(0, b"a", &proof0).unwrap();

        let proof1 = tree.inclusion_proof(1).unwrap();
        tree.verify_inclusion(1, b"b", &proof1).unwrap();
    }

    #[test]
    fn inclusion_proof_wrong_data_fails() {
        let entries: Vec<&[u8]> = vec![b"a", b"b", b"c"];
        let tree = MerkleTree::build(&entries);
        let proof = tree.inclusion_proof(0).unwrap();
        assert!(tree.verify_inclusion(0, b"wrong", &proof).is_err());
    }

    #[test]
    fn inclusion_proof_out_of_range() {
        let entries: Vec<&[u8]> = vec![b"a"];
        let tree = MerkleTree::build(&entries);
        assert!(tree.inclusion_proof(1).is_err());
    }

    // --- consistency proof ---------------------------------------------------

    #[test]
    fn consistency_proof_grow_from_2_to_4() {
        let all: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let old: Vec<&[u8]> = vec![b"a", b"b"];
        let new_tree = MerkleTree::build(&all);
        let old_tree = MerkleTree::build(&old);

        let proof = new_tree.consistency_proof(old_tree.size()).unwrap();
        new_tree
            .verify_consistency(old_tree.root(), old_tree.size(), &proof)
            .unwrap();
    }

    // --- proptest ------------------------------------------------------------

    proptest! {
        /// Inclusion proof verifies for every valid index.
        #[test]
        fn prop_inclusion_verifies(
            n in 1usize..=16,
            idx in 0usize..=15,
        ) {
            let entries: Vec<Vec<u8>> = (0u8..n as u8).map(|i| vec![i]).collect();
            let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
            let tree = MerkleTree::build(&refs);

            let idx = idx % n;
            let proof = tree.inclusion_proof(idx).unwrap();
            prop_assert!(
                tree.verify_inclusion(idx, &entries[idx], &proof).is_ok(),
                "inclusion proof failed for idx {idx} in tree of size {n}"
            );
        }

        /// Tampered data fails inclusion verification.
        #[test]
        fn prop_tampered_data_fails(n in 2usize..=8, idx in 0usize..=7) {
            let entries: Vec<Vec<u8>> = (0u8..n as u8).map(|i| vec![i]).collect();
            let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
            let tree = MerkleTree::build(&refs);
            let idx = idx % n;
            let proof = tree.inclusion_proof(idx).unwrap();
            let bad_data = vec![0xffu8; 4];
            if entries[idx] != bad_data {
                prop_assert!(
                    tree.verify_inclusion(idx, &bad_data, &proof).is_err(),
                    "tampered data should not verify"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Kani harnesses
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod kani_harnesses {
    use super::*;

    /// An authentic inclusion proof always verifies.
    #[kani::proof]
    #[kani::unwind(5)]
    fn merkle_inclusion_proof_verifies_authentic() {
        // Build a small tree with symbolic leaf data.
        let n: usize = kani::any_where(|n: &usize| *n >= 1 && *n <= 4);
        let mut entries: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let b: u8 = kani::any();
            entries.push(vec![b]);
        }
        let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
        let tree = MerkleTree::build(&refs);
        let idx: usize = kani::any_where(|i: &usize| *i < n);
        let proof = tree.inclusion_proof(idx).unwrap();
        assert!(tree.verify_inclusion(idx, &entries[idx], &proof).is_ok());
    }

    /// Consistency proof never forgets committed entries: the new tree's root
    /// must incorporate all leaves from the old tree.
    #[kani::proof]
    #[kani::unwind(5)]
    fn merkle_consistency_never_forgets() {
        let old_n: usize = kani::any_where(|n: &usize| *n >= 1 && *n <= 3);
        let extra: usize = kani::any_where(|n: &usize| *n >= 1 && *n <= 3);
        let new_n = old_n + extra;

        let mut entries: Vec<Vec<u8>> = Vec::new();
        for _ in 0..new_n {
            entries.push(vec![kani::any::<u8>()]);
        }

        let old_refs: Vec<&[u8]> = entries[..old_n].iter().map(Vec::as_slice).collect();
        let new_refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();

        let old_tree = MerkleTree::build(&old_refs);
        let new_tree = MerkleTree::build(&new_refs);

        let proof = new_tree.consistency_proof(old_n).unwrap();
        assert!(
            new_tree
                .verify_consistency(old_tree.root(), old_n, &proof)
                .is_ok(),
            "consistency proof must verify"
        );
    }
}
