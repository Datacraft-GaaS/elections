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
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Public API routes (packages/openapi-contracts/support-archive/archive.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/support/archive/collections
/// operationId: createArchivalCollection
pub async fn create_archival_collection() -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "status": "created" })))
}

/// GET /api/v1/support/archive/collections
/// operationId: listArchivalCollections
pub async fn list_archival_collections() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "items": [] })))
}

/// GET /api/v1/support/archive/collections/{collection_id}
/// operationId: getArchivalCollection
pub async fn get_archival_collection(
    Path(_collection_id): Path<Uuid>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

/// POST /api/v1/support/archive/retrieval-requests
/// operationId: createRetrievalRequest
pub async fn create_retrieval_request() -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "status": "created" })))
}

/// GET /api/v1/support/archive/retrieval-requests/{request_id}
/// operationId: getRetrievalRequest
pub async fn get_retrieval_request(
    Path(_request_id): Path<Uuid>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

// ---------------------------------------------------------------------------
// Module-level routes (/v1/archive/...)
// ---------------------------------------------------------------------------

/// POST /v1/archive/collections/{id}/ingest
pub async fn start_ingest(Path(_id): Path<Uuid>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "status": "pending" })))
}

/// GET /v1/archive/collections/{id}/ingest/{job_id}
pub async fn get_ingest_job(
    Path((_id, _job_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

/// PUT /v1/archive/collections/{id}/seal
pub async fn seal_collection(Path(_id): Path<Uuid>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "sealed" })))
}

/// POST /v1/archive/collections/{id}/attest
pub async fn attest_collection(Path(_id): Path<Uuid>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "status": "attested" })))
}

/// POST /v1/archive/collections/{id}/access
pub async fn request_access(Path(_id): Path<Uuid>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "status": "pending" })))
}

/// POST /v1/archive/collections/{id}/destroy
pub async fn destroy_collection(Path(_id): Path<Uuid>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "status": "destroyed" })))
}

/// GET /v1/archive/collections/{id}/format-migration
pub async fn list_format_migrations(Path(_id): Path<Uuid>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "items": [] })))
}

/// POST /v1/archive/collections/{id}/format-migration
pub async fn start_format_migration(Path(_id): Path<Uuid>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "status": "queued" })))
}
