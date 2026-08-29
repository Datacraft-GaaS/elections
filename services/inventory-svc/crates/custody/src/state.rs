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

use eel_spec_helpers_rs::SharedMap;

use crate::spec::{Asset, CustodyTransfer, ReconcileReport, Reserve, Seal, SealBreak};

/// Shared application state.
///
/// Uses [`SharedMap`] (thin wrapper over `Arc<Mutex<HashMap>>`) from
/// `eel-spec-helpers-rs` to deduplicate the `Arc<Mutex<HashMap>>` boilerplate
/// duplicated across ~21 handler state files (TD-03 / TD-08 exemplar).
#[derive(Clone, Default)]
pub struct AppState {
    /// assets.
    pub assets: SharedMap<String, Asset>,
    /// transfers.
    pub transfers: SharedMap<String, Vec<CustodyTransfer>>,
    /// seals.
    pub seals: SharedMap<String, Seal>,
    /// seal_breaks.
    pub seal_breaks: SharedMap<String, Vec<SealBreak>>,
    /// reserves.
    pub reserves: SharedMap<String, Reserve>,
    /// reconciles.
    pub reconciles: SharedMap<String, ReconcileReport>,
}

impl AppState {
    /// new.
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}
