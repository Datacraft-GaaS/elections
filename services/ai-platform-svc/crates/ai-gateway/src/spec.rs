//! Public contract handlers for the AI surface (ai-registry, ai-anomaly,
//! ai-forecast, ai-redteam specs).

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use crate::AppState;

/// Legacy `/v1/ai/*` → canonical `/api/v1/ai/*` permanent redirect.
async fn redirect_legacy(Path(tail): Path<String>) -> impl IntoResponse {
    use axum::http::header;
    (
        StatusCode::PERMANENT_REDIRECT,
        [(header::LOCATION, format!("/api/v1/ai/{tail}"))],
    )
}

pub fn router() -> Router<AppState> {
    Router::new()
        // Legacy redirect — deprecated `/v1/ai/*` → canonical `/api/v1/ai/*` (TD-06)
        .route("/v1/ai/{*tail}", get(redirect_legacy).post(redirect_legacy).put(redirect_legacy).patch(redirect_legacy).delete(redirect_legacy))
        // ai-registry/registry.yaml
        .route("/api/v1/ai/registry/models", get(list_models).post(register_model))
        .route("/api/v1/ai/registry/models/{model_id}", get(get_model))
        .route("/api/v1/ai/registry/models/{model_id}/deployments", post(deploy_model))
        .route("/api/v1/ai/registry/audit-log", get(list_audit_log))
        // ai-anomaly/detection.yaml
        .route("/api/v1/ai/anomaly/alerts", get(list_anomaly_alerts))
        .route("/api/v1/ai/anomaly/alerts/{alert_id}", get(get_anomaly_alert))
        .route("/api/v1/ai/anomaly/alerts/{alert_id}/adjudications", post(adjudicate_alert))
        .route("/api/v1/ai/anomaly/reports/summary", get(get_anomaly_summary_report))
        .route("/api/v1/ai/anomaly/ingest/events", post(ingest_event_batch))
        // ai-forecast/analytics.yaml
        .route("/api/v1/ai/forecast/turnout/national", get(get_national_turnout_forecast))
        .route("/api/v1/ai/forecast/turnout/county/{county_code}", get(get_county_turnout_forecast))
        .route("/api/v1/ai/forecast/turnout/constituency/{constituency_code}", get(get_constituency_turnout_forecast))
        .route("/api/v1/ai/forecast/reports", get(list_forecast_accuracy_reports))
        .route("/api/v1/ai/forecast/models/retrain", post(trigger_model_retrain))
        // ai-redteam/testing.yaml
        .route("/api/v1/ai/redteam/campaigns", get(list_redteam_campaigns).post(create_redteam_campaign))
        .route("/api/v1/ai/redteam/campaigns/{campaign_id}", get(get_redteam_campaign))
        .route("/api/v1/ai/redteam/campaigns/{campaign_id}/findings", post(add_redteam_finding))
        .route("/api/v1/ai/redteam/campaigns/{campaign_id}/complete", post(complete_redteam_campaign))
}

fn page(items: &Value) -> Value {
    eel_spec_helpers_rs::page_value(items)
}

// --- registry ---

async fn list_models(State(state): State<AppState>) -> impl IntoResponse {
    let records = state.models.read().await;
    let items: Vec<Value> = records.iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect();
    Json(page(&serde_json::to_value(items).unwrap()))
}

async fn get_model(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let records = state.models.read().await;
    match records.iter().find(|r| r.id == id) {
        Some(r) => Json(serde_json::to_value(r).unwrap()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "model not found" }))).into_response(),
    }
}

async fn register_model(State(state): State<AppState>, Json(req): Json<Value>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let v = json!({ "model_id": id, "status": "registered", "created_at": chrono::Utc::now().to_rfc3339(), "request": req });
    let mut m = state.models.write().await;
    let _ = &mut m;
    drop(m);
    (StatusCode::CREATED, Json(v))
}

