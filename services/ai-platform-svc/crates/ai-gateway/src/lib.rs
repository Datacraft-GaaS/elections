pub mod error;
pub mod models;
pub mod pii_strip;
pub mod routes;
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
