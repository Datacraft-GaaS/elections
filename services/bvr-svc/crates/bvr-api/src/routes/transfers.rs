use axum::{http::StatusCode, routing::post, Json, Router};
use bvr_domain::ids::uuid7str;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub voter_id: String,
    pub from_station_id: String,
    pub to_station_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub transfer_id: String,
    pub voter_id: String,
    pub status: String,
}

// These handlers need no state, but the router is typed with `AppState` so it
// can be merged into the composed router alongside the stateful sub-routers.
pub fn router() -> Router<crate::state::AppState> {
    Router::<crate::state::AppState>::new().route("/api/v1/bvr/transfers", post(create_transfer))
}

async fn create_transfer(
    Json(req): Json<TransferRequest>,
) -> Result<(StatusCode, Json<TransferResponse>), ApiError> {
    let transfer_id = uuid7str();
    Ok((
        StatusCode::ACCEPTED,
        Json(TransferResponse {
            transfer_id,
            voter_id: req.voter_id,
            status: "accepted".into(),
        }),
    ))
}
