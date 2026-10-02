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

//! eel verifier: RFC 6962 Merkle inclusion/consistency, STH signature, witness
//! cosign, and bundle hash verification.
//!
//! Wraps `eel-crypto-rs` primitives and exposes them via a clean Rust API,
//! a C ABI (cdylib feature, for Dart FFI), and WASM exports (wasm feature).
//!
//! # Example
//!
//! ```no_run
//! use eel_verifier::{Sth, InclusionProof, verify_inclusion};
//! // Caller provides a populated Sth and InclusionProof from the transparency
//! // log at spiffe://eel.local.
//! ```
#![warn(missing_docs)]

use eel_crypto_rs::merkle::{leaf_hash, node_hash, Hash};
use eel_crypto_rs::rfc8785::canonicalize;
use eel_crypto_rs::{constant_time::ct_eq_fixed, ed25519};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// RFC 6962 Signed Tree Head.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Sth {
    pub tree_size: u64,
    pub timestamp: u64,
    /// Hex-encoded SHA-256 root hash (64 hex chars = 32 bytes).
    pub sha256_root_hash: String,
    /// Hex-encoded Ed25519 signature over the canonical STH message (128 hex chars).
    pub tree_head_signature: String,
    /// Hex-encoded Ed25519 verifying key (64 hex chars = 32 bytes).
    pub log_key: String,
}

/// RFC 6962 inclusion proof for a single leaf.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct InclusionProof {
    /// 0-indexed position of the leaf in the tree.
    pub leaf_index: u64,
    pub tree_size: u64,
    /// Sibling hashes from leaf to root, each 64 hex chars.
    pub hashes: Vec<String>,
}

/// RFC 6962 consistency proof between two tree sizes.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ConsistencyProof {
    pub old_size: u64,
    pub new_size: u64,
    /// Proof hashes, each 64 hex chars.
    pub hashes: Vec<String>,
}

/// An Ed25519 witness cosignature over an STH.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WitnessSig {
    /// Hex-encoded Ed25519 verifying key (64 hex chars = 32 bytes).
    pub key: String,
    /// Hex-encoded Ed25519 signature (128 hex chars = 64 bytes).
    pub signature: String,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from eel verifier operations.
