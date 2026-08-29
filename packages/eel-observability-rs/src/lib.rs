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
