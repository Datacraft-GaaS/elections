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

//! Merkle proof generation over the stored leaf set.
//!
//! The leaves table stores domain-defined leaf hashes (see
//! `tlog_domain::LeafFormat::hash`). Proofs are computed with
//! [`eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes`] so that a proof
//! verifies via `tlog_domain::InclusionProof::verify`, which starts from the
//! stored leaf hash directly.

use sqlx::PgPool;
use tlog_domain::{ConsistencyProof, InclusionProof};

/// Load all stored leaf hashes in index order.
pub async fn load_leaf_hashes(pool: &PgPool) -> Result<Vec<[u8; 32]>, sqlx::Error> {
    let hashes: Vec<Vec<u8>> = sqlx::query_scalar("SELECT hash FROM leaves ORDER BY index")
        .fetch_all(pool)
        .await?;
    Ok(hashes
        .into_iter()
        .map(|h| <[u8; 32]>::try_from(h.as_slice()).expect("leaf hash is 32 bytes"))
        .collect())
}

/// Compute the current Merkle root over all stored leaves.
///
/// The empty tree hashes to SHA256("") per RFC 6962.
pub async fn current_root(pool: &PgPool) -> Result<[u8; 32], sqlx::Error> {
    let hashes = load_leaf_hashes(pool).await?;
    Ok(eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(hashes).root())
}

/// Build an inclusion proof for the leaf with the given hash.
///
/// Returns `Ok(None)` if no leaf with that hash exists.
pub async fn inclusion_proof(
    pool: &PgPool,
    leaf_hash: &[u8],
) -> Result<Option<InclusionProof>, sqlx::Error> {
    let index: Option<i64> = sqlx::query_scalar("SELECT index FROM leaves WHERE hash = $1")
        .bind(leaf_hash)
        .fetch_optional(pool)
        .await?;
    let Some(index) = index else {
        return Ok(None);
    };

    let hashes = load_leaf_hashes(pool).await?;
    let tree_size = hashes.len();
    if index as usize >= tree_size {
        return Ok(None);
    }
    let tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(hashes);
    let path = tree
        .inclusion_proof(index as usize)
        .map_err(proof_error)?;

    Ok(Some(InclusionProof {
        leaf_index: index as u64,
        tree_size: tree_size as u64,
        root_hash: hex::encode(tree.root()),
        hashes: path.iter().map(|h| hex::encode(h)).collect(),
    }))
}

/// Build an RFC 6962 consistency proof between `old_size` and the current
/// tree size.
///
/// Returns `Ok(None)` if `old_size` is zero or exceeds the current size.
pub async fn consistency_proof(
    pool: &PgPool,
    old_size: u64,
) -> Result<Option<ConsistencyProof>, sqlx::Error> {
    let hashes = load_leaf_hashes(pool).await?;
    let tree_size = hashes.len() as u64;
    if old_size == 0 || old_size > tree_size {
        return Ok(None);
    }

    let tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(hashes.clone());
    let old_tree = eel_crypto_rs::merkle::MerkleTree::from_leaf_hashes(
        hashes[..old_size as usize].to_vec(),
    );
    let path = tree.consistency_proof(old_size as usize).map_err(proof_error)?;

    Ok(Some(ConsistencyProof {
        old_size,
        new_size: tree_size,
        old_root: hex::encode(old_tree.root()),
        new_root: hex::encode(tree.root()),
        hashes: path.iter().map(|h| hex::encode(h)).collect(),
    }))
}

fn proof_error(e: eel_crypto_rs::CryptoError) -> sqlx::Error {
    sqlx::Error::Configuration(Box::new(std::io::Error::other(e.to_string())))
}
