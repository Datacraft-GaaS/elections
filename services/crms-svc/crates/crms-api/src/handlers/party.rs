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

//! Party handlers.

use super::*;

/// Register a political party.
pub async fn create_party(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateParty>,
) -> Result<Json<PoliticalParty>> {
    let party = PoliticalParty::new(
        Uuid::new_v4(),
        req.name_en,
        req.name_sw,
        req.abbreviation,
        req.hq_address,
    );
    state
        .parties
        .lock()
        .map_err(|_| lock_err())?
        .insert(party.id, party.clone());
    Ok(Json(party))
}

/// List political parties.
pub async fn list_parties(State(state): State<Arc<AppState>>) -> Result<Json<Vec<PoliticalParty>>> {
    let all = state
        .parties
        .lock()
        .map_err(|_| lock_err())?
        .values()
        .cloned()
        .collect();
    Ok(Json(all))
}

/// Fetch a party by id.
pub async fn get_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PoliticalParty>> {
    let party = state
        .parties
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("party", id))?;
    Ok(Json(party))
}

/// Submit a party for registrar approval.
pub async fn submit_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PoliticalParty>> {
    let mut guard = state.parties.lock().map_err(|_| lock_err())?;
    let party = guard.get_mut(&id).ok_or_else(|| not_found("party", id))?;
    party.submit();
    Ok(Json(party.clone()))
}

/// Approve a party registration.
pub async fn approve_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApproveParty>,
) -> Result<Json<PoliticalParty>> {
    let mut guard = state.parties.lock().map_err(|_| lock_err())?;
    let party = guard.get_mut(&id).ok_or_else(|| not_found("party", id))?;
    party.approve(req.registration_number, req.gazette_url);
    let p = party.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::PartyApproved,
        "party",
        id,
        serde_json::json!({ "party_id": id, "registration_number": p.registration_number }),
    ));
    Ok(Json(p))
}

/// Suspend a party.
pub async fn suspend_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PoliticalParty>> {
    let mut guard = state.parties.lock().map_err(|_| lock_err())?;
    let party = guard.get_mut(&id).ok_or_else(|| not_found("party", id))?;
    party.suspend();
    let p = party.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::PartySuspended,
        "party",
        id,
        serde_json::json!({ "party_id": id }),
    ));
    Ok(Json(p))
}

/// Deregister a party.
pub async fn deregister_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PoliticalParty>> {
    let mut guard = state.parties.lock().map_err(|_| lock_err())?;
    let party = guard.get_mut(&id).ok_or_else(|| not_found("party", id))?;
    party.deregister();
    Ok(Json(party.clone()))
}
