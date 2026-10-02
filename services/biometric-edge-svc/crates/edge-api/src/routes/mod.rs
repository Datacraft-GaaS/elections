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

pub mod attestation;
pub mod config;
pub mod distress;
pub mod enrolment;
pub mod health;
pub mod identification;
pub mod mrz;
pub mod ota;
pub mod sync;
pub mod telemetry;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(health::router())
        .merge(enrolment::router())
        .merge(identification::router())
        .merge(mrz::router())
        .merge(attestation::router())
        .merge(config::router())
        .merge(sync::router())
        .merge(ota::router())
        .merge(telemetry::router())
        .merge(distress::router())
}
