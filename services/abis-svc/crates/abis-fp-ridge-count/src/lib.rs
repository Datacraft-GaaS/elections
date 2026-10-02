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

//! Ridge counting between minutiae and K-plet descriptor matching.
//!
//! Proper Bozorth3-style ridge counting traverses the orientation field between
//! two minutiae and counts ridge crossings via zero-crossings perpendicular to
//! the path direction.
//!
//! Reference: C. I. Watson, "NIST Special Database 4, Fingerprint Database,"
//! NIST, 1992; C. Watson et al., "User's Guide to NIST Biometric Image Software
//! (NBIS)," NISTIR 7392, 2007, describing the Bozorth3 algorithm.
#![warn(missing_docs)]

pub mod count;
pub mod kplet;
pub mod matcher;

pub use count::{count_ridges, OrientationField};
pub use kplet::{KPlet, KPletDescriptor};
pub use matcher::{KPletMatcher, RidgeCountMatcher};
