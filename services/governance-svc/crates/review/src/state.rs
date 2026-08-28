//! In-memory store for the public review contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{ElectionReview, ReviewFinding};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Reviews by `review_id`.
    pub reviews: Mutex<HashMap<String, ElectionReview>>,
    /// Findings by `review_id`, in insertion order.
    pub findings: Mutex<HashMap<String, Vec<ReviewFinding>>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
