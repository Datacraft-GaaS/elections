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

//! Gazette handlers.

use super::*;

/// Build a gazette notice.
pub async fn build_gazette(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildGazette>,
) -> Result<Json<Gazette>> {
    let gazette = Gazette::build(
        req.election_id,
        req.position_code,
        req.entries,
        req.officer_a_id,
        req.officer_b_id,
    );
    state
        .gazettes
        .lock()
        .map_err(|_| lock_err())?
        .insert(gazette.id, gazette.clone());
    Ok(Json(gazette))
}

/// Fetch a gazette by id.
pub async fn get_gazette(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Gazette>> {
    let gazette = state
        .gazettes
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("gazette", id))?;
    Ok(Json(gazette))
}

/// Publish a gazette.
pub async fn publish_gazette(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Gazette>> {
    let mut guard = state.gazettes.lock().map_err(|_| lock_err())?;
    let gazette = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("gazette", id))?;
    gazette.publish();
    let g = gazette.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::GazettePublished,
        "gazette",
        id,
        serde_json::json!({ "gazette_id": id, "content_hash": g.content_hash }),
    ));
    Ok(Json(g))
}

/// Export a gazette as CSV.
pub async fn gazette_csv(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let gazette = state
        .gazettes
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("gazette", id))?;
    Ok(([(header::CONTENT_TYPE, "text/csv; charset=utf-8")], gazette.to_csv()).into_response())
}

/// Export a signed JSON gazette.
pub async fn gazette_signed_json(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let gazette = state
        .gazettes
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("gazette", id))?;
    let parsed = serde_json::from_str(&gazette.to_signed_json())
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(parsed))
}
