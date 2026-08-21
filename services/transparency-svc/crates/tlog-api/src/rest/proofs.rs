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
    State(_state): State<ApiState>,
    Query(_q): Query<InclusionQuery>,
) -> Result<Json<InclusionProof>, StatusCode> {
    // TODO(WP-2.x): look up leaf by hash, compute proof via MerkleTree.
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn get_consistency_proof(
    State(_state): State<ApiState>,
    Query(q): Query<ConsistencyQuery>,
) -> Result<Json<ConsistencyProof>, StatusCode> {
    // TODO(WP-2.x): compute consistency proof between two STHs.
    Ok(Json(ConsistencyProof {
        old_size: q.from_size,
        new_size: q.to_size,
        old_root: String::new(),
        new_root: String::new(),
        hashes: vec![],
    }))
}
