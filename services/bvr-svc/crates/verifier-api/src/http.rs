//! Public voter verification handlers (verifier/verify.yaml + ussd + whatsapp).

use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/verifier/lookups", post(lookup_voter_registration))
        .route("/api/v1/verifier/station/{station_code}", get(get_polling_station_detail))
        .route("/api/v1/verifier/results/summary", get(get_result_summary))
        .route("/api/v1/verifier/ussd/session", post(handle_ussd_session))
        .route("/api/v1/verifier/whatsapp/webhook", get(verify_whatsapp_webhook).post(handle_whatsapp_webhook))
}

async fn lookup_voter_registration(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "registered": false, "request": req }))
}

async fn get_polling_station_detail(Path(station_code): Path<String>) -> impl IntoResponse {
    Json(json!({ "station_code": station_code, "name": null, "constituency": null }))
}

async fn get_result_summary() -> impl IntoResponse {
    Json(json!({ "results": [], "generated_at": chrono::Utc::now().to_rfc3339() }))
}

async fn handle_ussd_session(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "response_text": "END", "action": "end", "request": req }))
}

async fn handle_whatsapp_webhook(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "acknowledged": true, "request": req }))
}

async fn verify_whatsapp_webhook() -> impl IntoResponse {
    Json(json!({ "challenge": "verified" }))
}
