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

//! Party symbol handlers.

use super::*;

/// Register a party symbol.
pub async fn register_symbol(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterSymbol>,
) -> Result<Json<PartySymbol>> {
    let existing: Vec<PartySymbol> = state
        .symbols
        .lock()
        .map_err(|_| lock_err())?
        .values()
        .cloned()
        .collect();
    check_symbol_availability(&existing, &req.symbol_name, &req.colour_hex)
        .map_err(ApiError::Conflict)?;
    let symbol = PartySymbol::new(req.party_id, req.symbol_name, req.symbol_url, req.colour_hex);
    state
        .symbols
        .lock()
        .map_err(|_| lock_err())?
        .insert(symbol.id, symbol.clone());
    Ok(Json(symbol))
}

/// List registered symbols.
pub async fn list_symbols(State(state): State<Arc<AppState>>) -> Result<Json<Vec<PartySymbol>>> {
    let all = state
        .symbols
        .lock()
        .map_err(|_| lock_err())?
        .values()
        .cloned()
        .collect();
    Ok(Json(all))
}

/// Approve a symbol.
pub async fn approve_symbol(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PartySymbol>> {
    let mut guard = state.symbols.lock().map_err(|_| lock_err())?;
    let symbol = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("symbol", id))?;
    symbol.approve();
    let s = symbol.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::SymbolAssigned,
        "symbol",
        id,
        serde_json::json!({ "symbol_id": id, "party_id": s.party_id }),
    ));
    Ok(Json(s))
}

/// Reject a symbol.
pub async fn reject_symbol(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PartySymbol>> {
    let mut guard = state.symbols.lock().map_err(|_| lock_err())?;
    let symbol = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("symbol", id))?;
    symbol.reject();
    Ok(Json(symbol.clone()))
}
