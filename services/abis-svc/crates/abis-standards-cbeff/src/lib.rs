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

//! ISO/IEC 19785-1 CBEFF v3 container codec.
//!
//! Provides a binary parser and serialiser for Common Biometric Exchange
//! Formats Framework (CBEFF) containers, including nested container support
//! and a format-owner registry.
//!
//! # Quick start
//!
//! ```rust
//! use abis_standards_cbeff::{
//!     CbeffContainer, BiometricDataBlock, StandardBiometricHeader,
//! };
//!
//! let sbh = StandardBiometricHeader {
//!     format_owner: 0x0101,
//!     format_type:  0x0007,
//!     creation_date: *b"20260821",
//!     validity_period: None,
//!     biometric_type: 0x0000_0008,
//!     biometric_subtype: 0,
//!     product_id: 1,
//!     capture_device_id: 2,
//!     quality_records: vec![],
//! };
//! let bdb = BiometricDataBlock::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
//! let container = CbeffContainer::new(sbh, bdb, None);
//!
//! let bytes = container.serialize();
//! let parsed = CbeffContainer::parse(&bytes).expect("round-trip");
//! assert_eq!(parsed, container);
//! ```
#![warn(missing_docs)]

pub mod bdb;
pub mod container;
pub mod error;
pub mod registry;
pub mod sb;
pub mod sbh;

pub use bdb::BiometricDataBlock;
pub use container::{CbeffContainer, CBEFF_MAGIC};
pub use error::CbeffError;
pub use registry::FormatRegistry;
pub use sb::SecurityBlock;
pub use sbh::{QualityRecord, StandardBiometricHeader};
