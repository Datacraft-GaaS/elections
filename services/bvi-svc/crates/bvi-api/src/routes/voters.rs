//! GET /api/v1/bvi/voters/{voter_id}/station.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    error::ApiError,
    idem,
    models::{LookupAuditCtx, VoterStation},
    routes::{canaries, store_error},
    state::AppState,
    store::BviStore,
};

const INSTANCE: &str = "/api/v1/bvi/voters/{voter_id}/station";

#[derive(Debug, Deserialize)]
pub struct VoterStationQuery {
    pub election_id: Option<String>,
}

/// GET /api/v1/bvi/voters/{voter_id}/station (getVoterStation).
///
/// Station-assignment lookup only (data-class B); every lookup, found or not,
/// is audit-logged with hashed actor/IP context per FR-M07-020.
pub async fn get_voter_station<S: BviStore>(
    State(state): State<AppState<S>>,
    Path(voter_id): Path<String>,
    headers: HeaderMap,
    Query(q): Query<VoterStationQuery>,
) -> Result<Json<VoterStation>, ApiError> {
    idem::require_bearer(&headers, INSTANCE)?;

    let election_id = q
        .election_id
        .filter(|e| !e.is_empty())
        .ok_or_else(|| ApiError::bad_request(INSTANCE, "election_id is required"))?;

    let audit = LookupAuditCtx {
        actor_session_hash: hash_header(&headers, "authorization"),
        ip_hash: hash_header(&headers, "x-forwarded-for"),
    };

    let lookup = state
        .store
        .voter_station(&voter_id, &election_id, &audit)
        .await
        .map_err(|e| store_error(e, INSTANCE))?;

    // Canary tripwire on BOTH outcomes; the caller-visible response below is
    // byte-for-byte identical whether or not this fires.
    let response = match lookup {
        Some(station) => {
            canaries::observe_lookup(&voter_id, "found");
            Ok(Json(station))
        }
        None => {
            canaries::observe_lookup(&voter_id, "not_found");
            Err(ApiError::not_found(INSTANCE))
        }
    };
    response
}

/// SHA-256 of a request header (hex), or of the literal "absent" when unset.
fn hash_header(headers: &HeaderMap, name: &str) -> String {
    let raw = headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("absent");
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}
