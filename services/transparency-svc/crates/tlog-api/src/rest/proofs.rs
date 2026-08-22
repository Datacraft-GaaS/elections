use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
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
    let hashes: Vec<Vec<u8>> = sqlx::query_scalar("SELECT hash FROM leaves ORDER BY index")
        .fetch_all(&*state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let entries: Vec<&[u8]> = hashes.iter().map(|h| h.as_slice()).collect();
    let tree = eel_crypto_rs::merkle::MerkleTree::build(&entries);
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
    let hashes: Vec<Vec<u8>> = sqlx::query_scalar("SELECT hash FROM leaves ORDER BY index")
        .fetch_all(&*state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tree_size = hashes.len() as u64;
    if q.to_size > tree_size {
        return Err(StatusCode::BAD_REQUEST);
    }
    if q.from_size == 0 {
        // Empty prefix proves nothing; RFC 6962 defines the empty tree as its
        // own prefix, so return the current root with no hashes.
        let entries: Vec<&[u8]> = hashes.iter().map(|h| h.as_slice()).collect();
        let tree = eel_crypto_rs::merkle::MerkleTree::build(&entries);
        return Ok(Json(ConsistencyProof {
            old_size: 0,
            new_size: tree_size,
            old_root: String::new(),
            new_root: hex::encode(tree.root()),
            hashes: vec![],
        }));
    }
    let entries: Vec<&[u8]> = hashes.iter().map(|h| h.as_slice()).collect();
    let tree = eel_crypto_rs::merkle::MerkleTree::build(&entries);
    let old_entries: Vec<&[u8]> = hashes[..q.from_size as usize]
        .iter()
        .map(|h| h.as_slice())
        .collect();
    let old_tree = eel_crypto_rs::merkle::MerkleTree::build(&old_entries);
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
