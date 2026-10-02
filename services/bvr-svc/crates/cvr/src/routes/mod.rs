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

pub mod batches;
pub mod events;
pub mod family_notify;
pub mod field_notifications;
pub mod health;
pub mod snapshots;
pub mod spec;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(batches::router())
        .merge(events::router())
        .merge(snapshots::router())
        .merge(family_notify::router())
        .merge(field_notifications::router())
        .merge(spec::router())
}
