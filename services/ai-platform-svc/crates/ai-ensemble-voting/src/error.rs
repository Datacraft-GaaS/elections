//! Error types for ai-ensemble-voting.

use thiserror::Error;

use crate::policy::DisagreementAction;

#[derive(Debug, Error)]
pub enum EnsembleError {
    #[error("policy invalid: k={k} > n={n}")]
    PolicyInvalid { k: usize, n: usize },

    #[error("no models registered for decision class '{class}'")]
    NoModelsRegistered { class: String },

    #[error(
        "ensemble disagreement: only {agreed}/{required} models agreed on label '{label}'; action={action:?}"
    )]
    Disagreement {
        agreed: usize,
        required: usize,
        label: String,
        action: DisagreementAction,
    },

    #[error("model execution error: {0}")]
    ModelError(String),
}
