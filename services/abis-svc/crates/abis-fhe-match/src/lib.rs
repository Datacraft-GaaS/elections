//! eel-abis-fhe-match: Homomorphic-encrypted biometric matching.
//!
//! Interface-only crate. No external FHE library is pulled in; a pure-Rust
//! toy implementation operates on integer polynomials with correct semantics
//! on small parameters, suitable for unit tests.
//!
//! Scheme support: CKKS (cosine similarity), BFV (Hamming distance), BGV, TFHE.

pub mod batch_1n;
pub mod context;
pub mod match_bfv;
pub mod match_ckks;
pub mod policy;

pub use context::{FheContext, FheScheme, SerializedContext};
pub use match_bfv::{BfvCiphertext, BfvMatcher, BfvMatchError};
pub use match_ckks::{CkksCiphertext, CkksMatcher, CkksMatchError};
pub use policy::{FhePolicy, FhePolicyError, FhePolicyOutcome};
