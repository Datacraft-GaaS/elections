use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::DecisionLogEntry,
    pii_strip,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/entries", get(list_entries).post(create_entry))
}

async fn list_entries(
    State(state): State<AppState>,
) -> Json<Vec<DecisionLogEntry>> {
    let log = state.decision_log.read().await;
    Json(log.clone())
}

#[derive(Debug, Deserialize)]
struct CreateEntryPayload {
    model_id: String,
    model_version: String,
    interaction_type: String,
    input_text: String,
    confidence_score: f64,
    model_output: serde_json::Value,
    human_decision: Option<String>,
}

async fn create_entry(
    State(state): State<AppState>,
    Json(payload): Json<CreateEntryPayload>,
) -> (StatusCode, Json<DecisionLogEntry>) {
    let (stripped, _) = pii_strip::strip_pii(&payload.input_text);
    let hash = pii_strip::input_hash(&stripped);

    let entry = DecisionLogEntry {
        id: Uuid::now_v7().to_string(),
        model_id: payload.model_id,
        model_version: payload.model_version,
        interaction_type: payload.interaction_type,
        input_hash: hash,
        confidence_score: payload.confidence_score,
        model_output: payload.model_output,
        human_decision: payload.human_decision,
        decision_at: Utc::now(),
    };

    let mut log = state.decision_log.write().await;
    log.push(entry.clone());
    (StatusCode::CREATED, Json(entry))
}
