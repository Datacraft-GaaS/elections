//! GET /api/v1/bvi/register/anchors (wave-2 N2).
//!
//! Serves register-bundle anchor views from the env-provisioned registry
//! cached at startup; kits fetch once at poll-open and verify offline.

use axum::{
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use bvi_register_anchor::{parse_registry, BundleView};

/// Cached at first call from `EEL_REGISTER_BUNDLES`; stable for the process
/// lifetime so poll-day operation needs no upstream reachability.
fn registry() -> &'static Vec<(String, String)> {
    static REG: std::sync::OnceLock<Vec<(String, String)>> = std::sync::OnceLock::new();
    REG.get_or_init(|| {
        match parse_registry(&std::env::var("EEL_REGISTER_BUNDLES").unwrap_or_default()) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "EEL_REGISTER_BUNDLES malformed");
                Vec::new()
            }
        }
    })
}

pub async fn get_register_anchors(headers: HeaderMap) -> Response {
    if let Err(e) = crate::idem::require_bearer(&headers, "/api/v1/bvi/register/anchors") {
        return e.into_response();
    }

    let views: Vec<BundleView> = registry()
        .iter()
        .map(|(id, root)| BundleView {
            bundle_id: id.clone(),
            merkle_root_hex: root.clone(),
            log_anchor: String::new(),
            from_cache: true,
        })
        .collect();

    Json(serde_json::json!({ "bundles": views })).into_response()
}
