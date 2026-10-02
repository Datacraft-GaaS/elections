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
    routing::{get, post, put},
    Router,
};

use crate::handlers;

/// Bind all archive routes and return the combined Router.
pub fn router() -> Router {
    // Public API routes (openapi-contracts/support-archive/archive.yaml)
    let public = Router::new()
        .route(
            "/api/v1/support/archive/collections",
            post(handlers::create_archival_collection).get(handlers::list_archival_collections),
        )
        .route(
            "/api/v1/support/archive/collections/:collection_id",
            get(handlers::get_archival_collection),
        )
        .route(
            "/api/v1/support/archive/retrieval-requests",
            post(handlers::create_retrieval_request),
        )
        .route(
            "/api/v1/support/archive/retrieval-requests/:request_id",
            get(handlers::get_retrieval_request),
        );

    // Module-level routes
    let module = Router::new()
        .route(
            "/v1/archive/collections/:id/ingest",
            post(handlers::start_ingest),
        )
        .route(
            "/v1/archive/collections/:id/ingest/:job_id",
            get(handlers::get_ingest_job),
        )
        .route(
            "/v1/archive/collections/:id/seal",
            put(handlers::seal_collection),
        )
        .route(
            "/v1/archive/collections/:id/attest",
            post(handlers::attest_collection),
        )
        .route(
            "/v1/archive/collections/:id/access",
            post(handlers::request_access),
        )
        .route(
            "/v1/archive/collections/:id/destroy",
            post(handlers::destroy_collection),
        )
        .route(
            "/v1/archive/collections/:id/format-migration",
            get(handlers::list_format_migrations).post(handlers::start_format_migration),
        );

    Router::new().merge(public).merge(module)
}
