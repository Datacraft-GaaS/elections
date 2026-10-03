// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Axum 0.8 REST router for the telemetry query API.
//!
//! Routes:
//!   GET  /v1/telemetry/devices/{device_id}/timeline?from=&to=
//!   POST /v1/telemetry/query
//!   POST /v1/telemetry/query/nl
//!   GET  /v1/telemetry/geo/near?lat=&lon=&radius_m=&from=&to=
//!   GET  /v1/telemetry/officers/{officer_id}/sessions?from=&to=
//!   GET  /v1/telemetry/drops?severity=critical&from=&to=
//!   GET  /v1/telemetry/anomalies?class=&from=&to=
//!   GET  /v1/telemetry/presets
//!   POST /v1/telemetry/presets/{name}/run
//!   POST /v1/telemetry/queries/{query_id}/export

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::audit;
use crate::dsl::{SignalClass, TelemetryQuery, TimeRange};
use crate::evidence_bundle::EvidenceBundleBuilder;
use crate::executor::TelemetryExecutor;
use crate::metrics;
use crate::nl_bridge::{LiteLlmBridge, MockBridge, NlQueryBridge, ProposedQuery};
use crate::permission;
use crate::preset::PresetLibrary;
use crate::validator::TelemetryQueryValidator;

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct TelemetryApiState {
    pub pool: PgPool,
    pub use_mock_nl: bool,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(state: TelemetryApiState) -> Router {
    Router::new()
        .route(
            "/v1/telemetry/devices/{device_id}/timeline",
            get(timeline_handler),
        )
        .route("/v1/telemetry/query", post(structured_query_handler))
        .route("/v1/telemetry/query/nl", post(nl_query_handler))
        .route("/v1/telemetry/geo/near", get(geo_near_handler))
        .route(
            "/v1/telemetry/officers/{officer_id}/sessions",
            get(officer_sessions_handler),
        )
        .route("/v1/telemetry/drops", get(drops_handler))
        .route("/v1/telemetry/anomalies", get(anomalies_handler))
        .route("/v1/telemetry/presets", get(list_presets_handler))
        .route("/v1/telemetry/presets/{name}/run", post(run_preset_handler))
        .route(
            "/v1/telemetry/queries/{query_id}/export",
            post(export_handler),
        )
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Common types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct TimeParams {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<serde_json::Value>>,
}

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorBody {
            error: msg.into(),
            diagnostics: None,
        }),
    )
}

fn forbidden(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorBody {
            error: msg.into(),
            diagnostics: None,
        }),
    )
}

fn internal(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            error: msg.into(),
            diagnostics: None,
        }),
    )
}

/// Extract actor SVID from `X-Actor-Svid` header (stub: defaults to "anonymous").
fn actor_svid(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-actor-svid")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string()
}

// ---------------------------------------------------------------------------
// GET /v1/telemetry/devices/{device_id}/timeline
// ---------------------------------------------------------------------------

