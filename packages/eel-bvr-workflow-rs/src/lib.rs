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

//! Declarative model for BVR enrolment workflows (BiCCL-equivalent).
//!
//! A workflow is described as data, not code: the desktop client generates its
//! form layout, field types, and validation from this spec, so adding a data
//! item or reordering steps needs no UI change. This mirrors the intent of
//! NIST's BiCCL (NIST IR 7531), which exists so that biometric acquisition
//! behaviour can be *formally described* rather than hard-coded.
//!
//! Parsing is separated from resolution: [`WorkflowSpec`] is the literal
//! document, while [`CompiledWorkflow`](crate::CompiledWorkflow) is the result of
//! [`compile`](crate::compile), which checks cross-references and pre-resolves
//! every validation rule. Nothing consumes an uncompiled spec.

#![warn(missing_docs)]

pub mod compile;
pub mod model;
pub mod validate;

pub use compile::{CompileError, CompiledWorkflow, compile};
pub use model::{
    CaptureKind, CapturePolicy, CaptureRequirement, Constraints, DataItem, DataKind, Step,
    StepKind, WorkflowSpec,
};
pub use validate::{SubmissionError, ValidationError, validate_submission};
