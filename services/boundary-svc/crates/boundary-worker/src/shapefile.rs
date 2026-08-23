//! Shapefile ingest worker.
//!
//! Polls MinIO bucket `eel-shapefiles` for new NLC/KNBS shapefile archives.
//! On discovery: verifies the file hash, parses geometries, upserts rows into
//! `shapefile_imports`, emits `eel.boundary.shapefile_imported.v1`.

use anyhow::Result;
use tracing::info;

/// Single ingest pass. Idempotent: already-imported file_refs are skipped.
pub async fn run_once() -> Result<()> {
    // Ingest pass prerequisites are not wired in this workspace yet: no
    // S3-compatible list/download client, no SHP+DBF parser crate, and no
    // PgPool (the `shapefile_imports` upsert needs one). Once wired, the pass
    // lists new archives in bucket `eel-shapefiles`, verifies file hashes,
    // parses geometries, upserts rows idempotently on file_ref, and emits
    // `eel.boundary.shapefile_imported.v1`. Until then the worker loop stays
    // healthy with a logged skip.
    info!("shapefile ingest: skipped, S3 client, shapefile parser and PgPool not wired");
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
