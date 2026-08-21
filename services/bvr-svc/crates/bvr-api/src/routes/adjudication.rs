use axum::{extract::Path, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct AdjudicationResolveRequest {
    /// "accept_primary" | "accept_secondary" | "reject_both"
    pub outcome: String,
    pub notes: Option<String>,
    pub resolved_by: String,
}

#[derive(Debug, Serialize)]
pub struct AdjudicationResolveResponse {
    pub case_id: String,
    pub outcome: String,
    pub status: String,
}

pub fn router() -> Router {
    Router::new().route(
        "/api/v1/bvr/adjudication/{case_id}/resolve",
        post(resolve_adjudication),
    )
}

async fn resolve_adjudication(
    Path(case_id): Path<String>,
    Json(req): Json<AdjudicationResolveRequest>,
) -> Result<Json<AdjudicationResolveResponse>, ApiError> {
    Ok(Json(AdjudicationResolveResponse {
        case_id,
        outcome: req.outcome,
        status: "resolved".into(),
    }))
}
