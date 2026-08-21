//! Shapefile ingest worker.
//!
//! Polls MinIO bucket `eel-shapefiles` for new NLC/KNBS shapefile archives.
//! On discovery: verifies the file hash, parses geometries, upserts rows into
//! `shapefile_imports`, emits `eel.boundary.shapefile_imported.v1`.

use anyhow::Result;
use tracing::info;

/// Single ingest pass. Idempotent: already-imported file_refs are skipped.
pub async fn run_once() -> Result<()> {
    // TODO(WP-2.4): connect to MinIO via S3-compatible client, list new
    // shapefile archives, parse SHP+DBF, upsert boundaries, emit event.
    info!("shapefile ingest: no-op stub");
    Ok(())
}

/// Validate that a shapefile import has a non-empty file_ref (ingest guard).
pub fn validate_import(file_ref: &str) -> Result<(), crate::shapefile::ImportError> {
    if file_ref.is_empty() {
        return Err(ImportError::EmptyFileRef);
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("file_ref must not be empty")]
    EmptyFileRef,
    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("parse error: {reason}")]
    ParseError { reason: String },
}
