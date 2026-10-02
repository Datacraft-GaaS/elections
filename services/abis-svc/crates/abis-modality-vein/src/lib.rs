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

//! abis-modality-vein crate.
#![warn(missing_docs)]
pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use domain::{BodySide, VeinTemplate, VeinType};
pub use matcher::{match_vein, ncc, Score, DEFAULT_THRESHOLD};
pub use parser::{parse_nir_bitmap, serialize_nir_bitmap, VeinError};
pub use quality::{count_branch_points, score_quality, VeinQuality};
