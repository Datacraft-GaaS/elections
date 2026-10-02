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

//! SpiceDB permission checks for the telemetry query API.
//!
//! Roles:
//!   TELEMETRY_ANALYST  -- required to run any query
//!   TELEMETRY_ADMIN    -- required for evidence export

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("forbidden: actor {actor} lacks role {role}")]
    Forbidden { actor: String, role: String },

    #[error("SpiceDB check failed: {0}")]
    SpiceDb(String),
}

/// Required SpiceDB roles.
pub const ROLE_ANALYST: &str = "TELEMETRY_ANALYST";
pub const ROLE_ADMIN: &str = "TELEMETRY_ADMIN";

/// Check that the actor SVID holds the required role.
///
/// In production this calls SpiceDB via `eel-auth-rs`. The stub here returns
/// `Ok(())` so the crate compiles without a live SpiceDB. Replace the stub
/// with the real SpiceDB client from `eel-auth-rs` when wiring the service.
pub async fn check_role(actor_svid: &str, role: &str) -> Result<(), PermissionError> {
    // Stub: in production, call:
    //   eel_auth_rs::spicedb::check_permission(actor_svid, "telemetry", role).await
    //
    // For now allow all authenticated actors so the crate builds standalone.
    if actor_svid.is_empty() {
        return Err(PermissionError::Forbidden {
            actor: actor_svid.to_string(),
            role: role.to_string(),
        });
    }

    tracing::debug!(actor = actor_svid, role = role, "permission check (stub: allowed)");
    Ok(())
}

/// Guard for read-only query endpoints.
pub async fn require_analyst(actor_svid: &str) -> Result<(), PermissionError> {
    check_role(actor_svid, ROLE_ANALYST).await
}

/// Guard for evidence export endpoints.
pub async fn require_admin(actor_svid: &str) -> Result<(), PermissionError> {
    check_role(actor_svid, ROLE_ADMIN).await
}
