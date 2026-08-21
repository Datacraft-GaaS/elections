use axum::{http::StatusCode, routing::post, Json, Router};
use bvr_batch::sar::SarRequest;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct SarCreateRequest {
    pub voter_id: String,
    pub requester_national_id: String,
}

#[derive(Debug, Serialize)]
pub struct SarCreateResponse {
    pub sar_id: String,
    pub status: String,
}

pub fn router() -> Router {
    Router::new().route("/api/v1/bvr/sar", post(create_sar))
}

async fn create_sar(
    Json(req): Json<SarCreateRequest>,
) -> Result<(StatusCode, Json<SarCreateResponse>), ApiError> {
    let sar = SarRequest::new(req.voter_id, req.requester_national_id);
    Ok((
        StatusCode::ACCEPTED,
        Json(SarCreateResponse {
            sar_id: sar.sar_id,
            status: "pending".into(),
        }),
    ))
}
