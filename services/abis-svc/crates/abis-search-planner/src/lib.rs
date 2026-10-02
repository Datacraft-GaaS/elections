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

//! `eel-abis-search-planner`: cost-based 1:N query planner across shards and accelerators.
//!
//! Plans the cheapest sequence of: demographic_prune -> face_ANN -> fingerprint_exact ->
//! iris_exact, and executes concurrently across shards with early-termination when the
//! confidence threshold is reached.
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod estimator;
pub mod executor;
pub mod plan;

pub use domain::{ExecutionPlan, PlanStep, ShardSpec, StepKind};
pub use error::PlannerError;
pub use plan::QueryPlanner;
