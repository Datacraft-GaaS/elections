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

//! Admin HTTP routes for delivery management.
//!
//! Routes:
//!   GET  /v1/reports/deliveries?run_id=<uuid>          list deliveries for a run
//!   GET  /v1/reports/deliveries/{id}                   fetch one delivery
//!   POST /v1/reports/deliveries/{id}/retry             reset to pending
//!   POST /v1/reports/deliveries/{id}/cancel            abandon

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::repository;

pub type AppState = Arc<PgPool>;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/reports/deliveries", get(list_deliveries))
        .route("/v1/reports/deliveries/{id}", get(get_delivery))
        .route("/v1/reports/deliveries/{id}/retry", post(retry_delivery))
        .route("/v1/reports/deliveries/{id}/cancel", post(cancel_delivery))
}

#[derive(Deserialize)]
struct ListQuery {
    run_id: Uuid,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    50
}

async fn list_deliveries(
    State(pool): State<AppState>,
    Query(q): Query<ListQuery>,
) -> impl IntoResponse {
    match repository::list_by_run(&pool, q.run_id, q.limit, q.offset).await {
        Ok(rows) => (
            StatusCode::OK,
            Json(serde_json::json!({ "deliveries": rows })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_delivery(State(pool): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match repository::fetch_delivery(&pool, id).await {
        Ok(Some(d)) => (StatusCode::OK, Json(d)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn retry_delivery(State(pool): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match repository::retry(&pool, id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "pending" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn cancel_delivery(State(pool): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match repository::cancel(&pool, id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "abandoned" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
