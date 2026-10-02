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

//! Latent biometric modality for the EEL ABIS system.
//!
//! Provides ANSI/NIST-ITL Type-13 parsing, quality-weighted partial print
//! matching, chain-of-custody management, and quality scoring for latent prints.
#![warn(missing_docs)]

pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use matcher::Score;
