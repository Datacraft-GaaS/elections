//! POST /api/v1/bvi/telemetry (submitTelemetry) and
//! POST /api/v1/bvi/distress (raiseDistress).

use axum::{
    extract::{rejection::JsonRejection, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    error::ApiError,
    idem,
    models::{DistressCreate, DISTRESS_TYPES, TelemetryCreate},
    routes::store_error,
    state::AppState,
    store::BviStore,
};

const TELEMETRY_INSTANCE: &str = "/api/v1/bvi/telemetry";
const DISTRESS_INSTANCE: &str = "/api/v1/bvi/distress";

/// POST /api/v1/bvi/telemetry. Data-class D; accepted with 204 and no body.
pub async fn submit_telemetry<S: BviStore>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    body: Result<Json<TelemetryCreate>, JsonRejection>,
) -> Result<Response, ApiError> {
    let key = idem::require_key(&headers, TELEMETRY_INSTANCE)?;
    if let Some(replayed) = idem::replay(&state, "submit_telemetry", key).await? {
        return Ok(replayed);
    }

    let Json(t) = body.map_err(|rej| ApiError::bad_request(TELEMETRY_INSTANCE, rej.body_text()))?;
    if !(0..=100).contains(&t.battery_pct) {
        return Err(ApiError::bad_request(TELEMETRY_INSTANCE, "battery_pct must be within [0, 100]"));
    }

    state
        .store
        .insert_telemetry(&t)
        .await
        .map_err(|e| store_error(e, TELEMETRY_INSTANCE))?;

    // Body-less snapshot: duplicate keys replay an empty 200.
    state
        .store
        .idem_put("submit_telemetry", &key.to_string(), 204, None)
        .await
        .map_err(|_| ApiError::internal(TELEMETRY_INSTANCE))?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// POST /api/v1/bvi/distress. Persists the signal even if NOC delivery is
/// unavailable; notification rides the transactional outbox.
pub async fn raise_distress<S: BviStore>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    body: Result<Json<DistressCreate>, JsonRejection>,
) -> Result<Response, ApiError> {
    let key = idem::require_key(&headers, DISTRESS_INSTANCE)?;
    if let Some(replayed) = idem::replay(&state, "raise_distress", key).await? {
        return Ok(replayed);
    }

    let Json(d) = body.map_err(|rej| ApiError::bad_request(DISTRESS_INSTANCE, rej.body_text()))?;
    if !DISTRESS_TYPES.contains(&d.distress_type.as_str()) {
        return Err(ApiError::bad_request(
            DISTRESS_INSTANCE,
            "distress_type must be one of officer_duress|device_tamper|power_failure|network_failure|physical_intrusion|other",
        ));
    }

    let distress = state
        .store
        .insert_distress(&d)
        .await
        .map_err(|e| store_error(e, DISTRESS_INSTANCE))?;

    idem::snapshot(&state, "raise_distress", key, &distress).await?;
    Ok((StatusCode::CREATED, Json(distress)).into_response())
}
