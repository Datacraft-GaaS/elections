use axum::{extract::Path, routing::get, Json, Router};
use bvr_batch::export::{run_export, ExportConfig};
use serde::Serialize;

use crate::error::ApiError;

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub snapshot_id: String,
    pub election_scope: String,
    pub voter_count: u64,
    pub merkle_root: String,
}

pub fn router() -> Router {
    Router::new().route("/api/v1/bvr/registers/{scope}", get(get_register))
}

async fn get_register(Path(scope): Path<String>) -> Result<Json<RegisterResponse>, ApiError> {
    let snap = run_export(ExportConfig {
        election_scope: scope,
        resume_from: None,
        batch_size: 10_000,
    })
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(RegisterResponse {
        snapshot_id: snap.snapshot_id,
        election_scope: snap.election_scope,
        voter_count: snap.voter_count,
        merkle_root: snap.merkle_root,
    }))
}
