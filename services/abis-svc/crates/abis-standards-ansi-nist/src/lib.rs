//! ANSI/NIST-ITL 1-2011 (NIST SP 500-290 e3) transaction codec.
//!
//! Provides typed structs for record types 1-22, 98, and 99, a binary
//! parser/serialiser for complete transactions, and CBEFF wrapping via
//! the Type-99 record.
//!
//! # Quick start
//!
//! ```rust
//! use abis_standards_ansi_nist::{
//!     NistTransaction, NistRecord, RecordType,
//!     types::TransactionInfo,
//! };
//!
//! let info = TransactionInfo {
//!     tot: "CAR".to_string(),
//!     dat: "20260101".to_string(),
//!     pry: 4,
//!     dai: "DAI001".to_string(),
//!     ori: "ORI001".to_string(),
//!     tcn: "TCN0000001".to_string(),
//! };
//! let mut rec = NistRecord::new(RecordType::Type1, 0);
//! rec.fields = info.to_fields();
//!
//! let mut tx = NistTransaction::new();
//! tx.push_record(rec);
//!
//! let bytes = tx.serialize();
//! let tx2 = NistTransaction::parse(&bytes).expect("round-trip");
//! assert_eq!(tx, tx2);
//! ```
#![warn(missing_docs)]

pub mod error;
pub mod record;
pub mod transaction;
pub mod types;

pub use error::NistError;
pub use record::{NistField, NistRecord, RecordType};
pub use transaction::NistTransaction;
