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

//! Authentication and authorization primitives for the eel platform.
//!
//! - JWT verification against Keycloak JWKS with kid-miss refetch
//! - Subject extraction with roles and purpose scopes
//! - SpiceDB ReBAC check with in-memory cache and fail-closed semantics
//! - Audit event emission via the `AuditSink` trait
#![warn(missing_docs)]

pub mod audit;
pub mod error;
pub mod subject;

#[cfg(feature = "keycloak")]
pub mod jwks;

#[cfg(feature = "spicedb")]
pub mod spicedb;

pub use audit::{AuditEvent, AuditSink, LogSink, NoopSink};
pub use error::AuthError;
pub use subject::Subject;

#[cfg(feature = "keycloak")]
pub use jwks::JwksVerifier;

#[cfg(feature = "spicedb")]
pub use spicedb::{Consistency, Decision, SpiceDbClient};
