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

//! CDEFFS: Extended Feature Set for fingerprint records.
//!
//! Reference: NIST SP 500-290 e3 Annex A, "CDEFFS -- Candidate Definition of
//! Extended Feature Set for Fingerprints"; ANSI/NIST-ITL 1-2011 Type-9 fields 300-999.
#![warn(missing_docs)]

pub mod features;
pub mod matcher;
pub mod parse;
pub mod serialise;

pub use features::{
    ExtendedFeatureSet, PatternClass, RidgeQualityMap, SingularPoint, SingularPointKind,
};
pub use matcher::CdeffsMatcher;
pub use parse::parse_efs;
pub use serialise::serialise_efs;
