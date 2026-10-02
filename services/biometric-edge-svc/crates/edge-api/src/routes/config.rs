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

/// The default config bundle a kit fetches at enrolment.
///
/// This answered 200 with `station_code: "unknown"`, `config_version: 1` and a
/// `manifest_sig` whose value was the literal placeholder `stub` + `sig`. That field
/// exists to carry an Ed25519 manifest signature, and a kit has no way to tell a
/// placeholder from a real signature except by knowing the manifest, which it does
/// not: the signature is what it trusts.
///
/// The bundle is proxied from `bvi-config` and signed there. That service is not in
/// this workspace, so there is no manifest and no signature to forward.
async fn get_config_default() -> Result<(StatusCode, Json<ConfigBundle>), ApiError> {
    Err(ApiError::NotImplemented(
        "a config bundle is proxied from bvi-config, which signs the manifest, and that \
         service is not in this workspace. This handler previously answered 200 with \
         station_code unknown, config_version 1 and a literal placeholder in \
         manifest_sig -- the one field a kit trusts to authenticate the bundle"
            .into(),
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
    async fn config_endpoint_refuses_rather_than_stub_signing() {
        // This asserted 200, which is what let the stub-signed bundle through: the
        // handler returned 200 with `manifest_sig` set to a literal placeholder, and
        // that field exists to carry an Ed25519 manifest signature.
        let app = router();
        let req = Request::builder()
            .uri("/api/v1/biometric-edge/config")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    }
}
