//! In-memory store for referendum questions and county results.
//!
//! A production deployment replaces these registries with the governance
//! persistence layer (Postgres) plus a transactional outbox for the emitted
//! domain events; the handler surface is unchanged.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::explainer::ReferendumExplainer;
use crate::models::{ReferendumCountyResult, ReferendumQuestion};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Referendum questions by `question_id`.
    pub questions: Mutex<HashMap<String, ReferendumQuestion>>,
    /// County results by `referendum_id` (question_id), in insertion order.
    pub county_results: Mutex<HashMap<String, Vec<ReferendumCountyResult>>>,
    /// Referendum explainers by `question_id`.
    pub explainers: Mutex<HashMap<String, ReferendumExplainer>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
