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

//! Cross-sensor iris template alignment.
//!
//! Normalises iris templates between NIR (700-1000 nm) and visible
//! (400-700 nm) sensor modalities via a learned linear projection,
//! enabling cross-wavelength matching without re-enrolment.
#![warn(missing_docs)]

pub mod aligner;
pub mod projection;

pub use aligner::{AlignError, CrossSensorAligner};
