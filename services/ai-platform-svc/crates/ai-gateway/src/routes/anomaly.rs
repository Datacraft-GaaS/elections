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

use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    routing::{get, patch, post},
    Json, Router,
};
use futures::stream;
use serde::Deserialize;
use tokio_stream::StreamExt as _;

use crate::{
    error::ApiError,
    models::AnomalyAlert,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/alerts", get(list_alerts).post(ingest_alert))
        .route("/alerts/{id}", get(get_alert))
        .route("/alerts/{id}/resolve", patch(resolve_alert))
        .route("/stream", get(stream_alerts))
}

async fn list_alerts(
    State(state): State<AppState>,
) -> Json<Vec<AnomalyAlert>> {
    let alerts = state.alerts.read().await;
    Json(alerts.clone())
}

async fn get_alert(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AnomalyAlert>, ApiError> {
    let alerts = state.alerts.read().await;
    alerts
        .iter()
        .find(|a| a.id == id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}

#[derive(Debug, Deserialize)]
struct ResolvePayload {
    resolution: String,
}

async fn resolve_alert(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ResolvePayload>,
) -> Result<Json<AnomalyAlert>, ApiError> {
    let mut alerts = state.alerts.write().await;
    let alert = alerts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or(ApiError::NotFound)?;
    alert.resolution = Some(payload.resolution);
    Ok(Json(alert.clone()))
}

async fn ingest_alert(
    State(state): State<AppState>,
    Json(alert): Json<AnomalyAlert>,
) -> Json<AnomalyAlert> {
    let mut alerts = state.alerts.write().await;
    alerts.push(alert.clone());
    Json(alert)
}

async fn stream_alerts() -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(Duration::from_secs(30));
    let stream = tokio_stream::wrappers::IntervalStream::new(interval)
        .map(|_| Ok(Event::default().data("heartbeat")));
    Sse::new(stream)
}
