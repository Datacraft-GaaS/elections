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

//! intent: domain aggregates, adjudication catalogue, model registry hook,
//! and rejection-rate alerting for M27 intent preservation (WP-5.5).
//!
//! On-device pre-mark visual validation model is registered via M28 registry.
//! Image auto-delete <= 1 second post-inference (FR-M27 privacy constraint).
#![warn(missing_docs)]

pub mod aggregates;
pub mod http;
/// Typed models for intent contract.
pub mod spec;
pub mod catalogue;
pub mod events;
pub mod model;
