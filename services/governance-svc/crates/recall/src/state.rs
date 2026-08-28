//! In-memory store for the public recall contract.
//!
//! A production deployment replaces these registries with the governance
//! persistence layer (Postgres) plus a transactional outbox; the handler
//! surface is unchanged.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::Petition;

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Recall petitions by `petition_id`.
    pub petitions: Mutex<HashMap<String, Petition>>,
    /// Accepted signature counts by `petition_id`.
    pub signatures: Mutex<HashMap<String, u64>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
