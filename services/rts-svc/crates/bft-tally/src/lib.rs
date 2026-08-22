//! eel-rts-bft-tally: HotStuff BFT consensus for tally aggregation.
//!
//! Implements the linear HotStuff protocol from:
//!   Yin, M., Malkhi, D., Reiter, M. K., Gueta, G. G., Abraham, I.
//!   "HotStuff: BFT Consensus with Linearity and Responsiveness."
//!   PODC 2019. ACM, pp. 347-356.
//!
//! Configuration: N=4 replicas, f=1 Byzantine fault tolerance.
//! Quorum size: 2f+1 = 3 signatures.
//!
//! Every published tally block requires a valid QuorumCertificate (QC).
//! The BftTallyAdapter drops in as a replacement for the Kafka-transactional
//! consumer used by rts-svc/crates/rts-tally.
//!
//! Fault-tolerance comparison:
//! | Property        | Kafka-transactional | HotStuff BFT         |
//! |-----------------|---------------------|----------------------|
//! | Safety          | Broker-level         | Byzantine (f faulty) |
//! | Liveness        | Kafka broker up      | f+1 correct replicas |
//! | Latency (happy) | O(1) Kafka round-trip| 3 message delays     |
//! | Leader failure  | Consumer restart     | View change          |
//! | Rewrite attack  | Possible at broker   | Cryptographically impossible |

pub mod adapter;
pub mod api;
pub mod certificate;
pub mod message;
pub mod pacemaker;
pub mod persistence;
pub mod replica;

pub use certificate::QuorumCertificate;
pub use message::{Block, HotStuffMessage, MessageType};
pub use replica::{Replica, ReplicaConfig, ReplicaId};

use thiserror::Error;

/// Top-level errors from this crate.
#[derive(Debug, Error)]
pub enum BftError {
	#[error("invalid quorum certificate: {0}")]
	InvalidQc(String),
	#[error("insufficient signatures: {got} of {required}")]
	InsufficientSignatures { got: usize, required: usize },
	#[error("view mismatch: expected {expected}, got {actual}")]
	ViewMismatch { expected: u64, actual: u64 },
	#[error("block hash mismatch")]
	HashMismatch,
	#[error("replica {id} is not the current leader")]
	NotLeader { id: u64 },
	#[error("liveness lost: too many replicas faulty ({faulty} > f={f})")]
	LivenessLost { faulty: usize, f: usize },
	#[error("signature error: {0}")]
	Signature(String),
	#[error("persistence error: {0}")]
	Persistence(String),
	#[error("adapter error: {0}")]
	Adapter(String),
}

/// Cluster constants: N=4, f=1, quorum=3.
pub const N: usize = 4;
pub const F: usize = 1;
pub const QUORUM: usize = 2 * F + 1; // 3
