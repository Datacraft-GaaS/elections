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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::state::AppState;

pub async fn get_assignments(
    state: State<AppState>,
    Path(worker_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    let Ok(store) = state.assignments.lock() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "store unavailable" })),
        );
    };

    let assignments: Vec<_> = store
        .iter()
        .filter(|a| a.worker_id == worker_id)
        .cloned()
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::to_value(assignments).unwrap_or_default()),
    )
}
