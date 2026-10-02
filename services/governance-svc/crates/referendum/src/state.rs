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

//! In-memory store for referendum questions and county results.
//!
//! A production deployment replaces these registries with the governance
//! persistence layer (Postgres) plus a transactional outbox for the emitted
//! domain events; the handler surface is unchanged.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::explainer::ReferendumExplainer;
use eel_memstore_rs::Collection;

use crate::models::{Referendum, ReferendumCountyResult, ReferendumQuestion};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Referendum questions by `question_id`.
    pub questions: Mutex<HashMap<String, ReferendumQuestion>>,
    /// County results by `referendum_id` (question_id), in insertion order.
    pub county_results: Mutex<HashMap<String, Vec<ReferendumCountyResult>>>,
    /// Referendum explainers by `question_id`.
    pub explainers: Mutex<HashMap<String, ReferendumExplainer>>,
    /// Configured referendums by `id`, so list and get read real records
    /// rather than echoing whatever id was asked for.
    pub referendums: Collection<Referendum>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
