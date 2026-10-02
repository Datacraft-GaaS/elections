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

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tlog_domain::{ConsistencyProof, InclusionProof};

use crate::state::ApiState;

#[derive(Deserialize)]
pub struct InclusionQuery {
    pub leaf_hash: String,
}

#[derive(Deserialize)]
pub struct ConsistencyQuery {
    pub from_size: u64,
    pub to_size: u64,
}

pub async fn get_inclusion_proof(
    State(state): State<ApiState>,
    Query(q): Query<InclusionQuery>,
) -> Result<Json<InclusionProof>, StatusCode> {
    // Decode the requested leaf hash (BLAKE3, 32 bytes).
    let leaf_hash = hex::decode(&q.leaf_hash).map_err(|_| StatusCode::BAD_REQUEST)?;
    if leaf_hash.len() != 32 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Locate the leaf by hash to resolve its index.
    let index: Option<i64> = sqlx::query_scalar("SELECT index FROM leaves WHERE hash = $1")
        .bind(&leaf_hash)
        .fetch_optional(&*state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(index) = index else {
        return Err(StatusCode::NOT_FOUND);
    };

    // Rebuild the tree from all leaf hashes (index order) and compute the proof.
    // Leaves must be the stored domain hashes themselves, NOT re-hashed, so
    // that tlog_domain::InclusionProof::verify reproduces the root.
    let hashes = tlog_sequencer::proofs::load_leaf_hashes(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(hashes);
    let proof =
        tree.inclusion_proof(index as usize)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(InclusionProof {
        leaf_index: index as u64,
        tree_size: tree.size() as u64,
        root_hash: hex::encode(tree.root()),
        hashes: proof.iter().map(hex::encode).collect(),
    }))
}

pub async fn get_consistency_proof(
    State(state): State<ApiState>,
    Query(q): Query<ConsistencyQuery>,
) -> Result<Json<ConsistencyProof>, StatusCode> {
    // Compute an RFC 6962 consistency proof between two tree sizes.
    if q.from_size > q.to_size {
        return Err(StatusCode::BAD_REQUEST);
    }
    let hashes = tlog_sequencer::proofs::load_leaf_hashes(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tree_size = hashes.len() as u64;
    if q.to_size > tree_size {
        return Err(StatusCode::BAD_REQUEST);
    }
    if q.from_size == 0 {
        // Empty prefix proves nothing; RFC 6962 defines the empty tree as its
        // own prefix, so return the current root with no hashes.
        let tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(hashes);
        return Ok(Json(ConsistencyProof {
            old_size: 0,
            new_size: tree_size,
            old_root: String::new(),
            new_root: hex::encode(tree.root()),
            hashes: vec![],
        }));
    }
    let tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(hashes.clone());
    let old_tree =
        eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(hashes[..q.from_size as usize].to_vec());
    let proof = tree
        .consistency_proof(q.from_size as usize)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ConsistencyProof {
        old_size: q.from_size,
        new_size: tree_size,
        old_root: hex::encode(old_tree.root()),
        new_root: hex::encode(tree.root()),
        hashes: proof.iter().map(hex::encode).collect(),
    }))
}

/// Query parameters for receipt resolution. The station identifier is
/// accepted for auditability but resolution is by code prefix alone.
#[derive(Deserialize)]
pub struct ReceiptQuery {
    pub station: Option<String>,
}

/// Latest signed tree head, embedded so a client can verify the proof
/// offline in one round trip.
#[derive(Serialize)]
pub struct ReceiptSth {
    pub tree_size: u64,
    pub root_hash: String,
    pub cadence: String,
    pub signed_at: String,
    pub signature_hex: String,
}

#[derive(Serialize)]
pub struct ReceiptResponse {
    pub leaf_index: u64,
    pub tree_size: u64,
    pub root_hash: String,
    pub hashes: Vec<String>,
    pub leaf_hash_hex: String,
    pub sth: Option<ReceiptSth>,
}

/// Outcome of resolving a receipt code against the ordered leaf set.
#[derive(Debug, PartialEq, Eq)]
pub enum PrefixResolution {
    /// Exactly one leaf matches; carries its index.
    Unique(usize),
    /// No leaf matches the prefix.
    NotFound,
    /// More than one leaf shares the prefix.
    Ambiguous,
}

