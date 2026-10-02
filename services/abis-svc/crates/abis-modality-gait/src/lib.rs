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

//! abis-modality-gait crate.
#![warn(missing_docs)]
pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use domain::{GaitCycle, GaitFrame, GaitTemplate, JointAngle, NUM_JOINTS};
pub use matcher::{match_gait, Score, DEFAULT_THRESHOLD};
pub use parser::{parse_gait, serialize_gait};
pub use quality::{score_quality, GaitQuality};

#[derive(Debug, thiserror::Error)]
pub enum GaitError {
    #[error("invalid magic bytes: expected GAIT")]
    InvalidMagic,
    #[error("buffer too short: need at least {need} bytes, got {got}")]
    BufferTooShort { need: usize, got: usize },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
