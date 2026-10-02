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

//! Public contract handlers (diaspora-logistics/logistics.yaml).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};

use crate::api::SharedState;

fn page(items: &Value) -> Value {
    eel_spec_helpers_rs::page_value(items)
}

pub async fn list_embassies(State(s): State<SharedState>) -> impl IntoResponse {
    match s.diaspora_store.list_embassies().await {
        Ok(mut items) => {
            items.sort_by(|a, b| a["embassy_code"].as_str().cmp(&b["embassy_code"].as_str()));
            Json(page(&serde_json::to_value(items).unwrap())).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "list_embassies failed");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": "internal error"}))).into_response()
        }
    }
}

pub async fn get_embassy(State(s): State<SharedState>, Path(code): Path<String>) -> impl IntoResponse {
    match s.diaspora_store.get_embassy(&code).await {
        Ok(Some(v)) => Json(v).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "detail": "embassy not found" }))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "get_embassy failed");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": "internal error"}))).into_response()
        }
    }
}

pub async fn create_ballot_dispatch(State(s): State<SharedState>, Json(req): Json<Value>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let v = json!({ "dispatch_id": id, "status": "dispatched", "created_at": chrono::Utc::now().to_rfc3339(), "request": req });
    match s.diaspora_store.create_ballot_dispatch(&id, v.clone()).await {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "create_ballot_dispatch failed");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": "internal error"}))).into_response()
        }
    }
}

pub async fn list_ballot_dispatches(State(s): State<SharedState>) -> impl IntoResponse {
    match s.diaspora_store.list_ballot_dispatches().await {
        Ok(mut items) => {
            items.sort_by(|a, b| a["dispatch_id"].as_str().cmp(&b["dispatch_id"].as_str()));
            Json(page(&serde_json::to_value(items).unwrap())).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "list_ballot_dispatches failed");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": "internal error"}))).into_response()
        }
    }
}

pub async fn get_ballot_dispatch(State(s): State<SharedState>, Path(id): Path<String>) -> impl IntoResponse {
    match s.diaspora_store.get_ballot_dispatch(&id).await {
        Ok(Some(v)) => Json(v).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "detail": "dispatch not found" }))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "get_ballot_dispatch failed");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": "internal error"}))).into_response()
        }
    }
}

pub async fn confirm_ballot_delivery(State(s): State<SharedState>, Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    match s.diaspora_store.confirm_ballot_delivery(&id, req).await {
        Ok(Some(v)) => Json(v).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "detail": "dispatch not found" }))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "confirm_ballot_delivery failed");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": "internal error"}))).into_response()
        }
    }
}

pub async fn list_return_dispatches(State(s): State<SharedState>) -> impl IntoResponse {
    match s.diaspora_store.list_return_dispatches().await {
        Ok(mut items) => {
            items.sort_by(|a, b| a["return_id"].as_str().cmp(&b["return_id"].as_str()));
            Json(page(&serde_json::to_value(items).unwrap())).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "list_return_dispatches failed");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": "internal error"}))).into_response()
        }
    }
}
