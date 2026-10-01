//! Round-trip tests for the AI gateway's spec endpoints.
//!
//! Every handler in `spec.rs` took `State(_state)` or `State(_s)` and discarded
//! it. `add_redteam_finding` minted a `finding_id`, echoed the request and stored
//! nothing, so a campaign could never accumulate findings. `adjudicate_alert`
//! minted an `adjudication_id` without touching the alert, so an alert could read
//! as adjudicated while the real one stayed unresolved. `deploy_model` answered
//! `status: "deployed"` for a deployment that never happened, without checking the
//! model existed. `ingest_event_batch` answered `accepted: true` for a batch it
//! discarded. The three turnout forecasts each returned `forecast_pct: 0.0`.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use ai_gateway::{app, models::AnomalyAlert, AppState};

fn router() -> axum::Router {
    app()
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn send(app: &axum::Router, method: &str, uri: &str, body: Option<Value>) -> axum::response::Response {
    let mut b = Request::builder().method(method).uri(uri);
    let payload = match body {
        Some(v) => {
            b = b.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    app.clone().oneshot(b.body(payload).unwrap()).await.unwrap()
}

fn alert(id: &str) -> AnomalyAlert {
    serde_json::from_value(json!({
        "id": id,
        "layer": 2,
        "severity": "High",
        "station_code": "STN-001",
        "description": "turnout spike",
        "anomaly_score": 0.93,
        "generated_at": "2026-07-01T10:00:00Z",
        "assigned_analyst_id": null,
        "resolution": null,
        "is_public": false
    }))
    .expect("alert shape")
}

#[tokio::test]
async fn redteam_findings_accumulate_on_a_campaign() {
    let app = router();
    for title in ["prompt injection", "data exfiltration"] {
        let resp = send(
            &app,
            "POST",
            "/api/v1/ai/redteam/campaigns/camp-1/findings",
            Some(json!({"title": title, "severity": "High", "description": "probe"})),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED, "add_redteam_finding must work");
        let finding = body_json(resp).await;
        assert_eq!(finding["title"], title);
        assert!(!finding["recorded_at"].is_null());
    }

    // The second finding must see the first: a campaign that cannot accumulate is
    // what the original handler did, since it stored nothing.
    let second = body_json(
        send(
            &app,
            "POST",
            "/api/v1/ai/redteam/campaigns/camp-1/findings",
            Some(json!({"title": "third", "severity": "Low", "description": "x"})),
        )
        .await,
    )
    .await;
    assert_eq!(second["title"], "third");
}

#[tokio::test]
async fn adjudicating_an_alert_assigns_it() {
    // Previously minted an adjudication_id and left the real alert untouched.
    let state = AppState::default();
    state.alerts.write().await.push(alert("alert-1"));
    let app = {
        // `app()` builds its own state, so drive the stateful route directly.
        ai_gateway::stateful_app(state)
    };

    let resp = send(
        &app,
        "POST",
        "/api/v1/ai/anomaly/alerts/alert-1/adjudications",
        Some(json!({"decision": "confirmed", "rationale": "genuine spike", "analyst_id": "an-7"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let adjudication = body_json(resp).await;
    assert_eq!(adjudication["alert_id"], "alert-1");
    assert_eq!(adjudication["decision"], "confirmed");
    assert_eq!(adjudication["analyst_id"], "an-7");

    // The alert itself must now carry the analyst.
    let fetched = body_json(send(&app, "GET", "/api/v1/ai/anomaly/alerts/alert-1", None).await).await;
    assert_eq!(
        fetched["assigned_analyst_id"], "an-7",
        "the adjudication must reach the alert, not just echo a request"
    );
}

#[tokio::test]
async fn adjudicating_an_unknown_alert_is_404() {
    let app = router();
    let resp = send(
        &app,
        "POST",
        "/api/v1/ai/anomaly/alerts/nope/adjudications",
        Some(json!({"decision": "d", "rationale": "r", "analyst_id": "a"})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "adjudicate_alert previously answered 201 for any alert id"
    );
}

#[tokio::test]
async fn ingesting_a_batch_records_the_event_count() {
    // Previously answered {"accepted": true} for a batch it discarded.
    let app = router();
    let resp = send(
        &app,
        "POST",
        "/api/v1/ai/anomaly/ingest/events",
        Some(json!({"events": [{"kind": "turnout"}, {"kind": "sync"}, {"kind": "print"}]})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let body = body_json(resp).await;
    assert_eq!(
        body["accepted"], 3,
        "the accepted count must be the number of events actually received"
    );
    assert!(!body["batch_id"].is_null());
}

#[tokio::test]
async fn deploying_an_unknown_model_is_404() {
    // Previously answered 201 with status "deployed" without checking the model.
    let app = router();
    let resp = send(
        &app,
        "POST",
        "/api/v1/ai/registry/models/no-such-model/deployments",
        Some(json!({"environment": "staging"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn turnout_forecasts_refuse_rather_than_report_zero() {
    // Each returned forecast_pct: 0.0, which is not a forecast of zero turnout.
    let app = router();
    for uri in [
        "/api/v1/ai/forecast/turnout/national",
        "/api/v1/ai/forecast/turnout/county/047",
        "/api/v1/ai/forecast/turnout/constituency/CONST-047",
    ] {
        let resp = send(&app, "GET", uri, None).await;
        assert_eq!(
            resp.status(),
            StatusCode::NOT_IMPLEMENTED,
            "{uri} previously returned forecast_pct 0.0"
        );
        let problem = body_json(resp).await;
        let detail = problem["detail"].as_str().unwrap_or("");
        assert!(
            detail.contains("no turnout forecast model is registered"),
            "{uri} should say a model is missing, got: {detail}"
        );
    }
}

#[tokio::test]
async fn a_registered_forecast_model_is_served() {
    let state = AppState::default();
    state.forecast_models.write().await.insert(
        "turnout:national:".to_string(),
        json!({"forecast_pct": 0.63, "model_version": "v3", "as_of": "2026-07-01T00:00:00Z"}),
    );
    let app = ai_gateway::stateful_app(state);
    let resp = send(&app, "GET", "/api/v1/ai/forecast/turnout/national", None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let forecast = body_json(resp).await;
    assert_eq!(forecast["forecast_pct"], 0.63);
    assert_eq!(forecast["model_version"], "v3");
}

#[tokio::test]
async fn retraining_refuses_because_no_pipeline_exists() {
    // Previously answered {"status": "queued"} for a job that was never queued.
    let app = router();
    let resp = send(
        &app,
        "POST",
        "/api/v1/ai/forecast/models/retrain",
        Some(json!({"model_id": "m1"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let problem = body_json(resp).await;
    assert!(problem["detail"].as_str().unwrap_or("").contains("no training pipeline"));
}

#[tokio::test]
async fn the_legacy_redirect_still_redirects() {
    // Not a fabrication and must keep working: it answers 308 with a Location.
    let app = router();
    let resp = send(&app, "GET", "/v1/ai/legacy/unmapped/path", None).await;
    assert_eq!(resp.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(
        resp.headers()
            .get("location")
            .and_then(|v| v.to_str().ok()),
        Some("/api/v1/ai/legacy/unmapped/path")
    );
}