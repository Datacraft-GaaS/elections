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
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use tracing::info;

use crate::models::SignatureFusionInput;
use crate::pipeline::fuse_snapshot;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"ok": true})))
}

pub async fn post_snapshot(
    Path(station_code): Path<String>,
    Json(input): Json<SignatureFusionInput>,
) -> impl IntoResponse {
    info!(station_code, "post_snapshot");
    let snapshot = fuse_snapshot(&station_code, input);
    (StatusCode::OK, Json(snapshot))
}
