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

//! `eel-ai-ensemble-voting` (HARD-22)
//!
//! K-of-N ensemble voting across every AI model decision.
//! N models are dispatched in parallel via Rayon; at least K must agree
//! on a label for the decision to be accepted.
#![warn(missing_docs)]

pub mod ensemble;
pub mod error;
pub mod policy;
pub mod registry;

pub use ensemble::{Ensemble, Model, Prediction};
pub use error::EnsembleError;
pub use policy::{DisagreementAction, EnsemblePolicy};
pub use registry::{DecisionClass, ModelRegistry};
