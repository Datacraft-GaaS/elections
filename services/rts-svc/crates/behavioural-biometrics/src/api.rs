//! GET /v1/behavioural/session/{id}/features
//! Staff can review flagged sessions.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tokio::sync::RwLock;

use crate::session::SessionProfile;

#[derive(Debug, Serialize)]
pub struct SessionFeaturesResponse {
    pub session_id: String,
    pub officer_id: String,
    pub baseline_established: bool,
    pub last_distance: Option<f64>,
    pub sample_count: usize,
    pub flagged: bool,
}

pub struct AppState {
    pub profiles: Arc<RwLock<HashMap<String, SessionProfile>>>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/v1/behavioural/session/{session_id}/features",
            get(get_session_features),
        )
        .with_state(state)
}

async fn get_session_features(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let profiles = state.profiles.read().await;
    match profiles.get(&session_id) {
        None => StatusCode::NOT_FOUND.into_response(),
        Some(profile) => {
            let flagged = profile
                .last_distance
                .map(|d| d > 5.0)
                .unwrap_or(false);
            let response = SessionFeaturesResponse {
                session_id: profile.session_id.clone(),
                officer_id: profile.officer_id.clone(),
                baseline_established: profile.baseline.is_some(),
                last_distance: profile.last_distance,
                sample_count: profile.sample_count,
                flagged,
            };
            Json(response).into_response()
        }
    }
}
