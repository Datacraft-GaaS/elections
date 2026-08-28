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
