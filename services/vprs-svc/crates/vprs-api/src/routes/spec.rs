//! Public contract handler (vprs/portal.yaml admin analytics).

use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;

use crate::state::AppState;

/// GET /api/v1/vprs/admin/analytics — VPRS admin analytics.
pub async fn get_vprs_analytics(State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "total_applications": 0,
        "pending": 0,
        "approved": 0,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))
}
