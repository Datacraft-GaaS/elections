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

use crate::models::{DiagnosticAction, SupportTicket, WarrantyClaim};
use crate::spec::{Article, SupportTicket as SpecTicket};

/// In-memory store for the field-support service (M32).
///
/// Real deployments replace this with a Postgres-backed store; the handlers
/// only depend on the `AppState` surface so the swap is internal.
#[derive(Clone, Default)]
pub struct AppState {
    pub tickets: Arc<Mutex<Vec<SupportTicket>>>,
    pub diagnostics: Arc<Mutex<Vec<DiagnosticAction>>>,
    pub warranty_claims: Arc<Mutex<Vec<WarrantyClaim>>>,
    pub spec_tickets: Arc<Mutex<std::collections::HashMap<String, SpecTicket>>>,
    pub spec_articles: Arc<Mutex<std::collections::HashMap<String, Article>>>,
}
