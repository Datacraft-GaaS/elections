use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::models::{DiagnosticAction, DiagnosticActionType};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunDiagnosticRequest {
    kit_serial: String,
    action_type: DiagnosticActionType,
    engineer_id: Uuid,
    kit_consent_recorded: bool,
}

pub async fn run_diagnostic(
    state: State<AppState>,
    Path(ticket_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    // A diagnostic must attach to an existing ticket.
    let ticket_exists = state
        .tickets
        .lock()
        .ok()
        .map(|store| store.iter().any(|t| t.id == ticket_id))
        .unwrap_or(false);
    if !ticket_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "ticket not found" })),
        );
    }

    let req: RunDiagnosticRequest = match serde_json::from_value(body) {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid diagnostic request" })),
            )
        }
    };

    let action = DiagnosticAction {
        id: Uuid::new_v4(),
        ticket_id,
        kit_serial: req.kit_serial,
        action_type: req.action_type,
        engineer_id: req.engineer_id,
        kit_consent_recorded: req.kit_consent_recorded,
        signed_at: Utc::now(),
        result: None,
    };

    let Ok(mut store) = state.diagnostics.lock() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "store unavailable" })),
        );
    };
    store.push(action.clone());

    (
        StatusCode::CREATED,
        Json(serde_json::to_value(action).unwrap_or_default()),
    )
}
