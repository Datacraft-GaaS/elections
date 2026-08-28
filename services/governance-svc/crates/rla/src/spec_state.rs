//! In-memory store for the public RLA contract surface.
//!
//! This is the spec-facing tracker (audit lifecycle, drawn samples, manual
//! counts) — a lighter surface than the internal BRAVO `Store`. A production
//! deployment backs it with the same governance-svc schema.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{ManualCount, RlaAudit, SampleBallot};

/// Shared spec-state handle.
#[derive(Default)]
pub struct SpecState {
    /// Audits by `audit_id`.
    pub audits: Mutex<HashMap<String, RlaAudit>>,
    /// Drawn samples by `audit_id`, in insertion order.
    pub samples: Mutex<HashMap<String, Vec<SampleBallot>>>,
    /// Manual counts by `audit_id`, in insertion order.
    pub counts: Mutex<HashMap<String, Vec<ManualCount>>>,
}

/// Shared spec-state handle threaded through the router.
pub type SharedSpecState = Arc<SpecState>;

impl SpecState {
    /// Construct an empty spec store.
    pub fn new() -> SharedSpecState {
        Arc::new(Self::default())
    }
}
