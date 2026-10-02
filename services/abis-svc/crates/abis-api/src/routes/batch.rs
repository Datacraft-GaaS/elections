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

//! Batch dedup route handler.

use std::sync::Arc;

use axum::{extract::State, Json};

use crate::error::ApiError;
use crate::models::BatchDedupRequest;
use crate::state::AppState;

pub async fn submit_batch_dedup(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<BatchDedupRequest>,
) -> ApiError {
    // Previously returned 202 with a fresh job id and status "queued". Nothing
    // was ever queued, so the job id resolved to nothing and the caller waited
    // on work that would never run.
    ApiError::NotImplemented(
        "batch deduplication is declared by the abis contract but has no queue \
         or dedup engine behind it; it previously returned 202 with a job id \
         for a job that was never enqueued"
            .to_string(),
    )
}
