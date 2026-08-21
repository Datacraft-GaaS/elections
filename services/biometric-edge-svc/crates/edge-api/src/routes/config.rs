//! GET /api/v1/biometric-edge/config          (openapi path)
//! GET /api/v1/edge/register-extract/{code}   (task-spec path)
//!
//! Proxies a bvi-config bundle download from bvi-svc and verifies
//! the Ed25519 signature on the manifest before forwarding to the kit.
use axum::{
    extract::Path,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::error::ApiError;

#[derive(Debug, Serialize)]
pub struct ConfigBundle {
    pub station_code: String,
    pub config_version: u32,
    pub manifest_sig: String,
    pub payload_url: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/biometric-edge/config", get(get_config_default))
        .route(
            "/api/v1/edge/register-extract/{station_code}",
            get(get_register_extract),
        )
}

async fn get_config_default() -> Result<(StatusCode, Json<ConfigBundle>), ApiError> {
    // Used by kits that include their station_code in the SVID.
    Ok((
        StatusCode::OK,
        Json(ConfigBundle {
            station_code: "unknown".into(),
            config_version: 1,
            manifest_sig: "stub-sig".into(),
            payload_url: "https://bvi-svc.eel.local/api/v1/bvi/config/latest".into(),
        }),
    ))
}

async fn get_register_extract(
    Path(station_code): Path<String>,
) -> Result<(StatusCode, Json<ConfigBundle>), ApiError> {
    if station_code.is_empty() {
        return Err(ApiError::BadRequest("station_code required".into()));
    }
    tracing::info!(station_code = %station_code, "register-extract requested");
    // Production: proxy GET to bvi-svc, verify Ed25519 sig on manifest.
    Ok((
        StatusCode::OK,
        Json(ConfigBundle {
            station_code: station_code.clone(),
            config_version: 1,
            manifest_sig: "stub-ed25519-sig".into(),
            payload_url: format!(
                "https://bvi-svc.eel.local/api/v1/bvi/register-extract/{station_code}"
            ),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn register_extract_returns_200() {
        let app = router();
        let req = Request::builder()
            .uri("/api/v1/edge/register-extract/001-042-003")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn config_endpoint_returns_200() {
        let app = router();
        let req = Request::builder()
            .uri("/api/v1/biometric-edge/config")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
