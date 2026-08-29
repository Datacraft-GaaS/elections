//! AI gateway aggregating registry, anomaly, forecast, and red-team APIs.
#![warn(missing_docs)]
/// Error types for AI gateway.
pub mod error;
/// Domain models.
pub mod models;
/// PII stripping utilities.
pub mod pii_strip;
/// Route definitions.
pub mod routes;
/// Typed spec handlers for public contract.
pub mod spec;

use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;

#[derive(Clone, Default)]
pub struct AppState {
    pub models: Arc<tokio::sync::RwLock<Vec<models::ModelRecord>>>,
    pub alerts: Arc<tokio::sync::RwLock<Vec<models::AnomalyAlert>>>,
    pub decision_log: Arc<tokio::sync::RwLock<Vec<models::DecisionLogEntry>>>,
    pub forecast_models: Arc<tokio::sync::RwLock<std::collections::HashMap<String, serde_json::Value>>>,
    pub redteam_campaigns: Arc<tokio::sync::RwLock<std::collections::HashMap<String, serde_json::Value>>>,
}

pub fn app() -> Router {
    let state = AppState::default();
    Router::new()
        .nest("/v1/ai", routes::router())
        .merge(spec::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
