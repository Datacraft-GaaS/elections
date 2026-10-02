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

//! In-memory store for the public review contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eel_memstore_rs::Collection;

use crate::models::{Finding, LongDripAlert, ParliamentReferral, Recommendation};
use crate::spec::{ElectionReview, ReviewFinding};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Reviews by `review_id`.
    pub reviews: Mutex<HashMap<String, ElectionReview>>,
    /// Findings by `review_id`, in insertion order.
    pub findings: Mutex<HashMap<String, Vec<ReviewFinding>>>,
    /// Findings by finding id, so triage can update one. `models::Finding` is
    /// what `submit_finding` builds, so this is the same shape the caller saw.
    pub findings_by_id: Collection<Finding>,
    /// Recommendations by recommendation id.
    ///
    /// Nothing creates recommendations yet -- there is no endpoint for it, which
    /// is recorded in status.md rather than papered over by inventing one.
    pub recommendations: Collection<Recommendation>,
    /// Parliament referrals by referral id.
    pub referrals: Collection<ParliamentReferral>,
    /// Long-drip alerts by alert id.
    pub long_drip: Collection<LongDripAlert>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
