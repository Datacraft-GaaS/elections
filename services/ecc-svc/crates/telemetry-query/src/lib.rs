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

//! eel-ecc-telemetry-query: admin query console for mobile telemetry (TELE-C).
//!
//! Re-exports the public API for external callers. Internal modules are
//! `pub(crate)` unless they are part of the public surface.
#![warn(missing_docs)]

pub mod api;
pub mod audit;
pub mod dsl;
pub mod evidence_bundle;
pub mod executor;
pub mod metrics;
pub mod nl_bridge;
pub mod permission;
pub mod planner;
pub mod preset;
pub mod validator;

pub use dsl::{Aggregation, Field, GeoPolygon, SignalClass, TelemetryQuery, TimeRange};
pub use evidence_bundle::EvidenceBundle;
pub use validator::{Diagnostic, TelemetryQueryValidator, ValidationError};
