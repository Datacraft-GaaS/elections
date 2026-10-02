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

//! Shared handler state.

/// Application state threaded into every axum handler. Generic over the
/// store so tests can run the full router against an in-memory fake.
#[derive(Clone)]
pub struct AppState<S> {
    pub store: S,
}

impl<S: crate::store::BviStore> AppState<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