async fn timeline_handler(
    State(state): State<TelemetryApiState>,
    Path(device_id): Path<Uuid>,
    Query(params): Query<TimeParams>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let actor = actor_svid(&headers);
    if let Err(e) = permission::require_analyst(&actor).await {
        return forbidden(e.to_string()).into_response();
    }

    let q = TelemetryQuery::for_device(
        device_id,
        TimeRange {
            from: params.from,
            to: params.to,
        },
    );

    let executor = TelemetryExecutor::new(state.pool.clone());
    match executor.execute(&q).await {
        Ok(rows) => {
            metrics::inc_query("timeline", true);
            let _ =
                audit::record_query(&state.pool, &actor, &q, rows.len() as i64, None, None, None)
                    .await;
            Json(serde_json::json!({ "rows": rows, "count": rows.len() })).into_response()
        }
        Err(e) => {
            metrics::inc_query("timeline", false);
            internal(e.to_string()).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// POST /v1/telemetry/query
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct StructuredQueryRequest {
    #[serde(flatten)]
    query: TelemetryQuery,
    idempotency_key: Option<String>,
}

async fn structured_query_handler(
    State(state): State<TelemetryApiState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<StructuredQueryRequest>,
) -> impl IntoResponse {
    let actor = actor_svid(&headers);
    if let Err(e) = permission::require_analyst(&actor).await {
        return forbidden(e.to_string()).into_response();
    }

    if let Err(e) = TelemetryQueryValidator::validate(&req.query) {
        let msg = e.to_string();
        metrics::inc_query("structured", false);
        return bad_request(msg).into_response();
    }

    let executor = TelemetryExecutor::new(state.pool.clone());
    match executor.execute(&req.query).await {
        Ok(rows) => {
            metrics::inc_query("structured", true);
            let _ = audit::record_query(
                &state.pool,
                &actor,
                &req.query,
                rows.len() as i64,
                None,
                None,
                None,
            )
            .await;
            Json(serde_json::json!({
                "rows": rows,
                "count": rows.len(),
                "idempotency_key": req.idempotency_key,
            }))
            .into_response()
        }
        Err(e) => {
            metrics::inc_query("structured", false);
            internal(e.to_string()).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// POST /v1/telemetry/query/nl
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct NlQueryRequest {
    text: String,
    idempotency_key: Option<String>,
}

#[derive(Serialize)]
struct NlQueryResponse {
    proposed: ProposedQuery,
    idempotency_key: Option<String>,
    confirmation_required: bool,
}

async fn nl_query_handler(
    State(state): State<TelemetryApiState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<NlQueryRequest>,
) -> impl IntoResponse {
    let actor = actor_svid(&headers);
    if let Err(e) = permission::require_analyst(&actor).await {
        return forbidden(e.to_string()).into_response();
    }

    let result = if state.use_mock_nl {
        MockBridge.translate(&req.text, &actor).await
    } else {
        LiteLlmBridge::from_env().translate(&req.text, &actor).await
    };

    match result {
        Ok(proposed) => {
            metrics::inc_nl(if state.use_mock_nl { "mock" } else { "litellm" }, true);
            Json(NlQueryResponse {
                proposed,
                idempotency_key: req.idempotency_key,
                confirmation_required: true,
            })
            .into_response()
        }
        Err(e) => {
            metrics::inc_nl(if state.use_mock_nl { "mock" } else { "litellm" }, false);
            internal(e.to_string()).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// GET /v1/telemetry/geo/near
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GeoNearParams {
    lat: f64,
    lon: f64,
    radius_m: f64,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

async fn geo_near_handler(
    State(state): State<TelemetryApiState>,
    Query(params): Query<GeoNearParams>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let actor = actor_svid(&headers);
    if let Err(e) = permission::require_analyst(&actor).await {
        return forbidden(e.to_string()).into_response();
    }

    // Build a square bounding box as the geo polygon (approx; proper circle needs PostGIS)
    let deg_per_m = 1.0 / 111_320.0;
    let dlat = params.radius_m * deg_per_m;
    let dlon = params.radius_m * deg_per_m / params.lat.to_radians().cos().max(f64::EPSILON);

    let polygon = crate::dsl::GeoPolygon {
        coordinates: vec![
            [params.lon - dlon, params.lat - dlat],
            [params.lon + dlon, params.lat - dlat],
            [params.lon + dlon, params.lat + dlat],
            [params.lon - dlon, params.lat + dlat],
            [params.lon - dlon, params.lat - dlat],
        ],
    };

    let q = TelemetryQuery {
        device_ids: vec![],
        signal_classes: vec![SignalClass::Location],
        time_range: TimeRange {
            from: params.from,
            to: params.to,
        },
        geo_polygon: Some(polygon),
        aggregations: vec![],
        group_by: vec![],
        limit: 10_000,
        order_by: vec![],
    };

    if let Err(e) = TelemetryQueryValidator::validate(&q) {
        return bad_request(e.to_string()).into_response();
    }

    let executor = TelemetryExecutor::new(state.pool.clone());
    match executor.execute(&q).await {
        Ok(rows) => {
            metrics::inc_query("geo_near", true);
            let _ = audit::record_query(
                &state.pool,
                &actor,
                &q,
                rows.len() as i64,
                None,
                Some("geo_near"),
                None,
            )
            .await;
            Json(serde_json::json!({ "rows": rows, "count": rows.len() })).into_response()
        }
        Err(e) => {
            metrics::inc_query("geo_near", false);
            internal(e.to_string()).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// GET /v1/telemetry/officers/{officer_id}/sessions
// ---------------------------------------------------------------------------

async fn officer_sessions_handler(
    State(state): State<TelemetryApiState>,
    Path(officer_id): Path<String>,
    Query(params): Query<TimeParams>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let actor = actor_svid(&headers);
    if let Err(e) = permission::require_analyst(&actor).await {
        return forbidden(e.to_string()).into_response();
    }

    let mut p = serde_json::Map::new();
    p.insert("officer_id".into(), serde_json::json!(officer_id));
    p.insert("from".into(), serde_json::json!(params.from.to_rfc3339()));
    p.insert("to".into(), serde_json::json!(params.to.to_rfc3339()));

    match PresetLibrary::build("officer_activity", &p) {
        Ok((q, _desc)) => {
            let executor = TelemetryExecutor::new(state.pool.clone());
            match executor.execute(&q).await {
                Ok(rows) => {
                    metrics::inc_query("officer_sessions", true);
                    let _ = audit::record_query(
                        &state.pool,
                        &actor,
                        &q,
                        rows.len() as i64,
                        None,
                        Some("officer_activity"),
                        None,
                    )
                    .await;
                    Json(serde_json::json!({ "rows": rows, "count": rows.len() })).into_response()
                }
                Err(e) => internal(e.to_string()).into_response(),
            }
        }
        Err(e) => bad_request(e).into_response(),
    }
}

// ---------------------------------------------------------------------------
// GET /v1/telemetry/drops
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct DropsParams {
    severity: Option<String>,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

async fn drops_handler(
    State(state): State<TelemetryApiState>,
    Query(params): Query<DropsParams>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let actor = actor_svid(&headers);
    if let Err(e) = permission::require_analyst(&actor).await {
        return forbidden(e.to_string()).into_response();
    }

    let q = TelemetryQuery {
        device_ids: vec![],
        signal_classes: vec![SignalClass::PhysicalDrops],
        time_range: TimeRange {
            from: params.from,
            to: params.to,
        },
        geo_polygon: None,
        aggregations: vec![],
        group_by: vec![],
        limit: 50_000,
        order_by: vec![],
    };

    if let Err(e) = TelemetryQueryValidator::validate(&q) {
        return bad_request(e.to_string()).into_response();
    }

    let executor = TelemetryExecutor::new(state.pool.clone());
    match executor.execute(&q).await {
        Ok(rows) => {
            metrics::inc_query("drops", true);
            let _ =
                audit::record_query(&state.pool, &actor, &q, rows.len() as i64, None, None, None)
                    .await;
            Json(serde_json::json!({
                "rows": rows,
                "count": rows.len(),
                "severity_filter": params.severity,
            }))
            .into_response()
        }
        Err(e) => {
            metrics::inc_query("drops", false);
            internal(e.to_string()).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// GET /v1/telemetry/anomalies
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct AnomaliesParams {
    class: Option<String>,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

async fn anomalies_handler(
    State(state): State<TelemetryApiState>,
    Query(params): Query<AnomaliesParams>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let actor = actor_svid(&headers);
    if let Err(e) = permission::require_analyst(&actor).await {
        return forbidden(e.to_string()).into_response();
    }

    let q = TelemetryQuery {
        device_ids: vec![],
        signal_classes: vec![],
        time_range: TimeRange {
            from: params.from,
            to: params.to,
        },
        geo_polygon: None,
        aggregations: vec![],
        group_by: vec![],
        limit: 20_000,
        order_by: vec![],
    };

    if let Err(e) = TelemetryQueryValidator::validate(&q) {
        return bad_request(e.to_string()).into_response();
    }

    let executor = TelemetryExecutor::new(state.pool.clone());
    match executor.execute(&q).await {
        Ok(rows) => {
            metrics::inc_query("anomalies", true);
            Json(serde_json::json!({
                "rows": rows,
                "count": rows.len(),
                "class_filter": params.class,
            }))
            .into_response()
        }
        Err(e) => {
            metrics::inc_query("anomalies", false);
            internal(e.to_string()).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// GET /v1/telemetry/presets
// ---------------------------------------------------------------------------

async fn list_presets_handler() -> impl IntoResponse {
    Json(serde_json::json!({ "presets": PresetLibrary::list() }))
}

// ---------------------------------------------------------------------------
// POST /v1/telemetry/presets/{name}/run
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RunPresetRequest {
    #[serde(default)]
    params: serde_json::Map<String, serde_json::Value>,
    idempotency_key: Option<String>,
}

async fn run_preset_handler(
    State(state): State<TelemetryApiState>,
    Path(name): Path<String>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RunPresetRequest>,
) -> impl IntoResponse {
    let actor = actor_svid(&headers);
    if let Err(e) = permission::require_analyst(&actor).await {
        return forbidden(e.to_string()).into_response();
    }

    let (q, description) = match PresetLibrary::build(&name, &req.params) {
        Ok(r) => r,
        Err(e) => return bad_request(e).into_response(),
    };

    if let Err(e) = TelemetryQueryValidator::validate(&q) {
        return bad_request(e.to_string()).into_response();
    }

    let executor = TelemetryExecutor::new(state.pool.clone());
    match executor.execute(&q).await {
        Ok(rows) => {
            metrics::inc_query(&name, true);
            let _ = audit::record_query(
                &state.pool,
                &actor,
                &q,
                rows.len() as i64,
                None,
                Some(&name),
                None,
            )
            .await;
            Json(serde_json::json!({
                "rows": rows,
                "count": rows.len(),
                "description": description,
                "preset": name,
                "idempotency_key": req.idempotency_key,
            }))
            .into_response()
        }
        Err(e) => {
            metrics::inc_query(&name, false);
            internal(e.to_string()).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// POST /v1/telemetry/queries/{query_id}/export
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ExportRequest {
    idempotency_key: Option<String>,
}

async fn export_handler(
    State(state): State<TelemetryApiState>,
    Path(query_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ExportRequest>,
) -> impl IntoResponse {
    let actor = actor_svid(&headers);
    if let Err(e) = permission::require_admin(&actor).await {
        return forbidden(e.to_string()).into_response();
    }

    // Re-fetch the original query DSL from the audit log
    let row = sqlx::query(
        "SELECT query_dsl, executed_at, row_count FROM eel_telemetry_queries WHERE query_id = $1",
    )
    .bind(query_id)
    .fetch_optional(&state.pool)
    .await;

    let (q, executed_at, _row_count) = match row {
        Ok(Some(r)) => {
            use sqlx::Row as _;
            let dsl: serde_json::Value = r.try_get("query_dsl").unwrap_or_default();
            let ea: DateTime<Utc> = r.try_get("executed_at").unwrap_or_else(|_| Utc::now());
            let rc: i64 = r.try_get("row_count").unwrap_or(0);
            let q: TelemetryQuery = match serde_json::from_value(dsl) {
                Ok(q) => q,
                Err(e) => return internal(format!("DSL parse error: {}", e)).into_response(),
            };
            (q, ea, rc)
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "query not found".into(),
                    diagnostics: None,
                }),
            )
                .into_response()
        }
        Err(e) => return internal(e.to_string()).into_response(),
    };

    // Re-execute to get current rows for the bundle
    let executor = TelemetryExecutor::new(state.pool.clone());
    let rows = match executor.execute(&q).await {
        Ok(r) => r,
        Err(e) => return internal(e.to_string()).into_response(),
    };

    let builder = EvidenceBundleBuilder {
        query_id: query_id.to_string(),
        actor_svid: actor.clone(),
        query: q,
        rows,
        executed_at,
    };

    match builder.build() {
        Ok(bundle) => {
            metrics::inc_evidence(true);
            let _ = audit::mark_evidence_exported(
                &state.pool,
                query_id,
                &bundle.manifest.manifest_sha256,
            )
            .await;
            Json(serde_json::json!({
                "manifest": bundle.manifest,
                "idempotency_key": req.idempotency_key,
                "message": "Evidence bundle generated. Use the manifest sha256 to verify.",
            }))
            .into_response()
        }
        Err(e) => {
            metrics::inc_evidence(false);
            internal(e.to_string()).into_response()
        }
    }
}
