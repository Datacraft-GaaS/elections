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

//! Authentication & authorization (Keycloak JWT + SpiceDB ReBAC).
//!
//! Configured entirely via environment: if `SPICEDB_ENDPOINT` and `JWKS_URL`
//! are both set, the authed router enforces a JWT + SpiceDB permission check
//! on every endpoint (fail-closed). When unset, the service runs unauthenticated
//! (local development and the hermetic test suite).

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use eel_auth_rs::{Consistency, Decision, JwksVerifier, LogSink, SpiceDbClient, Subject};

#[derive(Clone)]
pub struct Auth {
    verifier: Arc<JwksVerifier>,
    spicedb: Arc<SpiceDbClient<LogSink>>,
}

impl Auth {
    /// Build auth from the environment, or `None` when auth is not configured.
    pub fn from_env() -> Option<Self> {
        let spicedb_endpoint = std::env::var("SPICEDB_ENDPOINT").ok()?;
        let jwks_url = std::env::var("JWKS_URL").ok()?;
        let realm = std::env::var("JWKS_REALM").unwrap_or_else(|_| "eel".to_string());
        let audience = std::env::var("EEL_OIDC_AUDIENCE").ok();

        Some(Self {
            verifier: Arc::new(JwksVerifier::with_audience(jwks_url, realm, audience)),
            spicedb: Arc::new(SpiceDbClient::new(spicedb_endpoint, LogSink)),
        })
    }

    /// Build auth from explicit endpoints (programmatic wiring and tests).
    pub fn from_endpoints(
        jwks_url: impl Into<String>,
        spicedb_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            verifier: Arc::new(JwksVerifier::new(jwks_url, "eel")),
            spicedb: Arc::new(SpiceDbClient::new(spicedb_endpoint, LogSink)),
        }
    }
}

/// Coarse ReBAC gate applied to every protected route. Verifies the bearer
/// token against Keycloak JWKS, then checks `inventory_access` on `inventory`
/// in SpiceDB. Fails closed when either dependency is unavailable.
pub async fn require_auth(State(auth): State<Auth>, mut req: Request, next: Next) -> Response {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|t| !t.is_empty());

    let Some(token) = token else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "missing bearer token" })),
        )
            .into_response();
    };

    let subject: Subject = match auth.verifier.verify(token).await {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "invalid token" })),
            )
                .into_response()
        }
    };

    match auth
        .spicedb
        .check(
            &subject,
            "inventory_access",
            "inventory",
            Consistency::MinimizeLatency,
            serde_json::json!({}),
        )
        .await
    {
        Ok(Decision::Allow) => {}
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "forbidden" })),
            )
                .into_response()
        }
    }

    req.extensions_mut().insert(subject);
    next.run(req).await
}
