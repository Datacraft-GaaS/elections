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
#![warn(missing_docs)]

pub mod error;
pub mod response;
pub mod tot;
pub mod transaction;

pub use error::EbtsError;
pub use response::{EbtsResponse, ResponseStatus};
pub use tot::TypeOfTransaction;
pub use transaction::{EbtsRecord, EbtsTransaction};
