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

use std::sync::{Arc, Mutex};

use crate::models::{WorkerAssignment, WorkerCheckIn, WorkerCheckOut, WorkerStatusPing};
use crate::spec::DispatchAssignment;

/// In-memory store for the dispatch service (M33).
///
/// Real deployments replace this with a Postgres-backed store; the handlers
/// only depend on the `AppState` surface so the swap is internal.
#[derive(Clone, Default)]
pub struct AppState {
    pub check_ins: Arc<Mutex<Vec<WorkerCheckIn>>>,
    pub check_outs: Arc<Mutex<Vec<WorkerCheckOut>>>,
    pub pings: Arc<Mutex<Vec<WorkerStatusPing>>>,
    pub assignments: Arc<Mutex<Vec<WorkerAssignment>>>,
    pub spec_assignments: Arc<Mutex<std::collections::HashMap<String, DispatchAssignment>>>,
}
