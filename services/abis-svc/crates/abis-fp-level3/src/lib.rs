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

//! Level-3 fingerprint feature extraction and matching.
//!
//! Level-3 features include sweat pores, ridge edge contours, incipient ridges,
//! and require image resolution >= 500 ppi for reliable extraction.
//!
//! Reference: A. K. Jain, Y. Chen, M. Demirkus, "Pores and Ridges: High-Resolution
//! Fingerprint Matching Using Level 3 Features," IEEE Trans. Pattern Anal. Mach.
//! Intell., vol. 29, no. 1, pp. 15-27, Jan. 2007.
#![warn(missing_docs)]

pub mod incipient;
pub mod matcher;
pub mod pores;
pub mod quality;
pub mod ridge_edge;

pub use incipient::IncipientRidge;
pub use matcher::Level3Matcher;
pub use pores::Pore;
pub use quality::{Level3Quality, Level3QualityError};
pub use ridge_edge::EdgePoint;
