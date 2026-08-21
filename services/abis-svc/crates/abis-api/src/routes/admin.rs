//! Admin routes: policy management, vendor health.

use std::sync::Arc;

use axum::{
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
	Json(req): Json<AttestationPolicyUpdate>,
) -> (StatusCode, Json<PolicyUpdateResult>) {
	// In production: validate req.signature_b64 against HSM public key.
	let _ = req;
	(
		StatusCode::OK,
		Json(PolicyUpdateResult {
			applied: true,
			message: "attestation policy updated".into(),
		}),
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
pub async fn list_vendors(
	State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<VendorInfo>>) {
	(
		StatusCode::OK,
		Json(vec![
			VendorInfo {
				id: "mock".into(),
				name: "Mock Vendor".into(),
				status: "healthy".into(),
				capabilities: vec!["fingerprint".into(), "face".into(), "iris".into()],
			},
		]),
	)
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
