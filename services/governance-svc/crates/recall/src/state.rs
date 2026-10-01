//! In-memory store for the public recall contract.
//!
//! A production deployment replaces these registries with the governance
//! persistence layer (Postgres) plus a transactional outbox; the handler
//! surface is unchanged.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eel_memstore_rs::Collection;

use crate::models::{RecallElection, SignatoryRecord};
use crate::spec::Petition;

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Recall petitions by `petition_id`.
    pub petitions: Mutex<HashMap<String, Petition>>,
    /// Accepted signature counts by `petition_id`.
    pub signatures: Mutex<HashMap<String, u64>>,
    /// Individual signatory records by record id, so threshold eligibility can
    /// be computed from real rows rather than a constant.
    pub signatories: Collection<SignatoryRecord>,
    /// Recall elections triggered from petitions, by election id.
    pub elections: Collection<RecallElection>,
    /// Registered voters per `constituency_code`.
    ///
    /// This is the electorate the article IV section 2 threshold is evaluated
    /// against. It used to be the literal 100_000 inside a handler. There is no
    /// roll-loading endpoint yet, so a petition whose constituency has no entry
    /// here is refused a threshold report rather than measured against a guess.
    pub roll: Mutex<HashMap<String, u64>>,
}

impl AppState {
    /// Register the electorate for a constituency.
    pub fn set_roll(&self, constituency_code: &str, registered_voters: u64) {
        self.roll
            .lock()
            .expect("recall roll lock poisoned")
            .insert(constituency_code.to_string(), registered_voters);
    }

    /// Registered voters for a constituency, if the roll has been loaded.
    pub fn roll_size(&self, constituency_code: &str) -> Option<u64> {
        self.roll
            .lock()
            .expect("recall roll lock poisoned")
            .get(constituency_code)
            .copied()
    }
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
