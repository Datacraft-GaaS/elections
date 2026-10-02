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

//! eel-abis-cert-fido: FIDO Biometric Component Certification harness.
//!
//! Generates certification evidence: PAD species coverage, PAI matrix,
//! FMR/FNMR report, and a tester signature stub.
#![warn(missing_docs)]

pub mod evidence;
pub mod matrix;

pub use evidence::{EvidenceError, EvidencePack};
pub use matrix::{PaiMatrix, PaiMatrixEntry, PaiMatrixError};
