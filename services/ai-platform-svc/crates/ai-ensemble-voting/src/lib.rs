//! `eel-ai-ensemble-voting` (HARD-22)
//!
//! K-of-N ensemble voting across every AI model decision.
//! N models are dispatched in parallel via Rayon; at least K must agree
//! on a label for the decision to be accepted.

pub mod ensemble;
pub mod error;
pub mod policy;
pub mod registry;

pub use ensemble::{Ensemble, Model, Prediction};
pub use error::EnsembleError;
pub use policy::{DisagreementAction, EnsemblePolicy};
pub use registry::{DecisionClass, ModelRegistry};
