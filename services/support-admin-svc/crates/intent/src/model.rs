//! On-device pre-mark validation model registry hook (M28 integration).
//!
//! The actual model artefact is managed by ai-platform-svc / M28 registry.
//! This module owns the registration descriptor and the image-delete invariant.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum allowed elapsed time (ms) from image capture to secure delete.
/// Enforced on-device; also asserted by the PreMarkValidationEvent payload.
pub const IMAGE_DELETE_DEADLINE_MS: u32 = 1_000;

/// Registration descriptor for the pre-mark validation model in M28 registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreMarkModelDescriptor {
    pub model_id: String,
    pub version: String,
    /// sha256 of the on-device ONNX/TFLite artefact.
    pub artefact_sha256: String,
    pub registered_at: String,
    pub registry_entry_id: String,
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error(
        "image-delete deadline exceeded: elapsed {elapsed_ms} ms > {IMAGE_DELETE_DEADLINE_MS} ms"
    )]
    ImageDeleteDeadlineExceeded { elapsed_ms: u32 },
    #[error("model version mismatch: expected '{expected}', got '{got}'")]
    VersionMismatch { expected: String, got: String },
}

/// Assert the image-delete invariant on a validation event payload.
/// Called server-side when the PreMarkValidationEvent is ingested.
pub fn assert_image_deleted(elapsed_ms: u32) -> Result<(), ModelError> {
    if elapsed_ms > IMAGE_DELETE_DEADLINE_MS {
        return Err(ModelError::ImageDeleteDeadlineExceeded { elapsed_ms });
    }
    Ok(())
}

/// Validate that the event's model version matches the registered descriptor.
pub fn assert_model_version(
    descriptor: &PreMarkModelDescriptor,
    reported_version: &str,
) -> Result<(), ModelError> {
    if descriptor.version != reported_version {
        return Err(ModelError::VersionMismatch {
            expected: descriptor.version.clone(),
            got: reported_version.to_string(),
        });
    }
    Ok(())
}
