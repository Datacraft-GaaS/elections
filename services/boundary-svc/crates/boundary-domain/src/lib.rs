//! Domain model for `boundary-svc`.
//!
//! Pure value types and aggregates. No I/O, no async, no database deps.
#![warn(missing_docs)]

pub mod aggregate;
pub mod events;
pub mod types;

pub use aggregate::Boundary;
pub use events::BoundaryEvent;
pub use types::{
    BoundaryChange, BoundaryKind, BoundaryStatus, FairnessMetric, GeofencePolygon,
    MetricName, SubmissionStatus, SubmitterClass,
};