/// Resolve a receipt code (hex string) to a leaf index by byte-prefix match
/// over the stored hashes in index order.
///
/// Pure so it is unit-testable without a database.
pub fn resolve_prefix(hashes: &[[u8; 32]], code: &str) -> PrefixResolution {
    let bytes = match hex::decode(code.to_lowercase()) {
        Ok(b) => b,
        Err(_) => return PrefixResolution::NotFound,
    };
    let mut found: Option<usize> = None;
    for (i, h) in hashes.iter().enumerate() {
        if h.starts_with(&bytes) {
            if found.is_some() {
                return PrefixResolution::Ambiguous;
            }
            found = Some(i);
        }
    }
    match found {
        Some(i) => PrefixResolution::Unique(i),
        None => PrefixResolution::NotFound,
    }
}

pub async fn resolve_receipt(
    State(state): State<ApiState>,
    Path(code): Path<String>,
    Query(_q): Query<ReceiptQuery>,
) -> Result<Json<ReceiptResponse>, StatusCode> {
    // Codes are 8..=16 hex characters as printed on the posted Form 34A.
    if !(8..=16).contains(&code.len()) || !code.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let hashes = tlog_sequencer::proofs::load_leaf_hashes(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let index = match resolve_prefix(&hashes, &code) {
        PrefixResolution::Unique(i) => i,
        PrefixResolution::NotFound => return Err(StatusCode::NOT_FOUND),
        PrefixResolution::Ambiguous => return Err(StatusCode::CONFLICT),
    };

    let tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(hashes.clone());
    let proof = tree
        .inclusion_proof(index)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sth_row: Option<(i64, Vec<u8>, String, chrono::DateTime<chrono::Utc>, Vec<u8>)> =
        sqlx::query_as(
            "SELECT tree_size, root_hash, cadence, signed_at, signature \
             FROM sth ORDER BY tree_size DESC LIMIT 1",
        )
        .fetch_optional(&*state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ReceiptResponse {
        leaf_index: index as u64,
        tree_size: tree.size() as u64,
        root_hash: hex::encode(tree.root()),
        hashes: proof.iter().map(hex::encode).collect(),
        leaf_hash_hex: hex::encode(hashes[index]),
        sth: sth_row.map(|(tree_size, root_hash, cadence, signed_at, sig)| ReceiptSth {
            tree_size: tree_size as u64,
            root_hash: hex::encode(root_hash),
            cadence,
            signed_at: signed_at.to_rfc3339(),
            signature_hex: hex::encode(sig),
        }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(first: &str) -> [u8; 32] {
        let mut full = String::with_capacity(64);
        while full.len() < 64 {
            full.push_str(first);
        }
        hex::decode(&full[..64])
            .ok()
            .and_then(|v| <[u8; 32]>::try_from(v.as_slice()).ok())
            .expect("test hash is 32 bytes")
    }

    #[test]
    fn unique_prefix_resolves_to_leaf_index() {
        let hashes = vec![h("aa"), h("ab"), h("ac")];
        assert_eq!(resolve_prefix(&hashes, "AA"), PrefixResolution::Unique(0));
        assert_eq!(resolve_prefix(&hashes, "ab"), PrefixResolution::Unique(1));
        assert_eq!(resolve_prefix(&hashes, "acacacac"), PrefixResolution::Unique(2));
    }

    #[test]
    fn missing_prefix_is_not_found() {
        let hashes = vec![h("aa"), h("ab")];
        assert_eq!(resolve_prefix(&hashes, "ff"), PrefixResolution::NotFound);
        assert_eq!(
            resolve_prefix(&hashes, "zz"),
            PrefixResolution::NotFound,
            "non-hex codes resolve to not-found"
        );
    }

    #[test]
    fn shared_prefix_is_ambiguous() {
        let mut x = [0u8; 32];
        let mut y = [0u8; 32];
        x[..4].copy_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd]);
        y[..4].copy_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd]);
        y[31] = 0xff;
        let hashes = vec![h("ff"), x, y];
        assert_eq!(
            resolve_prefix(&hashes, "aabbcc"),
            PrefixResolution::Ambiguous
        );
        assert_eq!(
            resolve_prefix(&hashes, "AABBCCDD"),
            PrefixResolution::Ambiguous,
            "ambiguity holds for any prefix shared by multiple leaves"
        );
    }
}
