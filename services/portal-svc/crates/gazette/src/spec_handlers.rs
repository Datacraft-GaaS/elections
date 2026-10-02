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

//! Public contract handlers (gazette/calendar.yaml inspection + notices).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::api::AppState;
use crate::spec::{InspectionWindow, ListObjection, ObjectionStatus};

/// GET /api/v1/gazette/notices/{notice_id} — fetch a gazette notice.
pub async fn get_gazette_notice(
    State(_state): State<AppState>,
    Path(notice_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!(%notice_id, "get_gazette_notice");
    (StatusCode::NOT_FOUND, Json(json!({ "detail": "notice not found" })))
}

/// GET /api/v1/gazette/inspection/windows — list inspection windows.
pub async fn list_inspection_windows(State(state): State<AppState>) -> impl IntoResponse {
    tracing::info!("list_inspection_windows");
    let guard = state.windows.lock().expect("lock poisoned");
    let mut items: Vec<InspectionWindow> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.opens_at.cmp(&b.opens_at));
    Json(json!({ "items": items, "total": items.len() }))
}

/// GET /api/v1/gazette/inspection/windows/{window_id} — fetch one window.
pub async fn get_inspection_window(
    State(state): State<AppState>,
    Path(window_id): Path<String>,
) -> impl IntoResponse {
    tracing::info!(window_id, "get_inspection_window");
    let guard = state.windows.lock().expect("lock poisoned");
    match guard.get(&window_id) {
        Some(w) => Json(w.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "window not found" }))).into_response(),
    }
}

/// POST /api/v1/gazette/inspection/objections — file a list objection.
pub async fn file_list_objection(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!("file_list_objection");
    let reference_code = format!("OBJ-{}", Uuid::now_v7().simple());
    let objection = ListObjection {
        objection_id: Uuid::now_v7().to_string(),
        reference_code: reference_code.clone(),
        status: "submitted".to_string(),
        submitted_at: chrono::Utc::now().to_rfc3339(),
    };
    let _ = req;
    let mut guard = state.objections.lock().expect("lock poisoned");
    guard.insert(reference_code.clone(), objection.clone());
    drop(guard);
    (StatusCode::CREATED, Json(objection)).into_response()
}

/// GET /api/v1/gazette/inspection/objections/{reference_code} — objection status.
pub async fn get_objection_status(
    State(state): State<AppState>,
    Path(reference_code): Path<String>,
) -> impl IntoResponse {
    tracing::info!(reference_code, "get_objection_status");
    let guard = state.objections.lock().expect("lock poisoned");
    match guard.get(&reference_code) {
        Some(o) => Json(ObjectionStatus {
            reference_code: reference_code.clone(),
            status: o.status.clone(),
            updated_at: o.submitted_at.clone(),
        })
        .into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "objection not found" }))).into_response(),
    }
}
