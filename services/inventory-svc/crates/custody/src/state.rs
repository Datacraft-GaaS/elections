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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{Asset, CustodyTransfer, ReconcileReport, Reserve, Seal, SealBreak};

/// Shared application state.
#[derive(Clone, Default)]
pub struct AppState {
    /// assets.
    pub assets: Arc<Mutex<HashMap<String, Asset>>>,
    /// transfers.
    pub transfers: Arc<Mutex<HashMap<String, Vec<CustodyTransfer>>>>,
    /// seals.
    pub seals: Arc<Mutex<HashMap<String, Seal>>>,
    /// seal_breaks.
    pub seal_breaks: Arc<Mutex<HashMap<String, Vec<SealBreak>>>>,
    /// reserves.
    pub reserves: Arc<Mutex<HashMap<String, Reserve>>>,
    /// reconciles.
    pub reconciles: Arc<Mutex<HashMap<String, ReconcileReport>>>,
}

impl AppState {
    /// new.
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}
