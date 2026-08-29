//! Zero-knowledge proof of correct tally aggregation for the eel platform.
//!
//! Implements a Groth16 SNARK (Succinct Non-interactive ARgument of Knowledge)
//! over BLS12-381 proving that the sum of station-level vote counts (34A leaves)
//! equals the published national tally, without revealing individual station tallies.
//!
//! # Algorithm citations
//!
//! - Groth, Jens. "On the Size of Pairing-Based Non-Interactive Arguments."
//!   CRYPTO 2016. Springer LNCS 9666, pp. 305-326.
//!   <https://eprint.iacr.org/2016/260>
//!
//! - Bunz, B., Bootle, J., Boneh, D., Poelstra, A., Wuille, P., Maxwell, G.
//!   "Bulletproofs: Short Proofs for Confidential Transactions and More."
//!   IEEE Symposium on Security and Privacy 2018.
//!   <https://eprint.iacr.org/2017/1066>
//!
//! # Proof size
//!
//! A Groth16 proof over BLS12-381 serialises to 192 bytes
//! (2 G1 points + 1 G2 point). The `serialise` module compresses it
//! to ~192 bytes suitable for embedding in the portal and verifier-rs.
//!
//! # Usage
//!
//! ```no_run
//! use zk_tally::{TallyLeaf, PublishedTally, setup, TallyProver, TallyVerifier};
//!
//! let leaves = vec![TallyLeaf { station_id: "KE-01-0001".into(), votes: 1234 }];
//! let published = PublishedTally { total_votes: 1234 };
//! let params = setup::generate_params(1).unwrap();
//! let proof = TallyProver::prove(&leaves, &published, &params).unwrap();
//! let vk = params.vk();
//! assert!(TallyVerifier::verify(&proof, &vk, &published).unwrap());
//! ```
#![warn(missing_docs)]

pub mod circuit;
pub mod error;
pub mod prover;
pub mod serialise;
pub mod setup;
pub mod verifier;

pub use circuit::{PublishedTally, TallyLeaf};
pub use error::ZkError;
pub use prover::{Groth16Proof, TallyProver};
pub use setup::TallyParams;
pub use verifier::TallyVerifier;
