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

//! special-voter: domain aggregates, events, workflows, and guardrails for M25.
//!
//! WP-5.5 — M25 special voter workflows: remand prisoners, hospital patients,
//! KDF/GSU/Prison Service, refugees (conditional), and IDPs.
//! Elections Act ss.5, 44, 44A, 54, 72.
#![warn(missing_docs)]

pub mod aggregates;
pub mod http;
/// Typed models for support-special-voter contract.
pub mod spec;
pub mod events;
pub mod guards;
pub mod workflows;
