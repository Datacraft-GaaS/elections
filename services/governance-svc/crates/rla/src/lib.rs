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

//! Risk-limiting audit — BRAVO audit lifecycle over [`Store`].
#![warn(missing_docs)]
pub mod engine;
/// HTTP handlers driving the BRAVO audit lifecycle over [`Store`].
pub mod handlers;
pub mod manifest;
pub mod models;
pub mod router;
pub mod spec;
pub mod spec_state;
pub mod store;

pub use spec_state::{SharedSpecState, SpecState};

/// Composite router state: the internal BRAVO store plus the public spec
/// tracker. `FromRef` lets handlers extract either piece independently.
#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub store: store::Store,
    pub spec_state: SharedSpecState,
}
