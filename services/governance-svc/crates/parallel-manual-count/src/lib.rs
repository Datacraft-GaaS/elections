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

//! parallel-manual-count crate.
#![warn(missing_docs)]
pub mod api;
pub mod divergence;
pub mod ingest;
pub mod plan;
pub mod rla_feed;
pub mod sample;

pub use divergence::{compute_divergence, DivergenceReport};
pub use plan::SamplePlan;
pub use sample::{sample_stations, SampledStation, Stratum};
