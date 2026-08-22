//! Axum 0.8 health and query routes for bft-tally.
//!
//! Routes:
//!   GET /v1/bft/health             - liveness + current view
//!   GET /v1/bft/blocks/{height}    - retrieve committed block by height
//!   GET /v1/bft/qc/{block_hash}    - retrieve QC for a committed block

use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::IntoResponse,
	routing::get,
	Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::certificate::QuorumCertificate;
use crate::persistence::InMemoryStore;

#[derive(Clone)]
pub struct BftApiState {
	pub store: Arc<RwLock<InMemoryStore>>,
	pub current_view: Arc<RwLock<u64>>,
	/// Quorum certificates keyed by hex block hash.
	pub qcs: Arc<RwLock<std::collections::HashMap<String, QuorumCertificate>>>,
}

impl BftApiState {
	pub fn new(replica_id: u64) -> Self {
		Self {
			store: Arc::new(RwLock::new(InMemoryStore::new(replica_id))),
			current_view: Arc::new(RwLock::new(0)),
			qcs: Arc::new(RwLock::new(std::collections::HashMap::new())),
		}
	}
}

#[derive(Serialize)]
struct HealthResponse {
	status: &'static str,
	view: u64,
}

#[derive(Serialize)]
struct BlockResponse {
	height: u64,
	hash: String,
	parent_hash: String,
	payload_bytes: usize,
}

async fn health_handler(State(s): State<BftApiState>) -> impl IntoResponse {
	let view = *s.current_view.read().await;
	(StatusCode::OK, Json(HealthResponse { status: "ok", view }))
}

async fn block_handler(
	State(s): State<BftApiState>,
	Path(height): Path<u64>,
) -> impl IntoResponse {
	let store = s.store.read().await;
	match store.get_block(height) {
		Some(b) => {
			let resp = BlockResponse {
				height: b.height,
				hash: hex::encode(b.hash),
				parent_hash: hex::encode(b.parent_hash),
				payload_bytes: b.payload.len(),
			};
			(StatusCode::OK, Json(serde_json::to_value(resp).unwrap_or_default()))
		}
		None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "block not found"}))),
	}
}

async fn qc_handler(
	State(s): State<BftApiState>,
	Path(block_hash): Path<String>,
) -> impl IntoResponse {
	let qcs = s.qcs.read().await;
	match qcs.get(&block_hash) {
		Some(qc) => match serde_json::to_value(qc) {
			Ok(v) => (StatusCode::OK, Json(v)),
			Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "serialization error"}))),
		},
		None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "qc not found"}))),
	}
}

pub fn router(state: BftApiState) -> Router {
	Router::new()
		.route("/v1/bft/health", get(health_handler))
		.route("/v1/bft/blocks/{height}", get(block_handler))
		.route("/v1/bft/qc/{block_hash}", get(qc_handler))
		.with_state(state)
}
