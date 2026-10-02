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

//! Tamper-evident, encrypted-at-rest offline record store for BVR registration.
//!
//! A registration station must work with no network, hold citizen PII on
//! disk, and later prove to an auditor exactly where each datum came from. This
//! crate is the storage layer that makes those three things true at once.
//!
//! # Instance identity
//!
//! Every store has a UUIDv7 [`InstanceIdentity`] and an Ed25519 keypair. The
//! instance id is stamped into every record, so a datum recovered from any
//! station in a fleet can be traced to the exact kit that captured it. The
//! private key never leaves its [`KeyStore`].
//!
//! # Provenance
//!
//! Each record carries a [`Provenance`] block recording the platform's five-axis
//! audit reference: **where** (station and instance), **when** (UTC instant),
//! **who** (the enrolling officer), **why** (purpose), and **how** (workflow,
//! step, and capture method). This is the same 5-axis reference the rest of the
//! platform preserves, not a scheme invented for this store.
//!
//! # Tamper evidence, then tamper proofing
//!
//! Two independent mechanisms, because they fail differently:
//!
//! 1. **Hash chain.** Each record's [`Record::this_hash`] commits to its
//!    predecessor, so altering record *N* invalidates every hash from *N*
//!    onward. [`verify_chain`] detects modification, deletion, and reordering.
//! 2. **Per-record signature.** The record hash is signed with the instance
//!    key, so an edit that leaves the signature intact still fails
//!    verification. This is what turns "we noticed" into "we can prove it".
//!
//! On sync, a batch is reduced to a Merkle root ([`Batch::merkle_root`]). Once
//! the server anchors that root, tampering is provable by a third party who
//! holds the published root, not merely by whoever holds the kit.
//!
//! Nothing here defends against an attacker who has both the store and the
//! private key. That is a hardware problem, and the key belongs in a TPM or HSM
//! per ADR-0026; the [`KeyStore`] trait is the seam for that.
//!
//! # Encryption
//!
//! Records are sealed with AES-256-GCM ([`eel_crypto_rs::aead`]) under a
//! per-record data key, with the record's identity bound in as associated data
//! so a ciphertext cannot be moved between records or between stations.

#![warn(missing_docs)]

pub mod identity;
pub mod keystore;
pub mod provenance;
pub mod record;
pub mod store;
pub mod sync;

pub use identity::InstanceIdentity;
pub use keystore::{KeyStore, SoftwareKeyStore};
pub use provenance::{CaptureMethod, Provenance, Purpose};
pub use record::{Record, RecordId, TamperError};
pub use store::{Store, StoreError};
pub use sync::{Batch, SyncState};
