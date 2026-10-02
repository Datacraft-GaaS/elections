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

//! Ballot proof handlers.

use super::*;

/// Build a ballot proof.
pub async fn build_ballot(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildBallot>,
) -> Result<Json<BallotProof>> {
    let proof = BallotProof::build(
        req.election_id,
        req.position_code,
        req.constituency_code,
        req.positions,
    );
    state
        .ballots
        .lock()
        .map_err(|_| lock_err())?
        .insert(proof.id, proof.clone());
    Ok(Json(proof))
}

/// Fetch a ballot proof by id.
pub async fn get_ballot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BallotProof>> {
    let proof = state
        .ballots
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("ballot", id))?;
    Ok(Json(proof))
}

/// Record a party's sign-off on a ballot.
pub async fn ballot_party_sign_off(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BallotProof>> {
    let mut guard = state.ballots.lock().map_err(|_| lock_err())?;
    let proof = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("ballot", id))?;
    proof.record_party_sign_off();
    Ok(Json(proof.clone()))
}

/// Approve a ballot proof with the printer's signature.
pub async fn approve_ballot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApproveBallot>,
) -> Result<Json<BallotProof>> {
    let mut guard = state.ballots.lock().map_err(|_| lock_err())?;
    let proof = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("ballot", id))?;
    proof.approve(req.printer_sig);
    let p = proof.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::BallotApproved,
        "ballot",
        id,
        serde_json::json!({ "ballot_id": id, "ballot_hash": p.ballot_hash }),
    ));
    Ok(Json(p))
}
