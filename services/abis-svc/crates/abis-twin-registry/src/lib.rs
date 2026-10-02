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

//! `eel-abis-twin-registry`: twin group management with civil-registry sync.
//!
//! Identical twins score in an ambiguous zone during 1:N matching. This crate
//! manages twin groups, detects twin candidates, and cross-references civil registry.
#![warn(missing_docs)]

pub mod api;
pub mod detector;
pub mod domain;
pub mod error;
pub mod registry;

pub use domain::{TwinCandidate, TwinGroup, TwinMember, TwinVerificationStatus};
pub use error::TwinRegistryError;
