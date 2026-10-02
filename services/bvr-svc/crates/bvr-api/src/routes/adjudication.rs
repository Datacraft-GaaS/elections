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

use axum::{extract::Path, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct AdjudicationResolveRequest {
    /// "accept_primary" | "accept_secondary" | "reject_both"
    pub outcome: String,
    pub notes: Option<String>,
    pub resolved_by: String,
}

#[derive(Debug, Serialize)]
pub struct AdjudicationResolveResponse {
    pub case_id: String,
    pub outcome: String,
    pub status: String,
}

// These handlers need no state, but the router is typed with `AppState` so it
// can be merged into the composed router alongside the stateful sub-routers.
pub fn router() -> Router<crate::state::AppState> {
    Router::<crate::state::AppState>::new().route(
        "/api/v1/bvr/adjudication/{case_id}/resolve",
        post(resolve_adjudication),
    )
}

async fn resolve_adjudication(
    Path(case_id): Path<String>,
    Json(req): Json<AdjudicationResolveRequest>,
) -> Result<Json<AdjudicationResolveResponse>, ApiError> {
    Ok(Json(AdjudicationResolveResponse {
        case_id,
        outcome: req.outcome,
        status: "resolved".into(),
    }))
}
