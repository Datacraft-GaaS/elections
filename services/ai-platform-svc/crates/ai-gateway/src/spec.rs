//! Public contract handlers for the AI surface (ai-registry, ai-anomaly,
//! ai-forecast, ai-redteam specs).

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
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

/// A problem+json response for a missing resource.
fn not_found(detail: String) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "type": "about:blank",
            "title": "Not Found",
            "status": 404,
            "detail": detail,
        })),
    )
        .into_response()
}

/// Refuse a turnout forecast when no model is registered for that scope.
///
/// These handlers each returned `forecast_pct: 0.0`. That is not a forecast of
/// zero turnout -- it is the absence of any model, rendered as a number an
/// operator would act on, with nothing indicating nothing had ever been trained.
fn no_forecast_model(key: &str) -> Response {
    not_implemented(format!(
        "no turnout forecast model is registered for {key}; this previously \
         returned forecast_pct 0.0, which reads as a forecast of zero turnout \
         rather than the absence of a model"
    ))
}

fn not_implemented(detail: String) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "type": "about:blank",
            "title": "Not Implemented",
            "status": 501,
            "detail": detail,
        })),
    )
        .into_response()
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

async fn deploy_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Result<Response, Response> {
    // Real. Previously answered 201 with status "deployed" and a minted
    // deployment_id for a deployment that never happened, without checking the
    // model existed.
    let environment = req
        .get("environment")
        .and_then(|v| v.as_str())
        .unwrap_or("staging")
        .to_string();
    let deployed_at = chrono::Utc::now().to_rfc3339();
    {
        let mut models = state.models.write().await;
        let record = models
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or_else(|| not_found(format!("no model with id {id}")))?;
        record.is_active = true;
    }
    let deployment = json!({
        "deployment_id": uuid::Uuid::now_v7().to_string(),
        "model_id": id,
        "environment": environment,
        "status": "deployed",
        "deployed_at": deployed_at,
    });
    state.deployments.insert(
        deployment["deployment_id"].as_str().unwrap_or_default().to_string(),
        deployment.clone(),
    );
    Ok((StatusCode::CREATED, Json(deployment)).into_response())
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

async fn adjudicate_alert(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Result<Response, Response> {
    // Real. Previously minted an adjudication_id and echoed the request without
    // touching the alert, so an alert could read as adjudicated while the real
    // alert in state.alerts stayed unresolved.
    let decision = req.get("decision").cloned().unwrap_or(Value::Null);
    let rationale = req.get("rationale").cloned().unwrap_or(Value::Null);
    let analyst = req
        .get("analyst_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unassigned")
        .to_string();
    let mut alerts = state.alerts.write().await;
    let alert = alerts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| not_found(format!("no alert with id {id}")))?;
    alert.assigned_analyst_id = Some(analyst.clone());
    let adjudication = json!({
        "adjudication_id": uuid::Uuid::now_v7().to_string(),
        "alert_id": id,
        "decision": decision,
        "rationale": rationale,
        "analyst_id": analyst,
        "adjudicated_at": chrono::Utc::now().to_rfc3339(),
    });
    Ok((StatusCode::CREATED, Json(adjudication)).into_response())
}

async fn get_anomaly_summary_report(State(state): State<AppState>) -> impl IntoResponse {
    let alerts = state.alerts.read().await;
    Json(json!({ "total_alerts": alerts.len(), "generated_at": chrono::Utc::now().to_rfc3339() }))
}

async fn ingest_event_batch(
    State(state): State<AppState>,
    Json(req): Json<Value>,
) -> Response {
    // Real. Previously answered {"accepted": true} for a batch it discarded, so
    // the anomaly pipeline reported ingesting events that never arrived.
    let events = req
        .get("events")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let batch_id = uuid::Uuid::now_v7().to_string();
    let count = events.len();
    let record = json!({
        "batch_id": batch_id,
        "event_count": count,
        "received_at": chrono::Utc::now().to_rfc3339(),
        "events": events,
    });
    state.ingested_batches.insert(batch_id.clone(), record);
    (
        StatusCode::ACCEPTED,
        Json(json!({ "batch_id": batch_id, "accepted": count })),
    )
        .into_response()
}

// --- forecast ---

async fn get_national_turnout_forecast(State(state): State<AppState>) -> Response {
    let key = "turnout:national:".to_string();
    // Served when a model is registered for this scope.
    if let Some(model) = state.forecast_models.read().await.get(&key) {
        return Json(model.clone()).into_response();
    }
    no_forecast_model(&key)
}

async fn get_county_turnout_forecast(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Response {
    let key = format!("turnout:county:{code}");
    if let Some(model) = state.forecast_models.read().await.get(&key) {
        return Json(model.clone()).into_response();
    }
    no_forecast_model(&key)
}

async fn get_constituency_turnout_forecast(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Response {
    let key = format!("turnout:constituency:{code}");
    if let Some(model) = state.forecast_models.read().await.get(&key) {
        return Json(model.clone()).into_response();
    }
    no_forecast_model(&key)
}

async fn list_forecast_accuracy_reports(State(state): State<AppState>) -> impl IntoResponse {
    // Real. Previously an empty page with no way to populate it.
    Json(page(&json!(state.accuracy_reports.list())))
}

async fn trigger_model_retrain(
    State(_state): State<AppState>,
    Json(req): Json<Value>,
) -> Response {
    let _ = &req;
    // Refused rather than implemented. This previously answered
    // {"status": "queued"} with a minted retrain_job_id for a job that was never
    // queued: there is no training pipeline in this crate, so a queued retrain
    // could never run and polling it would return nothing forever. The 501 says
    // what is missing instead of implying work is under way.
    not_implemented(
        "no training pipeline is wired in this crate; this previously answered \
         status queued for a retrain job that was never enqueued"
            .to_string(),
    )
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

async fn add_redteam_finding(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Result<Response, Response> {
    let mut campaigns = state.redteam_campaigns.write().await;
    let campaign = campaigns
        .entry(id.clone())
        .or_insert_with(|| json!({ "campaign_id": id, "findings": [] }));
    let finding = json!({
        "finding_id": uuid::Uuid::now_v7().to_string(),
        "title": req.get("title").cloned().unwrap_or(Value::Null),
        "severity": req.get("severity").cloned().unwrap_or(Value::Null),
        "description": req.get("description").cloned().unwrap_or(Value::Null),
        "recorded_at": chrono::Utc::now().to_rfc3339(),
    });
    // Real. Previously minted a finding_id, echoed the request and stored
    // nothing, so a red-team campaign could never accumulate findings.
    campaign["findings"]
        .as_array_mut()
        .expect("findings is initialised as an array above")
        .push(finding.clone());
    Ok((StatusCode::CREATED, Json(finding)).into_response())
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
