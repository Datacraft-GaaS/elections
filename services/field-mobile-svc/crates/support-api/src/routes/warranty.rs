use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::models::WarrantyClaim;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WarrantyClaimRequest {
    kit_serial: String,
    oem_code: String,
    fault_description: String,
    engineer_id: Uuid,
}

pub async fn submit_claim(
    state: State<AppState>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let req: WarrantyClaimRequest = match serde_json::from_value(body) {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid warranty claim" })),
            )
        }
    };

    let claim = WarrantyClaim {
        id: Uuid::new_v4(),
        kit_serial: req.kit_serial,
        oem_code: req.oem_code,
        fault_description: req.fault_description,
        claim_status: String::from("PENDING"),
        engineer_id: req.engineer_id,
        submitted_at: Utc::now(),
    };

    let Ok(mut store) = state.warranty_claims.lock() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "store unavailable" })),
        );
    };
    store.push(claim.clone());

    (
        StatusCode::CREATED,
        Json(serde_json::to_value(claim).unwrap_or_default()),
    )
}
