use axum::{http::StatusCode, routing::post, Json, Router};

use crate::{
    error::ApiError,
    ingest::IngestService,
    models::{BatchIngestionReport, BatchIngestRequest},
};

pub fn router() -> Router {
    Router::new().route("/v1/cvr/batches", post(ingest_batch))
}

async fn ingest_batch(
    Json(req): Json<BatchIngestRequest>,
) -> Result<(StatusCode, Json<BatchIngestionReport>), ApiError> {
    let svc = IngestService::new();
    let report = svc
        .ingest(req)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok((StatusCode::ACCEPTED, Json(report)))
}
