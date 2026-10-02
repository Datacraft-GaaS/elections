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

//! eel-kit-cert-transparency: CT-log-style transparency for kit certificate issuance.
//!
//! Implements RFC 6962 Certificate Transparency semantics specialised for
//! field-kit (SPIRE + step-ca) certificate lifecycle in the eel election
//! integrity platform.
//!
//! Key design decisions:
//! - Every kit cert is appended to the log BEFORE delivery. No cert exists
//!   outside the log.
//! - Merkle structure uses eel-crypto-rs primitives (SHA-256, RFC 6962
//!   0x00/0x01 domain separation).
//! - Kits verify their own cert inclusion at boot. Anomaly if cert exists but
//!   is absent from the log.
//! - The Signed Tree Head (STH) is signed externally (HSM or Ed25519 key).
//!
//! References:
//! - RFC 6962: Ben Laurie, Adam Langley, Emilia Kasper (2013)
//! - Google Trillian: https://github.com/google/trillian
#![warn(missing_docs)]

pub mod api;
pub mod entry;
pub mod issuance_hook;
pub mod log;
pub mod verifier;

pub use entry::KitCertEntry;
pub use issuance_hook::{HookError, IssuanceReceipt, IssuanceRequest};
pub use log::{InclusionProof, KitCertTransparencyLog, LogError, SignedTreeHead};
pub use verifier::{BootVerifyResult, VerifierError};
