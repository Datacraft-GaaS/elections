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

//! inventory-api — HTTP API surface for inventory-svc.
//!
//! Exposes the custody (M20), transport & security, and warehouse/pallet/
//! incident domains over REST. [`store::Store`] is an enum over Postgres
//! (sqlx) and an in-memory fake, so the service runs without a database and
//! is fully testable. [`auth::Auth`] adds JWT + SpiceDB authorization when
//! configured via the environment.
#![warn(missing_docs)]

pub mod auth;
pub mod handlers;
pub mod router;
pub mod spec;
pub mod store;
