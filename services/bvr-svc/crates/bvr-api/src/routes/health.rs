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

use axum::{routing::get, Json, Router};
use serde_json::json;

// These handlers need no state, but the router is typed with `AppState` so it
// can be merged into the composed router alongside the stateful sub-routers.
pub fn router() -> Router<crate::state::AppState> {
    Router::<crate::state::AppState>::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({"ok": true}))
}

async fn readyz() -> Json<serde_json::Value> {
    Json(json!({"ready": true}))
}
