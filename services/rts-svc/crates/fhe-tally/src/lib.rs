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

//! eel-rts-fhe-tally: CKKS homomorphic tally over encrypted Form34A leaves.
//!
//! Implements encrypted aggregation of tally counts so the running total can
//! be computed by the aggregator without ever seeing plaintext counts.
//! Decryption requires M-of-N threshold key shares (Shamir secret sharing,
//! 3-of-5 default matching HSM ceremony configuration).
//!
//! Scheme: CKKS (approximate arithmetic over floating-point data).
//! Reference:
//!   Cheon, J. H., Kim, A., Kim, M., Song, Y.
//!   "Homomorphic Encryption for Arithmetic of Approximate Numbers."
//!   ASIACRYPT 2017. LNCS 10624, pp. 409-437.
//!
//! Toy implementation (pure-Rust integer arithmetic) is used here and in CI.
//! Production must swap CkksContext for a library binding (e.g., SEAL, Zama
//! Concrete) at a polynomial_degree >= 8192 and security_level >= 128 bits.
//!
//! Performance caveat: at prod parameters (degree 8192, 128-bit security)
//! each homomorphic addition takes ~0.5ms on a Cortex-X2. Full 47,500-station
//! tally aggregation: ~24 seconds single-threaded; ~3 seconds with 8 threads.
//! Reference: SEAL benchmarks, Microsoft Research 2024.
#![warn(missing_docs)]

pub mod aggregate;
pub mod decrypt;
pub mod encrypt_leaf;
pub mod policy;
pub mod prove;

pub use aggregate::{aggregate_encrypted_leaves, EncryptedTally};
pub use decrypt::{decrypt_tally, ThresholdKey, ThresholdShare};
pub use encrypt_leaf::{encrypt_34a_leaf, EncryptedLeaf, Form34ALeaf};
pub use policy::{FhePolicy, FhePolicyError};

use thiserror::Error;

/// Top-level errors for this crate.
#[derive(Debug, Error)]
pub enum FheTallyError {
	#[error("policy violation: {0}")]
	Policy(#[from] FhePolicyError),
	#[error("encryption error: {0}")]
	Encryption(String),
	#[error("aggregation error: {0}")]
	Aggregation(String),
	#[error("decryption error: {0}")]
	Decryption(String),
	#[error("insufficient shares: {got} of {required}")]
	InsufficientShares { got: usize, required: usize },
	#[error("proof verification failed: {0}")]
	ProofVerification(String),
	#[error("tampered ciphertext detected")]
	TamperedCiphertext,
}
