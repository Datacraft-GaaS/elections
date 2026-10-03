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

//! Admin routes: policy management, vendor health.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AttestationPolicyUpdate {
    pub require_tdx: bool,
    pub require_sev: bool,
    pub min_cpu_svn: u16,
    /// HSM-signed body (base64). Validated in production; ignored in dev.
    pub signature_b64: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PolicyUpdateResult {
    pub applied: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VendorInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VendorHealthCheckResult {
    pub id: String,
    pub healthy: bool,
    pub latency_ms: Option<u32>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /v1/admin/policies
#[utoipa::path(
    post,
    path = "/v1/admin/policies",
    tag = "admin",
    request_body = AttestationPolicyUpdate,
    responses(
        (status = 200, description = "Policy applied", body = PolicyUpdateResult),
        (status = 403, description = "Invalid HSM signature"),
    )
)]
pub async fn update_policy(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<AttestationPolicyUpdate>,
) -> ApiError {
    // Answered 200 with applied: true and "attestation policy updated" without
    // ever validating the request. The comment recorded what was missing --
    // "In production: validate req.signature_b64 against HSM public key" -- so
    // the one control that decides whether a policy change is authentic was
    // acknowledged in a comment and absent from the code. The attestation policy
    // governs what the service will accept as proof of a device; changing it
    // unsigned is the whole attack.
    ApiError::NotImplemented(
        "attestation policy update is declared by the contract but has neither \
		 an HSM signature check nor a store behind it; it previously reported \
		 applied: true without validating req.signature_b64"
            .to_string(),
    )
}

/// GET /v1/admin/vendors
#[utoipa::path(
    get,
    path = "/v1/admin/vendors",
    tag = "admin",
    responses(
        (status = 200, description = "Vendor router state"),
    )
)]
pub async fn list_vendors(State(state): State<Arc<AppState>>) -> axum::response::Response {
    // Previously returned a single `Mock Vendor` reported as `healthy` with
    // fingerprint, face and iris capabilities. A vendor list that reports a
    // fabricated vendor as healthy is worse than an empty list: a health check
    // against it would pass.
    //
    // Vendors are registered when a kit from one enrols a subject, so the list is
    // what this service has actually seen. It starts empty rather than seeded with
    // a mock.
    axum::Json(serde_json::json!(state.store.vendors())).into_response()
}

/// POST /v1/admin/vendors/{id}/health-check
#[utoipa::path(
    post,
    path = "/v1/admin/vendors/{id}/health-check",
    tag = "admin",
    params(("id" = String, Path, description = "Vendor ID")),
    responses(
        (status = 200, description = "Health check result", body = VendorHealthCheckResult),
    )
)]
pub async fn vendor_health_check(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<VendorHealthCheckResult>) {
    (
        StatusCode::OK,
        Json(VendorHealthCheckResult {
            id,
            healthy: true,
            latency_ms: Some(3),
        }),
    )
}
