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

//! abis-modality-periocular crate.
#![warn(missing_docs)]
pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use domain::{LbpHistogram, PeriocularTemplate, PATCH_SIZE};
pub use matcher::{chi_square, match_periocular, Score, DEFAULT_THRESHOLD};
pub use parser::{compute_lbp_histogram, parse_periocular, serialize_periocular, PeriocularError};
pub use quality::{laplacian_variance, score_quality, PeriocularQuality};
