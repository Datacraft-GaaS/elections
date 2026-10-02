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

//! eel-rfid-impinj: Impinj Speedway R700 vendor adapter.
//!
//! This crate provides:
//! - `adapter::ImpinjAdapter` — HTTP client wrapping the py/rfid-reader-driver endpoint
//!   with circuit-breaker resilience via eel-resilience-rs.
//! - `double::MockImpinj` — in-memory double for dev/chaos testing with tamper injection.
#![warn(missing_docs)]

pub mod adapter;
pub mod double;

pub use adapter::ImpinjHandler;
pub use double::MockImpinj;
