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

//! In-memory store for the public custody contract surface.
//!
//! # Swap boundary
//!
//! Replace `Mutex<HashMap>` with a DB-backed store when persistence lands:
//!
//! ```ignore
//! #[async_trait::async_trait]
//! pub trait CustodyStore: Send + Sync {
//!     async fn create_asset(&self, req: AssetCreate) -> Result<Asset, StoreError>;
//!     async fn get_asset(&self, id: &str) -> Result<Option<Asset>, StoreError>;
//!     // ... transfer, seal, reserve, reconcile ...
//! }
//! // Pg impl: `sqlx::query_as!(...)` against `custody_assets`, `custody_transfers`, etc.
//! // Mem impl: current `Mutex<HashMap>` (kept for hermetic tests, mirrors `rla` pattern).
//! ```
//!
//! ```sql
//! -- migrations/VXXX__custody_api.sql (sketch)
//! CREATE TABLE custody_assets (
//!   asset_id UUID PRIMARY KEY,
//!   asset_type TEXT NOT NULL,
//!   serial_number TEXT NOT NULL UNIQUE,
//!   current_custodian_id TEXT NOT NULL,
//!   current_location TEXT NOT NULL,
//!   status TEXT NOT NULL,
//!   condition TEXT NOT NULL,
//!   created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
//!   updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
//! );
//! -- + custody_transfers, custody_seals, custody_seal_breaks, custody_reserves, custody_reconciles
//! ```

use std::sync::Arc;

use sqlx::PgPool;

use crate::store::{MemStore, Store};

/// Shared application state — postgres-backed (no purely in-memory).
///
/// Production (`Store::Pg`) uses `PgPool` against `custody_api_*` tables
/// (`migrations/V052__custody_api_surface.sql`). Hermetic tests use
/// `Store::Mem` (SharedMap) behind `#[cfg(test)]` only.
#[derive(Clone)]
pub struct AppState {
    /// store.
    pub store: Store,
}

impl Default for AppState {
    fn default() -> Self {
        Self { store: Store::Mem(MemStore::default()) }
    }
}

impl AppState {
    /// new in-memory (tests, hermetic).
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// new from postgres pool (production).
    pub fn from_pool(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { store: Store::Pg(pool) })
    }
}
