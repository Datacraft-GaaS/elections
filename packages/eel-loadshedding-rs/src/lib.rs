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

//! CoDel-style priority load shedder for the eel platform (ADR-0009, ADR-0011).
//!
//! Wraps any tower::Service with configurable load shedding:
//! - Priority classes: Admin > AuthenticatedUser > AccreditedMachine > Anonymous
//! - CoDel algorithm: targets latency budget, sheds lowest priority first
//! - Returns 503 with Retry-After and problem+json on shed
//! - Metrics: shed_count per priority, breach count, queue depth
#![warn(missing_docs)]

pub mod error;
pub mod priority;
pub mod metrics;
pub mod shedder;
pub mod layer;

pub use error::ShedError;
pub use priority::Priority;
pub use metrics::{ShedMetrics, MetricsSnapshot};
pub use shedder::{CoDelConfig, CoDelShedder};
pub use layer::{LoadSheddingLayer, LoadSheddingService};
