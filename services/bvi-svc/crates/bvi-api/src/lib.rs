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

//! BVI backend REST API (modules M05 bvi-device, M06 bvi-backend).
//!
//! Contract: `packages/openapi-contracts/bvi/openapi.yaml` v1.0.0.
//! Conventions per ADR-0011: Idempotency-Key on all writes with a 24 hour
//! response snapshot replay, RFC 9457 problem+json errors, x-eel-tier aware
//! auth posture (internal mTLS terminated at the Envoy sidecar, tier 2/3 JWT).
#![warn(missing_docs)]

pub mod error;
pub mod idem;
pub mod models;
pub mod routes;
pub mod state;
pub mod store;

use axum::Router;

/// Build the full API router for any store implementation.
pub fn app<S: store::BviStore>(state: state::AppState<S>) -> Router {
    routes::router(state)
}
