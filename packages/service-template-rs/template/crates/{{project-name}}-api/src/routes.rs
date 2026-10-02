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
use serde_json::{json, Value};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(list_items), info(title = "{{project-name}} API", version = "0.1.0"))]
pub struct ApiDoc;

pub fn router() -> Router {
	Router::new().route("/api/v1/items", get(list_items))
}

/// List items.
#[utoipa::path(get, path = "/api/v1/items", responses((status = 200, description = "OK")))]
async fn list_items() -> Json<Value> {
	Json(json!({ "items": [] }))
}
