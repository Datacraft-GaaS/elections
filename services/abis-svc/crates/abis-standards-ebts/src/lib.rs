//! FBI EBTS 10.0 transaction codec.
//!
//! Provides typed structs for all 13 defined Type-of-Transaction (TOT) codes,
//! a binary serialiser/parser for complete EBTS transactions and responses,
//! and IDC-linked record management.
//!
//! # Quick start
//!
//! ```rust
//! use abis_standards_ebts::{EbtsTransaction, TypeOfTransaction};
//!
//! let mut tx = EbtsTransaction::build(
//!     TypeOfTransaction::CAR,
//!     "DAI001",
//!     "ORI001",
//!     "TCN0000001",
//! );
//! tx.add_record(1, 4, vec![0xDE, 0xAD, 0xBE, 0xEF]);
//!
//! let bytes = tx.serialize();
//! let tx2 = EbtsTransaction::parse(&bytes).expect("round-trip");
//! assert_eq!(tx, tx2);
//! ```

pub mod error;
pub mod response;
pub mod tot;
pub mod transaction;

pub use error::EbtsError;
pub use response::{EbtsResponse, ResponseStatus};
pub use tot::TypeOfTransaction;
pub use transaction::{EbtsRecord, EbtsTransaction};