async fn deploy_model(State(_state): State<AppState>, Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    let _ = id;
    (StatusCode::CREATED, Json(json!({ "deployment_id": uuid::Uuid::now_v7().to_string(), "status": "deployed", "request": req })))
}

async fn list_audit_log(State(state): State<AppState>) -> impl IntoResponse {
    let log = state.decision_log.read().await;
    let items: Vec<Value> = log.iter().map(|e| serde_json::to_value(e).unwrap_or(Value::Null)).collect();
    Json(page(&serde_json::to_value(items).unwrap()))
}

// --- anomaly ---

async fn list_anomaly_alerts(State(state): State<AppState>) -> impl IntoResponse {
    let alerts = state.alerts.read().await;
    let items: Vec<Value> = alerts.iter().map(|a| serde_json::to_value(a).unwrap_or(Value::Null)).collect();
    Json(page(&serde_json::to_value(items).unwrap()))
}

async fn get_anomaly_alert(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let alerts = state.alerts.read().await;
    match alerts.iter().find(|a| a.id == id) {
        Some(a) => Json(serde_json::to_value(a).unwrap()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "alert not found" }))).into_response(),
    }
}

async fn adjudicate_alert(State(_state): State<AppState>, Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    let _ = id;
    (StatusCode::CREATED, Json(json!({ "adjudication_id": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn get_anomaly_summary_report(State(state): State<AppState>) -> impl IntoResponse {
    let alerts = state.alerts.read().await;
    Json(json!({ "total_alerts": alerts.len(), "generated_at": chrono::Utc::now().to_rfc3339() }))
}

async fn ingest_event_batch(Json(req): Json<Value>) -> impl IntoResponse {
    (StatusCode::ACCEPTED, Json(json!({ "accepted": true, "batch": req })))
}

// --- forecast ---

async fn get_national_turnout_forecast(State(_s): State<AppState>) -> impl IntoResponse {
    Json(json!({ "forecast_pct": 0.0, "as_of": chrono::Utc::now().to_rfc3339() }))
}

async fn get_county_turnout_forecast(State(_s): State<AppState>, Path(code): Path<String>) -> impl IntoResponse {
    Json(json!({ "county_code": code, "forecast_pct": 0.0 }))
}

async fn get_constituency_turnout_forecast(State(_s): State<AppState>, Path(code): Path<String>) -> impl IntoResponse {
    Json(json!({ "constituency_code": code, "forecast_pct": 0.0 }))
}

async fn list_forecast_accuracy_reports(State(_s): State<AppState>) -> impl IntoResponse {
    Json(page(&json!([])))
}

async fn trigger_model_retrain(Json(req): Json<Value>) -> impl IntoResponse {
    (StatusCode::ACCEPTED, Json(json!({ "retrain_job_id": uuid::Uuid::now_v7().to_string(), "status": "queued", "request": req })))
}

// --- redteam ---

async fn create_redteam_campaign(State(state): State<AppState>, Json(req): Json<Value>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let v = json!({ "campaign_id": id, "status": "created", "created_at": chrono::Utc::now().to_rfc3339(), "request": req });
    state.redteam_campaigns.write().await.insert(id, v.clone());
    (StatusCode::CREATED, Json(v))
}

async fn list_redteam_campaigns(State(state): State<AppState>) -> impl IntoResponse {
    let c = state.redteam_campaigns.read().await;
    let items: Vec<Value> = c.values().cloned().collect();
    Json(page(&serde_json::to_value(items).unwrap()))
}

async fn get_redteam_campaign(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let c = state.redteam_campaigns.read().await;
    match c.get(&id) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "campaign not found" }))).into_response(),
    }
}

async fn add_redteam_finding(State(_s): State<AppState>, Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    let _ = id;
    (StatusCode::CREATED, Json(json!({ "finding_id": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn complete_redteam_campaign(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut c = state.redteam_campaigns.write().await;
    match c.get_mut(&id) {
        Some(v) => {
            v["status"] = json!("completed");
            Json(v.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "campaign not found" }))).into_response(),
    }
}
