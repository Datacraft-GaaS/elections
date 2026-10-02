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

//! ABIS multi-modal matching engine.
//!
//! Provides fingerprint (10-print, ISO/IEC 19794-2), face (512-d embedding),
//! and iris (2048-bit IrisCode) matching at 30M-enrolment scale.
//!
//! # Architecture
//! - `modality` -- template types and the `Template` trait
//! - `matcher`  -- per-modality matchers and multi-modal fusion
//! - `index`    -- HNSW ANN index, 256-shard hasher, Bloom-filter pruner
//! - `quality`  -- quality gates (NFIQ2 / ICAO / ISO 29794)
//! - `fairness` -- per-demographic FMR/FNMR harness (ISO 19795-10)
//! - `pad`      -- presentation attack detection interface
#![warn(missing_docs)]

pub mod adjudication;
pub mod batch;
pub mod domain;
pub mod fairness;
pub mod fmr_fnmr;
pub mod index;
pub mod matcher;
pub mod modality;
pub mod pad;
pub mod pipeline;
pub mod quality;

#[cfg(any(test, feature = "mock"))]
pub mod mock_matcher;

pub use domain::*;
pub use pipeline::{MatchPipeline, Pipeline11, Pipeline1N};

#[cfg(any(test, feature = "mock"))]
pub use mock_matcher::MockMatcher;
