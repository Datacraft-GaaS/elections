use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::{ModelRecord, ModelRegistration},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/models", get(list_models).post(register_model))
        .route("/models/{id}", get(get_model))
        .route("/models/{id}/activate", patch(activate_model))
}

async fn list_models(
    State(state): State<AppState>,
) -> Json<Vec<ModelRecord>> {
    let records = state.models.read().await;
    Json(records.clone())
}

async fn get_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ModelRecord>, ApiError> {
    let records = state.models.read().await;
    records
        .iter()
        .find(|r| r.id == id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn register_model(
    State(state): State<AppState>,
    Json(reg): Json<ModelRegistration>,
) -> Result<(StatusCode, Json<ModelRecord>), ApiError> {
    let id = Uuid::now_v7().to_string();
    let record = ModelRecord {
        id,
        code: reg.code,
        name: reg.name,
        version: reg.version,
        architecture: reg.architecture,
        training_data_description: reg.training_data_description,
        eval_accuracy: reg.eval_accuracy,
        known_limitations: reg.known_limitations,
        human_in_loop: reg.human_in_loop,
        ccb_approval_id: reg.ccb_approval_id,
        iab_notified_at: reg.iab_notified_at,
        model_card_url: reg.model_card_url,
        registered_at: Utc::now(),
        is_active: false,
    };
    let mut records = state.models.write().await;
    records.push(record.clone());
    Ok((StatusCode::CREATED, Json(record)))
}

async fn activate_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ModelRecord>, ApiError> {
    let mut records = state.models.write().await;
    let record = records
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or(ApiError::NotFound)?;
    record.is_active = true;
    Ok(Json(record.clone()))
}
