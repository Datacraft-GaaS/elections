//! In-memory store for the public custody contract surface.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{Asset, CustodyTransfer, ReconcileReport, Reserve, Seal, SealBreak};

/// Shared application state.
#[derive(Clone, Default)]
pub struct AppState {
    pub assets: Arc<Mutex<HashMap<String, Asset>>>,
    pub transfers: Arc<Mutex<HashMap<String, Vec<CustodyTransfer>>>>,
    pub seals: Arc<Mutex<HashMap<String, Seal>>>,
    pub seal_breaks: Arc<Mutex<HashMap<String, Vec<SealBreak>>>>,
    pub reserves: Arc<Mutex<HashMap<String, Reserve>>>,
    pub reconciles: Arc<Mutex<HashMap<String, ReconcileReport>>>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}
