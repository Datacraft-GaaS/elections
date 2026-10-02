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

//! `archive` HTTP service for M69 long-term-archive (WP-5.6).
//!
//! Manages archival collections, ingest jobs, preservation attestations,
//! research access requests, destruction certificates, and citation records
//! for support-admin-svc.
//!
//! WORM storage: MinIO Object Lock is configured via bucket policy (s3:PutObjectLegalHold,
//! COMPLIANCE mode, 15-year retention). This is enforced at the storage layer and is not
//! modeled in this domain.
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod handlers;
pub mod routes;