#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("hex decode: {0}")]
    HexDecode(#[from] hex::FromHexError),
    #[error("invalid proof: {0}")]
    InvalidProof(String),
    #[error("signature invalid: {0}")]
    Signature(String),
    #[error("bundle hash mismatch")]
    HashMismatch,
    #[error("crypto: {0}")]
    Crypto(#[from] eel_crypto_rs::CryptoError),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn decode_hash(hex_str: &str) -> Result<Hash, VerifierError> {
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 32 {
        return Err(VerifierError::InvalidInput(format!(
            "expected 32-byte hash, got {} bytes",
            bytes.len()
        )));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Largest power of two strictly less than `n`.  Requires `n > 1`.
fn largest_pow2_lt(n: usize) -> usize {
    debug_assert!(n > 1);
    let mut k = 1usize;
    while k * 2 < n {
        k *= 2;
    }
    k
}

/// Reconstruct the Merkle root from a leaf-to-root inclusion proof.
fn recompute_inclusion_root(
    index: usize,
    tree_size: usize,
    leaf_data: &[u8],
    proof: &[Hash],
) -> Result<Hash, VerifierError> {
    if tree_size == 0 {
        return Err(VerifierError::InvalidProof("tree_size is 0".into()));
    }
    let mut current = leaf_hash(leaf_data);

    // Build the root-to-leaf direction sequence.
    // is_right == true: target is in the right subtree at this level.
    let mut path: Vec<bool> = Vec::new();
    let mut sz = tree_size;
    let mut idx = index;
    while sz > 1 {
        let k = largest_pow2_lt(sz);
        if idx < k {
            path.push(false);
            sz = k;
        } else {
            path.push(true);
            idx -= k;
            sz -= k;
        }
    }

    // Apply proof hashes bottom-up (leaf-to-root = path reversed).
    let mut proof_iter = proof.iter();
    for &is_right in path.iter().rev() {
        let sibling = proof_iter
            .next()
            .ok_or_else(|| VerifierError::InvalidProof("inclusion proof too short".into()))?;
        current = if is_right {
            node_hash(sibling, &current)
        } else {
            node_hash(&current, sibling)
        };
    }

    if proof_iter.next().is_some() {
        return Err(VerifierError::InvalidProof("inclusion proof too long".into()));
    }
    Ok(current)
}

/// Recursively reconstruct (old_root, new_root) from a consistency proof.
///
/// Mirrors the algorithm in eel-crypto-rs merkle.rs.
fn reconstruct_consistency<'a>(
    old: usize,
    new: usize,
    b: bool,
    external_old: &Hash,
    proof: &mut impl Iterator<Item = &'a Hash>,
) -> Result<(Hash, Hash), VerifierError> {
    if old == new {
        if b {
            return Ok((*external_old, *external_old));
        }
        let h = proof
            .next()
            .ok_or_else(|| VerifierError::InvalidProof("consistency proof too short".into()))?;
        return Ok((*h, *h));
    }
    let k = largest_pow2_lt(new);
    if old <= k {
        let (oh, nh) = reconstruct_consistency(old, k, b, external_old, proof)?;
        let right = proof
            .next()
            .ok_or_else(|| VerifierError::InvalidProof("consistency proof too short".into()))?;
        Ok((oh, node_hash(&nh, right)))
    } else {
        let (oh, nh) =
            reconstruct_consistency(old - k, new - k, false, external_old, proof)?;
        let left = proof
            .next()
            .ok_or_else(|| VerifierError::InvalidProof("consistency proof too short".into()))?;
        Ok((node_hash(left, &oh), node_hash(left, &nh)))
    }
}

/// Produce the canonical signing message for an STH.
///
/// RFC 8785 canonical JSON of {sha256_root_hash, timestamp, tree_size}.
fn sth_signing_message(sth: &Sth) -> Result<Vec<u8>, VerifierError> {
    let v = serde_json::json!({
        "sha256_root_hash": sth.sha256_root_hash,
        "timestamp": sth.timestamp,
        "tree_size": sth.tree_size,
    });
    canonicalize(&v).map_err(VerifierError::Crypto)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Verify that `leaf_data` is included at `proof.leaf_index` in the tree
/// described by `sth`.
///
/// The proof must have been generated for the same tree described by `sth`.
pub fn verify_inclusion(
    sth: &Sth,
    proof: &InclusionProof,
    leaf_data: &[u8],
) -> Result<(), VerifierError> {
    let expected_root = decode_hash(&sth.sha256_root_hash)?;
    let index = usize::try_from(proof.leaf_index)
        .map_err(|_| VerifierError::InvalidInput("leaf_index overflow".into()))?;
    let tree_size = usize::try_from(proof.tree_size)
        .map_err(|_| VerifierError::InvalidInput("tree_size overflow".into()))?;

    if index >= tree_size {
        return Err(VerifierError::InvalidProof(format!(
            "leaf_index {index} >= tree_size {tree_size}"
        )));
    }

    let hashes: Vec<Hash> = proof
        .hashes
        .iter()
        .map(|h| decode_hash(h))
        .collect::<Result<_, _>>()?;

    let computed = recompute_inclusion_root(index, tree_size, leaf_data, &hashes)?;
    if ct_eq_fixed(&computed, &expected_root) {
        Ok(())
    } else {
        Err(VerifierError::InvalidProof("root hash mismatch".into()))
    }
}

/// Verify that the tree in `to_sth` is a consistent extension of `from_sth`.
pub fn verify_consistency(
    from_sth: &Sth,
    to_sth: &Sth,
    proof: &ConsistencyProof,
) -> Result<(), VerifierError> {
    let old_root = decode_hash(&from_sth.sha256_root_hash)?;
    let new_root = decode_hash(&to_sth.sha256_root_hash)?;
    let old_size = usize::try_from(proof.old_size)
        .map_err(|_| VerifierError::InvalidInput("old_size overflow".into()))?;
    let new_size = usize::try_from(proof.new_size)
        .map_err(|_| VerifierError::InvalidInput("new_size overflow".into()))?;

    if old_size == 0 {
        return Ok(());
    }
    if old_size > new_size {
        return Err(VerifierError::InvalidProof(format!(
            "old_size {old_size} > new_size {new_size}"
        )));
    }
    if old_size == new_size {
        if proof.hashes.is_empty() && ct_eq_fixed(&old_root, &new_root) {
            return Ok(());
        }
        return Err(VerifierError::InvalidProof(
            "old root must equal new root when sizes match".into(),
        ));
    }

    let hashes: Vec<Hash> = proof
        .hashes
        .iter()
        .map(|h| decode_hash(h))
        .collect::<Result<_, _>>()?;

    let mut proof_iter = hashes.iter();
    let (computed_old, computed_new) =
        reconstruct_consistency(old_size, new_size, true, &old_root, &mut proof_iter)?;

    if proof_iter.next().is_some() {
        return Err(VerifierError::InvalidProof(
            "consistency proof too long".into(),
        ));
    }
    if !ct_eq_fixed(&computed_old, &old_root) {
        return Err(VerifierError::InvalidProof("old root mismatch".into()));
    }
    if !ct_eq_fixed(&computed_new, &new_root) {
        return Err(VerifierError::InvalidProof("new root mismatch".into()));
    }
    Ok(())
}

/// Verify the Ed25519 signature on an STH.
///
/// If `key_override` is `Some`, that hex-encoded key is used instead of
/// `sth.log_key`.
pub fn verify_sth_signature(sth: &Sth, key_override: Option<&str>) -> Result<(), VerifierError> {
    let key_hex = key_override.unwrap_or(&sth.log_key);
    let key_bytes: Vec<u8> = hex::decode(key_hex)?;
    if key_bytes.len() != 32 {
        return Err(VerifierError::InvalidInput(
            "verifying key must be 32 bytes".into(),
        ));
    }
    let key_arr: [u8; 32] = key_bytes.try_into().unwrap();
    let vk = ed25519::verifying_key_from_bytes(&key_arr)
        .map_err(|e| VerifierError::Signature(e.to_string()))?;

    let sig_bytes: Vec<u8> = hex::decode(&sth.tree_head_signature)?;
    if sig_bytes.len() != 64 {
        return Err(VerifierError::InvalidInput(
            "signature must be 64 bytes".into(),
        ));
    }
    let sig_arr: [u8; 64] = sig_bytes.try_into().unwrap();
    let sig = ed25519::signature_from_bytes(&sig_arr)
        .map_err(|e| VerifierError::Signature(e.to_string()))?;

    let msg = sth_signing_message(sth)?;
    ed25519::verify(&vk, &msg, &sig)
        .map_err(|e| VerifierError::Signature(e.to_string()))
}

/// Verify all witness cosignatures over `sth`.
///
/// All signatures must verify.  An empty slice is accepted (policy is
/// enforced by the caller).
pub fn verify_witness_cosign(sth: &Sth, witness_sigs: &[WitnessSig]) -> Result<(), VerifierError> {
    let msg = sth_signing_message(sth)?;
    for ws in witness_sigs {
        let key_bytes: Vec<u8> = hex::decode(&ws.key)?;
        if key_bytes.len() != 32 {
            return Err(VerifierError::InvalidInput(
                "witness key must be 32 bytes".into(),
            ));
        }
        let key_arr: [u8; 32] = key_bytes.try_into().unwrap();
        let vk = ed25519::verifying_key_from_bytes(&key_arr)
            .map_err(|e| VerifierError::Signature(e.to_string()))?;

        let sig_bytes: Vec<u8> = hex::decode(&ws.signature)?;
        if sig_bytes.len() != 64 {
            return Err(VerifierError::InvalidInput(
                "witness signature must be 64 bytes".into(),
            ));
        }
        let sig_arr: [u8; 64] = sig_bytes.try_into().unwrap();
        let sig = ed25519::signature_from_bytes(&sig_arr)
            .map_err(|e| VerifierError::Signature(e.to_string()))?;

        ed25519::verify(&vk, &msg, &sig)
            .map_err(|e| VerifierError::Signature(format!("witness {}: {e}", ws.key)))?;
    }
    Ok(())
}

/// Verify that BLAKE3(bundle_bytes) matches `expected_root_hex`.
pub fn verify_bundle_hash(
    bundle_bytes: &[u8],
    expected_root_hex: &str,
) -> Result<(), VerifierError> {
    let actual = eel_crypto_rs::blake3::hash(bundle_bytes);
    let expected_bytes = hex::decode(expected_root_hex)?;
    if expected_bytes.len() != 32 {
        return Err(VerifierError::InvalidInput(
            "expected_root_hex must encode 32 bytes".into(),
        ));
    }
    let expected_arr: [u8; 32] = expected_bytes.try_into().unwrap();
    if ct_eq_fixed(&actual, &expected_arr) {
        Ok(())
    } else {
        Err(VerifierError::HashMismatch)
    }
}

// ---------------------------------------------------------------------------
// Sub-modules
// ---------------------------------------------------------------------------

#[cfg(feature = "cdylib")]
pub mod ffi;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;
