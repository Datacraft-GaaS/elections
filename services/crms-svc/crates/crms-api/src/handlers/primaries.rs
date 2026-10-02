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

//! Primaries handlers.

use super::*;

/// Submit a primary result.
pub async fn submit_primary_result(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SubmitPrimaryResult>,
) -> Result<Json<PrimaryResult>> {
    let result = PrimaryResult::build(
        req.party_id,
        req.constituency_code,
        req.position,
        req.submitted_by,
        req.candidate_tallies,
        req.spoilt_ballots,
        req.total_turnout,
    );
    state
        .primary_results
        .lock()
        .map_err(|_| lock_err())?
        .insert(result.id, result.clone());
    state.record_event(DomainEvent::new(
        EventType::PrimaryResultSubmitted,
        "primary_result",
        result.id,
        serde_json::json!({ "result_id": result.id, "result_hash": result.result_hash }),
    ));
    Ok(Json(result))
}

/// Fetch a primary result.
pub async fn get_primary_result(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PrimaryResult>> {
    let result = state
        .primary_results
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("primary result", id))?;
    Ok(Json(result))
}

/// Publish a primary result.
pub async fn publish_primary_result(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PrimaryResult>> {
    let mut guard = state.primary_results.lock().map_err(|_| lock_err())?;
    let result = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("primary result", id))?;
    result.publish();
    Ok(Json(result.clone()))
}

/// File a primary dispute.
pub async fn create_primary_dispute(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePrimaryDispute>,
) -> Result<Json<PrimaryDispute>> {
    let dispute = PrimaryDispute::new(
        req.result_id,
        req.complainant_name,
        req.complainant_id,
        req.grounds,
        req.evidence_urls,
    );
    state
        .disputes
        .lock()
        .map_err(|_| lock_err())?
        .push(dispute.clone());
    Ok(Json(dispute))
}

/// List primary disputes.
pub async fn list_primary_disputes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PrimaryDispute>>> {
    let all = state.disputes.lock().map_err(|_| lock_err())?.clone();
    Ok(Json(all))
}
