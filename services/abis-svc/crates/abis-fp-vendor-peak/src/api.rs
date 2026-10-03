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

//! Axum 0.8 admin routes for peak-policy management.
//!
//! Routes:
//!   POST /v1/admin/peak-policy/enable   - enable with optional freeze window
//!   POST /v1/admin/peak-policy/disable  - disable (Disabled variant)
//!   GET  /v1/admin/peak-policy/status   - current policy + metrics snapshot

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::metrics::snapshot;
use crate::peak_policy::PeakPolicy;

/// Shared application state for the admin API.
#[derive(Clone)]
pub struct PolicyState {
    pub policy: Arc<RwLock<PeakPolicy>>,
}

impl PolicyState {
    pub fn new(initial: PeakPolicy) -> Self {
        Self {
            policy: Arc::new(RwLock::new(initial)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EnableRequest {
    /// If provided, enables a freeze window. Otherwise AlwaysEnabled.
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub active: bool,
    pub policy: String,
    pub metrics: MetricsPayload,
}

#[derive(Debug, Serialize)]
pub struct MetricsPayload {
    pub agreements: u64,
    pub disagreements: u64,
}

async fn enable_handler(
    State(s): State<PolicyState>,
    Json(req): Json<EnableRequest>,
) -> impl IntoResponse {
    let policy = match (req.from, req.to) {
        (Some(from), Some(to)) => PeakPolicy::EnabledDuringFreeze { from, to },
        _ => PeakPolicy::AlwaysEnabled,
    };
    *s.policy.write().await = policy;
    (StatusCode::OK, Json(serde_json::json!({"ok": true})))
}

async fn disable_handler(State(s): State<PolicyState>) -> impl IntoResponse {
    *s.policy.write().await = PeakPolicy::Disabled;
    (StatusCode::OK, Json(serde_json::json!({"ok": true})))
}

async fn status_handler(State(s): State<PolicyState>) -> impl IntoResponse {
    let policy = s.policy.read().await.clone();
    let now = Utc::now();
    let active = policy.is_active(now);
    let snap = snapshot();
    let resp = StatusResponse {
        active,
        policy: format!("{policy:?}"),
        metrics: MetricsPayload {
            agreements: snap.agreements,
            disagreements: snap.disagreements,
        },
    };
    (StatusCode::OK, Json(resp))
}

/// Build the admin router. Nest under your service's root router.
pub fn router(state: PolicyState) -> Router {
    Router::new()
        .route("/v1/admin/peak-policy/enable", post(enable_handler))
        .route("/v1/admin/peak-policy/disable", post(disable_handler))
        .route("/v1/admin/peak-policy/status", get(status_handler))
        .with_state(state)
}
