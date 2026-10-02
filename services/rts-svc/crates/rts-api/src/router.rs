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

use axum::{routing::{get, post}, Router};
use crate::handlers::*;
use crate::store::SharedState;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .layer(axum::Extension(state.clone()))
        .route("/api/v1/rts/forms", post(submit_form))
        .route("/api/v1/rts/forms/sms-fallback", post(sms_fallback))
        .route("/api/v1/rts/forms/34b", post(compile_form34b))
        .route("/api/v1/rts/forms/34c", post(compile_form34c))
        .route("/api/v1/rts/forms/{form_id}/proof", get(get_form_proof))
        .route("/api/v1/rts/tokens", post(provision_token))
        .route("/api/v1/rts/events", post(ingest_event))
        .route("/api/v1/rts/acks/{scan_id}", get(get_ack))
        .route("/api/v1/rts/snapshots", get(list_snapshots))
        .route("/api/v1/rts/snapshots/{scope}/{code}", get(get_snapshot))
        .route("/api/v1/rts/icr/{form_id}", get(get_icr_job))
        .route("/api/v1/rts/icr/{job_id}/result", get(get_icr_result))
        .route("/api/v1/rts/audit/{form_id}", get(get_audit))
        .route("/api/v1/rts/ro/disputes", post(create_dispute))
        .route("/api/v1/rts/ro/disputes/{id}/resolve", post(resolve_dispute))
        .with_state(state)
}