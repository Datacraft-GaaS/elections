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

//! Privacy-preserving face de-identification.
//!
//! References:
//! - Sweeney, L. (2002). "k-anonymity: A model for protecting privacy."
//!   IJUFKS 10(5):557-570. (k-anonymity formal definition)
//! - Dwork, C. (2006). "Differential Privacy." ICALP 2006. (DP formal
//!   definition; Gaussian mechanism used here)
//! - Gross, R. et al. (2009). "Face de-identification." Protection of
//!   Privacy in Surveillance Systems. Springer. (composite-face approach)
#![warn(missing_docs)]

pub mod blur;
pub mod dp;
pub mod kanon;

pub use blur::{blur_image, pixelate_image};
pub use dp::{DpEmbeddingNoise, DpError};
pub use kanon::{KAnonDeident, KAnonError};
