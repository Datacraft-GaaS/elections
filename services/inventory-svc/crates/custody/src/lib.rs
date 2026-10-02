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

//! Event-sourced custody domain for M20 inventory management (WP-4.3).
#![warn(missing_docs)]
/// Domain aggregates for custody.
pub mod domain;
/// HTTP handlers for custody contract.
pub mod handlers;
/// HTTP router.
pub mod router;
/// Typed models for `custody/openapi.yaml`.
pub mod spec;
/// Postgres-backed application state (no in-memory). `Store::Mem` only behind `#[cfg(test)]`.
pub mod state;
/// Postgres + memory store (Store enum).
pub mod store;
/// Domain events.
pub mod events;
/// Geo RBAC helpers.
pub mod geo_rbac;
/// Merkle anchoring.
pub mod merkle;
/// RFID integration.
pub mod rfid;
