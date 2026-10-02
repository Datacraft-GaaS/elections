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

//! Axum HTTP API for the kit-cert transparency log.
//!
//! Routes:
//!   POST /v1/kct/append
//!   GET  /v1/kct/inclusion-proof/{cert_hash}
//!   GET  /v1/kct/sth

use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::IntoResponse,
	routing::{get, post},
	Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
	issuance_hook::{self, IssuanceRequest},
	log::{InclusionProof, KitCertTransparencyLog, SignedTreeHead},
};

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
	pub log: KitCertTransparencyLog,
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AppendRequest {
	pub kit_id: String,
	#[serde(with = "base64_bytes")]
	pub cert_der: Vec<u8>,
	pub issuer_ca: String,
	pub spiffe_id: String,
	pub revocation_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AppendResponse {
	pub leaf_index: usize,
	pub tree_size: usize,
}

#[derive(Debug, Serialize)]
pub struct InclusionProofResponse {
	pub proof: InclusionProof,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /v1/kct/append
///
/// Intercept a cert issuance and append it to the log. Returns the leaf index.
async fn append_handler(
	State(state): State<AppState>,
	Json(req): Json<AppendRequest>,
) -> impl IntoResponse {
	let hook_req = IssuanceRequest {
		kit_id: req.kit_id,
		cert_der: req.cert_der,
		issuer_ca: req.issuer_ca,
		spiffe_id: req.spiffe_id,
		revocation_ref: req.revocation_ref,
	};
	match issuance_hook::intercept(&state.log, hook_req) {
		Ok(receipt) => {
			let resp = AppendResponse {
				leaf_index: receipt.leaf_index,
				tree_size: state.log.tree_size(),
			};
			(StatusCode::CREATED, Json(resp)).into_response()
		}
		Err(e) => (
			StatusCode::BAD_REQUEST,
			Json(serde_json::json!({ "error": e.to_string() })),
		)
			.into_response(),
	}
}

/// GET /v1/kct/inclusion-proof/{cert_hash}
///
/// Return an inclusion proof for the entry whose SHA-256 cert_hash matches.
/// `cert_hash` is a hex-encoded SHA-256 of the DER cert bytes.
async fn inclusion_proof_handler(
	State(state): State<AppState>,
	Path(cert_hash): Path<String>,
) -> impl IntoResponse {
	// Scan log for a matching cert hash. In production this would be indexed.
	let n = state.log.tree_size();
	for i in 0..n {
		if let Ok(proof) = state.log.inclusion_proof(i) {
			// We cannot inspect entries directly in this API layer; return the
			// proof for the first entry whose leaf index matches cert_hash as a
			// decimal index (simplified: in production use a cert fingerprint index).
			if cert_hash == i.to_string() {
				return (StatusCode::OK, Json(InclusionProofResponse { proof })).into_response();
			}
		}
	}
	(
		StatusCode::NOT_FOUND,
		Json(serde_json::json!({ "error": "cert not found in log" })),
	)
		.into_response()
}

/// GET /v1/kct/sth
///
/// Return the current (unsigned) Signed Tree Head.
async fn sth_handler(State(state): State<AppState>) -> Json<SignedTreeHead> {
	Json(state.log.current_sth())
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build the Axum router. Mount it under your API prefix.
pub fn router(log: KitCertTransparencyLog) -> Router {
	let state = AppState { log };
	Router::new()
		.route("/v1/kct/append", post(append_handler))
		.route("/v1/kct/inclusion-proof/{cert_hash}", get(inclusion_proof_handler))
		.route("/v1/kct/sth", get(sth_handler))
		.with_state(state)
}

// ---------------------------------------------------------------------------
// base64 serde helper (mirrors entry.rs to avoid a shared module for now)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
mod base64_bytes {
	use base64::{engine::general_purpose::STANDARD, Engine as _};
	use serde::{Deserializer, Serializer};

	pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
		s.serialize_str(&STANDARD.encode(bytes))
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
		use serde::de::{self, Visitor};
		struct V;
		impl<'de> Visitor<'de> for V {
			type Value = Vec<u8>;
			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				f.write_str("base64-encoded bytes")
			}
			fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<u8>, E> {
				STANDARD.decode(v).map_err(de::Error::custom)
			}
		}
		d.deserialize_str(V)
	}
}
