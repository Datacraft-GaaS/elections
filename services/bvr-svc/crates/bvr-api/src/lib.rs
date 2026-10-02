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

//! BVR API — voter registration and verification service.
#![warn(missing_docs)]
pub mod error;
pub mod iprs;
pub mod nfc;
pub mod routes;
pub mod state;
pub mod store;

use axum::Router;

/// The election scope the Merkle endpoints report on.
pub const MERKLE_SCOPE: &str = "2027-general";

pub fn app(state: crate::state::AppState) -> Router {
    routes::router().with_state(state)
}
