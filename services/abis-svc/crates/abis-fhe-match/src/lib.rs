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

//! eel-abis-fhe-match: Homomorphic-encrypted biometric matching.
//!
//! Interface-only crate. No external FHE library is pulled in; a pure-Rust
//! toy implementation operates on integer polynomials with correct semantics
//! on small parameters, suitable for unit tests.
//!
//! Scheme support: CKKS (cosine similarity), BFV (Hamming distance), BGV, TFHE.
#![warn(missing_docs)]

pub mod batch_1n;
pub mod context;
pub mod match_bfv;
pub mod match_ckks;
pub mod policy;

pub use context::{FheContext, FheScheme, SerializedContext};
pub use match_bfv::{BfvCiphertext, BfvMatcher, BfvMatchError};
pub use match_ckks::{CkksCiphertext, CkksMatcher, CkksMatchError};
pub use policy::{FhePolicy, FhePolicyError, FhePolicyOutcome};
