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

//! abis-standards-icao crate.
#![warn(missing_docs)]
pub mod bac;
pub mod dg1;
pub mod dg14;
pub mod dg15;
pub mod dg2;
pub mod dg3;
pub mod dg4;
pub mod error;
pub mod sod;

pub use error::IcaoError;
