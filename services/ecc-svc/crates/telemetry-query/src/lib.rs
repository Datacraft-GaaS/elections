//! eel-ecc-telemetry-query: admin query console for mobile telemetry (TELE-C).
//!
//! Re-exports the public API for external callers. Internal modules are
//! `pub(crate)` unless they are part of the public surface.

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
