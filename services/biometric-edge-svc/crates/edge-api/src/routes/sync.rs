//! POST /api/v1/edge/sync/pull  — delta-sync pull for offline queue reconciliation.
//! POST /api/v1/edge/sync/ack   — acknowledgement of delivered sync batch.
use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct SyncPullRequest {
    pub device_serial: String,
    /// HLC timestamp of last successful sync (ISO-8601).
    pub since: String,
    /// Maximum events to return.
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SyncPullResponse {
    pub events: Vec<serde_json::Value>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
pub struct SyncAckRequest {
    pub device_serial: String,
    pub cursor: String,
}

#[derive(Debug, Serialize)]
pub struct SyncAckResponse {
    pub acknowledged: bool,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/edge/sync/pull", post(sync_pull))
        .route("/api/v1/edge/sync/ack", post(sync_ack))
}

async fn sync_pull(
    Json(req): Json<SyncPullRequest>,
) -> Result<(StatusCode, Json<SyncPullResponse>), ApiError> {
    if req.device_serial.is_empty() {
        return Err(ApiError::BadRequest("device_serial required".into()));
    }
    tracing::debug!(device_serial = %req.device_serial, since = %req.since, "sync pull");
    Ok((
        StatusCode::OK,
        Json(SyncPullResponse {
            events: vec![],
            next_cursor: None,
            has_more: false,
        }),
    ))
}

async fn sync_ack(
    Json(req): Json<SyncAckRequest>,
) -> Result<(StatusCode, Json<SyncAckResponse>), ApiError> {
    if req.cursor.is_empty() {
        return Err(ApiError::BadRequest("cursor required".into()));
    }
    Ok((StatusCode::OK, Json(SyncAckResponse { acknowledged: true })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn sync_pull_returns_200() {
        let app = router();
        let body = serde_json::json!({
            "device_serial": "BVI-KE-00001",
            "since": "2026-08-20T00:00:00Z",
            "limit": 100
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/sync/pull")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn sync_ack_returns_200() {
        let app = router();
        let body = serde_json::json!({"device_serial": "BVI-KE-00001", "cursor": "cursor-abc"});
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/edge/sync/ack")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
