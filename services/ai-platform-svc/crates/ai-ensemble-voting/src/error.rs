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
