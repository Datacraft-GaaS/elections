use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use base64::Engine;
use serde::Serialize;

/// Canonical response headers added by the signing layer.
pub const HEADER_SIGNATURE: &str = "x-eel-signature";
pub const HEADER_MERKLE_PROOF: &str = "x-eel-merkle-proof";
pub const HEADER_SNAPSHOT_ID: &str = "x-eel-snapshot-id";

/// Detached cosign-style signature over canonical JSON body.
///
/// Placeholder: base64(hash) so tests can assert the header exists.
pub fn sign_body(body_bytes: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    body_bytes.hash(&mut h);
    let digest = h.finish().to_le_bytes();
    base64::engine::general_purpose::STANDARD.encode(digest)
}

/// Build a Merkle inclusion proof header value (placeholder).
pub fn merkle_proof_header(snapshot_id: &str, leaf_index: usize) -> String {
    format!("snapshot={snapshot_id};leaf={leaf_index};proof=")
}

/// JSON response with optional response-signing headers.
pub struct SignedJson<T: Serialize> {
    pub payload: T,
    pub snapshot_id: Option<String>,
    pub leaf_index: Option<usize>,
    pub status: http::StatusCode,
}

impl<T: Serialize> SignedJson<T> {
    pub fn ok(payload: T) -> Self {
        Self {
            payload,
            snapshot_id: None,
            leaf_index: None,
            status: http::StatusCode::OK,
        }
    }

    pub fn with_snapshot(mut self, snapshot_id: impl Into<String>, leaf: usize) -> Self {
        self.snapshot_id = Some(snapshot_id.into());
        self.leaf_index = Some(leaf);
        self
    }
}

impl<T: Serialize> IntoResponse for SignedJson<T> {
    fn into_response(self) -> Response {
        let body = match serde_json::to_vec(&self.payload) {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("SignedJson serialization error: {e}");
                return (
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    "serialization error",
                )
                    .into_response();
            }
        };

        let sig = sign_body(&body);

        let mut builder = Response::builder()
            .status(self.status)
            .header(http::header::CONTENT_TYPE, "application/json")
            .header(http::HeaderName::from_static(HEADER_SIGNATURE), sig);

        if let (Some(sid), Some(leaf)) = (self.snapshot_id, self.leaf_index) {
            builder = builder
                .header(
                    http::HeaderName::from_static(HEADER_SNAPSHOT_ID),
                    sid.clone(),
                )
                .header(
                    http::HeaderName::from_static(HEADER_MERKLE_PROOF),
                    merkle_proof_header(&sid, leaf),
                );
        }

        builder.body(Body::from(body)).unwrap().into_response()
    }
}
