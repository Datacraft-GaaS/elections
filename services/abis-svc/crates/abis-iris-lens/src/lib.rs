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

//! Contact lens detection for iris PAD (presentation attack detection).
//!
//! References:
//!   Erdogmus, N., & Marcel, S. (2013). Spoofing 2D face recognition systems with 3D masks. BTAS.
//!   LivDet-Iris 2017.
//!
//! Detects three lens artefact classes: transparent, coloured, and patterned.
#![warn(missing_docs)]

pub mod coloured;
pub mod detector;
pub mod patterned;
pub mod transparent;

pub use detector::{DetectError, LensDetector, LensKind, LensResult};
