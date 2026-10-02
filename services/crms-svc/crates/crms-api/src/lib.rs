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

//! crms-api — HTTP surface for the Candidate Registration & Management Service.
//!
//! WP-4.11. Exposes the crms-domain aggregates (candidates, nominations,
//! clearance, quota, gazette, ballot, party/coalition, primaries, payments,
//! symbols) as axum endpoints. The router is composed with the shared
//! `eel-api-server-rs` builder for auth, load-shedding, idempotency, and
//! observability plumbing.

#![warn(missing_docs)]

pub mod handlers;
pub mod models;
pub mod router;
pub mod spec;
pub mod state;

pub use router::build_router;
pub use state::AppState;
