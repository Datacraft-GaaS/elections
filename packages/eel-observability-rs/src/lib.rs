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

//! Observability helpers for the eel platform (ADR-0016).
//!
//! - OTel init: traces to Tempo, metrics to Prometheus, logs via tracing
//! - SLO metric macro: `slo!(service, indicator, target)`
//! - Audit shadow trigger: creates PG trigger mirroring mutations to audit.<table>_history
//! - PII log scrubber: tracing layer detecting PIIRestricted/PIISensitive in log fields
#![warn(missing_docs)]

pub mod audit_shadow;
pub mod error;
pub mod otel;
pub mod scrubber;
pub mod slo;

pub use audit_shadow::{audit_shadow_sql, install_audit_shadow};
pub use error::ObservabilityError;
pub use otel::{init_otel, OtelConfig, OtelGuard};
pub use scrubber::PiiScrubberLayer;
pub use slo::SloDefinition;
