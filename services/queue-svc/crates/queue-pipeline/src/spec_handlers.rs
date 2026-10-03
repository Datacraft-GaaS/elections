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

//! Public contract handlers (queue/transparency.yaml).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::spec;
use crate::spec_state::SharedState;

/// POST /queue/observations — submit a queue observation.
pub async fn submit_queue_observation(
    State(state): State<SharedState>,
    Json(req): Json<spec::QueueObservationSubmission>,
) -> impl IntoResponse {
    info!(station = %req.station_code, "submit_queue_observation");
    let observation = spec::QueueObservation {
        observation_id: uuid::Uuid::new_v4().to_string(),
        station_code: req.station_code,
        observed_queue_length: req.observed_queue_length,
        observed_at: req.observed_at,
        submitted_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.observations.lock().expect("lock poisoned");
    guard
        .entry(observation.station_code.clone())
        .or_default()
        .push(observation.clone());
    drop(guard);
    (StatusCode::CREATED, Json(observation)).into_response()
}

fn status_for(station: &str, obs: &[spec::QueueObservation]) -> spec::QueueStatus {
    let latest = obs
        .iter()
        .max_by(|a, b| a.submitted_at.cmp(&b.submitted_at));
    let queue_length = latest.map(|o| o.observed_queue_length).unwrap_or(0);
    spec::QueueStatus {
        station_code: station.to_string(),
        estimated_wait_minutes: queue_length.saturating_mul(3),
        queue_length,
        last_updated_at: latest.map(|o| o.submitted_at.clone()).unwrap_or_default(),
        is_open: queue_length > 0,
        observation_count: obs.len() as u32,
    }
}

/// GET /queue/stations — list all observed station queue statuses.
pub async fn list_station_queue_statuses(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_station_queue_statuses");
    let guard = state.observations.lock().expect("lock poisoned");
    let mut items: Vec<spec::QueueStatus> = guard
        .iter()
        .map(|(station, obs)| status_for(station, obs))
        .collect();
    drop(guard);
    items.sort_by(|a, b| a.station_code.cmp(&b.station_code));
    let n = items.len();
    Json(serde_json::json!({ "items": items, "total": n, "page": 1, "page_size": n }))
}

/// GET /queue/stations/{station_code}/status — fetch one station's queue status.
pub async fn get_station_queue_status(
    State(state): State<SharedState>,
    Path(station_code): Path<String>,
) -> impl IntoResponse {
    info!(station_code, "get_station_queue_status");
    let guard = state.observations.lock().expect("lock poisoned");
    match guard.get(&station_code) {
        Some(obs) => Json(status_for(&station_code, obs)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "station not found"})),
        )
            .into_response(),
    }
}

/// GET /queue/analytics/county/{county_code} — aggregate county queue analytics.
pub async fn get_county_queue_analytics(
    State(state): State<SharedState>,
    Path(county_code): Path<String>,
) -> impl IntoResponse {
    info!(county_code, "get_county_queue_analytics");
    let stations: Vec<(String, Vec<spec::QueueObservation>)> = {
        let guard = state.observations.lock().expect("lock poisoned");
        guard
            .iter()
            .filter(|(station, _)| {
                station.starts_with(&format!("{county_code}-")) || station.starts_with(&county_code)
            })
            .map(|(station, obs)| (station.clone(), obs.clone()))
            .collect()
    };
    let mut stations = stations;
    stations.sort_by(|a, b| a.0.cmp(&b.0));

    let station_count = stations.len() as u32;
    let total_observations: u32 = stations.iter().map(|(_, obs)| obs.len() as u32).sum();
    let mut wait_sum: u64 = 0;
    let mut max_wait = 0u32;
    let mut open = 0u32;
    for (_, obs) in &stations {
        let ql = obs.last().map(|o| o.observed_queue_length).unwrap_or(0);
        let wait = ql * 3;
        wait_sum += wait as u64;
        max_wait = max_wait.max(wait);
        if ql > 0 {
            open += 1;
        }
    }
    let avg_wait = if station_count > 0 {
        wait_sum as f64 / station_count as f64
    } else {
        0.0
    };

    Json(spec::CountyQueueAnalytics {
        county_code,
        generated_at: chrono::Utc::now().to_rfc3339(),
        station_count,
        stations_open: open,
        avg_wait_minutes: avg_wait,
        max_wait_minutes: max_wait,
        total_observations,
        peak_hour: None,
    })
    .into_response()
}
