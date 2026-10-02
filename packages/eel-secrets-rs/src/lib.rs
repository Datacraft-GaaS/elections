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

//! OpenBao secrets client for the eel platform (ADR-0007).
//!
//! Provides:
//! - Dynamic PostgreSQL credentials via `PgDynamicCred`
//! - Transit encrypt/decrypt via `Transit`
//! - JIT admin credentials with 4h cap via `issue_admin_cred`
//! - PKI leaf cert issuance via `issue_leaf_cert`
#![warn(missing_docs)]

pub mod admin_creds;
pub mod dynamic_creds;
pub mod error;
pub mod pki;
pub mod transit;

pub use admin_creds::{AdminCred, issue_admin_cred};
pub use dynamic_creds::{PgCred, PgDynamicCred};
pub use error::SecretsError;
pub use pki::{LeafCert, issue_leaf_cert};
pub use transit::Transit;
