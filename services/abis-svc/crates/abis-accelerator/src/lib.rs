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

//! `eel-abis-accelerator`: GPU/FPGA offload interfaces and CPU fallback for biometric matching.
//!
//! Trait-based interfaces for hardware accelerators. The dispatcher selects an accelerator
//! based on gallery size, latency budget, and energy budget.
#![warn(missing_docs)]

pub mod cpu_fallback;
pub mod dispatcher;
pub mod domain;
pub mod error;
pub mod fpga;
pub mod gpu;

pub use cpu_fallback::CpuFallbackMatcher;
pub use domain::{AcceleratorCapabilities, AcceleratorKind, MatchQuery, Score};
pub use error::AcceleratorError;
pub use fpga::FpgaMatcher;
pub use gpu::GpuMatcher;
