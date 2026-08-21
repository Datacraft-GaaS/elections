use axum::{routing::get, Json, Router};
use chrono::Utc;
use serde::Serialize;

use crate::error::ApiError;

#[derive(Debug, Serialize)]
pub struct MerkleRootResponse {
    pub root_hash: String,
    pub seq: u64,
    pub election_scope: String,
    pub created_at: String,
}

pub fn router() -> Router {
    Router::new().route("/api/v1/bvr/merkle-root/latest", get(get_latest_merkle_root))
}

async fn get_latest_merkle_root() -> Result<Json<MerkleRootResponse>, ApiError> {
    Ok(Json(MerkleRootResponse {
        root_hash: "0".repeat(64),
        seq: 0,
        election_scope: "2027-general".into(),
        created_at: Utc::now().to_rfc3339(),
    }))
}
