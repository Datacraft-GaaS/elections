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

use std::sync::Arc;

use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::handlers::{
    binary_integrity_check, file_grievance, get_anomalies, get_queue_snapshot, get_results,
    get_results_by_scope, get_token_proof, health, queue_check_in, queue_check_out, record_token,
    report_clone, sse_events, verify_token,
};
use crate::spec::BffState;
use crate::sse::ShardedBroadcast;
use crate::verify_store::SharedVerifyStore;
use axum::extract::FromRef;

/// Combined router state: spec handlers extract `Arc<BffState>`, verify
/// handlers extract `Arc<ShardedBroadcast>` — both via [`FromRef`], so all
/// routes live in one merged `Router` with a single `with_state` call.
#[derive(Clone)]
pub struct AppState {
    /// BFF state (config, idempotency store, rate limits).
    pub bff: Arc<BffState>,
    /// SSE broadcast for verify/dashboard streams.
    pub broadcast: Arc<ShardedBroadcast>,
    /// The recorded token set, per-station queues, and clone reports behind the
    /// public `/verify/v1/*` surface.
    ///
    /// This was added because every one of those handlers answered from a
    /// constant: `verify_token` reported `included: true` for any token, and
    /// `get_queue_snapshot` reported `queue_length: 0` for any station.
    pub verify: SharedVerifyStore,
    /// Sessions, stations, dispatch, incidents, push tokens and observer passes
    /// behind the `/api/v1/mobile/bff/*` and `/v1/agent/*` surfaces.
    ///
    /// Added because those handlers answered from constants: station status
    /// reported a fixed turnout of 1247 and a fixed battery of 87.5 for every
    /// station, dispatch returned two fabricated assignments for any worker, and
    /// the OIDC session endpoints minted JWT-shaped strings with no session behind
    /// them.
    pub mobile: crate::mobile_store::SharedMobileStore,
}

impl FromRef<AppState> for Arc<BffState> {
    fn from_ref(state: &AppState) -> Self {
        state.bff.clone()
    }
}

impl FromRef<AppState> for Arc<ShardedBroadcast> {
    fn from_ref(state: &AppState) -> Self {
        state.broadcast.clone()
    }
}

impl FromRef<AppState> for SharedVerifyStore {
    fn from_ref(state: &AppState) -> Self {
        state.verify.clone()
    }
}

impl FromRef<AppState> for crate::mobile_store::SharedMobileStore {
    fn from_ref(state: &AppState) -> Self {
        state.mobile.clone()
    }
}

pub fn build_router() -> Router {
    let broadcast = ShardedBroadcast::new();
    build_router_with_state(broadcast)
}

pub fn build_router_with_test_state(
    state: Arc<BffState>,
    broadcast: Arc<ShardedBroadcast>,
) -> Router {
    let app_state = AppState {
        bff: state,
        broadcast,
        verify: crate::verify_store::VerifyStore::new(),
        mobile: crate::mobile_store::MobileStore::new(),
    };
    // Delegate to spec::router which has all the BFF routes.
    // The legacy /verify/v1/* and /health routes are also merged.
    Router::new()
        .route("/health", get(health))
        .route("/verify/v1/results", get(get_results))
        .route(
            "/verify/v1/results/{scope}/{code}",
            get(get_results_by_scope),
        )
        .route("/verify/v1/token", post(verify_token))
        .route("/verify/v1/token/record", post(record_token))
        .route("/verify/v1/token/{sha}/proof", get(get_token_proof))
        .route("/verify/v1/grievances", post(file_grievance))
        .route("/verify/v1/events", get(sse_events))
        .route("/verify/v1/queue/{station}", get(get_queue_snapshot))
        .route("/verify/v1/queue/{station}/check-in", post(queue_check_in))
        .route(
            "/verify/v1/queue/{station}/check-out",
            post(queue_check_out),
        )
        .route("/verify/v1/anomalies", get(get_anomalies))
        .route("/verify/v1/clone-report", post(report_clone))
        .route("/verify/v1/binary-integrity", post(binary_integrity_check))
        .merge(crate::spec::router(app_state.clone()))
        .with_state(app_state)
}

pub fn build_router_with_state(broadcast: Arc<ShardedBroadcast>) -> Router {
    let app_state = AppState {
        bff: Arc::new(crate::spec::BffState::new(None).expect("in-memory BffState")),
        broadcast,
        verify: crate::verify_store::VerifyStore::new(),
        mobile: crate::mobile_store::MobileStore::new(),
    };
    Router::new()
        .route("/health", get(health))
        .route("/verify/v1/results", get(get_results))
        .route(
            "/verify/v1/results/{scope}/{code}",
            get(get_results_by_scope),
        )
        .route("/verify/v1/token", post(verify_token))
        .route("/verify/v1/token/record", post(record_token))
        .route("/verify/v1/token/{sha}/proof", get(get_token_proof))
        .route("/verify/v1/grievances", post(file_grievance))
        .route("/verify/v1/events", get(sse_events))
        .route("/verify/v1/queue/{station}", get(get_queue_snapshot))
        .route("/verify/v1/queue/{station}/check-in", post(queue_check_in))
        .route(
            "/verify/v1/queue/{station}/check-out",
            post(queue_check_out),
        )
        .route("/verify/v1/anomalies", get(get_anomalies))
        .route("/verify/v1/clone-report", post(report_clone))
        .route("/verify/v1/binary-integrity", post(binary_integrity_check))
        .merge(crate::spec::router(app_state.clone()))
        .with_state(app_state)
}
