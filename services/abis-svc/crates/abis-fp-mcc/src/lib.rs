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

//! Minutia Cylinder-Code (MCC) fingerprint matching.
//!
//! Reference: R. Cappelli, M. Ferrara, D. Maltoni, "Minutia cylinder-code: a new
//! representation and matching technique for fingerprint recognition," IEEE Trans.
//! Pattern Anal. Mach. Intell., vol. 32, no. 12, pp. 2128-2141, 2010.
#![warn(missing_docs)]

pub mod build;
pub mod consolidate;
pub mod cylinder;
pub mod matcher;
pub mod params;
pub mod similarity;

pub use build::build_mcc;
pub use consolidate::global_similarity;
pub use cylinder::MinutiaCylinder;
pub use matcher::MccMatcher;
pub use params::MccParams;
pub use similarity::local_similarity;
