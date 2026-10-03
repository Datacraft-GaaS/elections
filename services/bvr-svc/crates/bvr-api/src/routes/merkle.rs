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

//! The latest published Merkle root.
//!
//! This returned `"0".repeat(64)` with `seq: 0`. That is the Merkle root of an
//! *empty* tree, presented as the root of the register -- and a Merkle root is
//! exactly the kind of value a client is meant to check a proof against, so a zero
//! root is a verification anchor that verifies nothing while looking authoritative.

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use crate::error::ApiError;

pub use crate::store::MerkleRootResponse;

pub fn router() -> Router<crate::state::AppState> {
    Router::new().route(
        "/api/v1/bvr/merkle-root/latest",
        get(get_latest_merkle_root),
    )
}

async fn get_latest_merkle_root(
    State(state): State<crate::state::AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let Some(root) = state
        .store
        .latest_merkle_root()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    else {
        // No root has been published. Saying so is the honest answer; a zero root
        // is not.
        return Err(ApiError::NotFound(
            "no Merkle root has been published for this election scope. This handler \
             previously returned 64 zeros with seq 0, which is the root of an empty tree \
             and reads as a verification anchor that verifies nothing"
                .to_string(),
        ));
    };
    if root.signature.is_none() {
        // The contract requires a signature over (root_hash || tree_size ||
        // timestamp). `merkle_roots.signed_by_hsm` records the signing key, and no
        // HSM has signed in this deployment. The root is real and is returned
        // below, but a root presented as a verification anchor without a signature
        // is not one, so the endpoint says so rather than inventing a signature.
        return Ok((
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "the latest root is unsigned: no HSM has signed it, and the \
                          contract requires a signature over (root_hash || tree_size || \
                          timestamp) for a root to be a verification anchor",
                "root": root,
            })),
        )
            .into_response());
    }
    Ok(Json(root).into_response())
}
