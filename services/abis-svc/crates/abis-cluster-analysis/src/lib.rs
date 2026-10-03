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

//! `eel-abis-cluster-analysis`: multi-way duplicate detection via connected-components.
//!
//! Builds a match graph from adjudication-confirmed pairs, then finds connected
//! components to identify duplicate clusters. Runs nightly over the last 30 days.
#![warn(missing_docs)]

pub mod api;
pub mod cluster;
pub mod detector;
pub mod domain;
pub mod error;
pub mod resolver;

pub use cluster::connected_components;
pub use domain::{ClusterRecord, ClusterResolution, MatchEdge};
pub use error::ClusterError;
