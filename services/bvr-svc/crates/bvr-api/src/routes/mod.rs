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

pub mod adjudication;
pub mod enrolments;
pub mod handoff;
pub mod health;
pub mod merkle;
pub mod registers;
pub mod sar;
pub mod transfers;
pub mod voters;

use axum::Router;

/// Composes the BVR router. Handlers that need the register take `AppState`; the
/// rest are stateless and are merged in their own right.
pub fn router() -> Router<crate::state::AppState> {
    // Seeded as `Router<AppState>` so the stateful sub-routers can be merged: a
    // bare `Router::new()` is `Router<()>` and will not accept them, and a
    // `let` annotation does not propagate into the method chain.
    let router = Router::<crate::state::AppState>::new()
        .merge(health::router())
        .merge(enrolments::router())
        .merge(voters::router())
        .merge(registers::router())
        .merge(transfers::router())
        .merge(sar::router())
        .merge(merkle::router())
        .merge(adjudication::router())
        .merge(handoff::router());
    router
}
