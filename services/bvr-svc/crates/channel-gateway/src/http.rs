//! USSD/IVR/WhatsApp channel gateway webhooks.

use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ussd/sessions", post(handle_ussd_session))
        .route("/ussd/sessions/{session_id}", get(get_ussd_session_log))
        .route("/ivr/voice", post(handle_voice_call))
        .route("/ivr/sessions", post(handle_ivr_session))
        .route("/webhook/whatsapp", get(verify_webhook).post(receive_message))
        .route("/whatsapp/webhook", get(verify_whatsapp_webhook).post(receive_whatsapp_webhook))
}

async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

async fn handle_ussd_session(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "response_text": "END", "action": "end", "request": req }))
}

async fn get_ussd_session_log(Path(session_id): Path<String>) -> impl IntoResponse {
    Json(json!({ "session_id": session_id, "log": [] }))
}

async fn handle_voice_call(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "response_text": "END", "request": req }))
}

async fn handle_ivr_session(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "response_text": "END", "request": req }))
}

async fn verify_webhook() -> impl IntoResponse {
    Json(json!({ "challenge": "verified" }))
}

async fn receive_message(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "acknowledged": true, "request": req }))
}

async fn receive_whatsapp_webhook(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "acknowledged": true, "request": req }))
}

async fn verify_whatsapp_webhook() -> impl IntoResponse {
    Json(json!({ "challenge": "verified" }))
}
