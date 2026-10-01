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
    /// Event batches accepted by the anomaly ingest endpoint, by batch id.
    ///
    /// `ingest_event_batch` answered `{"accepted": true}` for a batch it
    /// discarded, so nothing was ever ingested and the count was unknowable.
    pub ingested_batches: eel_memstore_rs::Collection<serde_json::Value>,
    /// Forecast accuracy reports, by report id.
    pub accuracy_reports: eel_memstore_rs::Collection<serde_json::Value>,
    /// Model deployments, by deployment id.
    pub deployments: eel_memstore_rs::Collection<serde_json::Value>,
}

pub fn app() -> Router {
    stateful_app(AppState::default())
}

/// The same router over caller-supplied state.
///
/// `app()` is the convenience constructor; tests that need to seed alerts,
/// forecasts or models need to supply them, and there is no other way in because
/// the state is created inside `app()`.
pub fn stateful_app(state: AppState) -> Router {
    Router::new()
        .nest("/v1/ai", routes::router())
        .merge(spec::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
