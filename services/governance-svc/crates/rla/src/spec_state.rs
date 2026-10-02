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
