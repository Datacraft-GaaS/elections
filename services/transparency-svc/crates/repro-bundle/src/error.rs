use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReproError {
    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("archive corrupt: {0}")]
    ArchiveCorrupt(String),

    #[error("version mismatch: bundle format {bundle_ver}, parser {parser_ver}")]
    VersionMismatch {
        bundle_ver: String,
        parser_ver: String,
    },

    #[error("assembly already exists for election {election_id} version {version}")]
    AlreadyAssembled {
        election_id: String,
        version: String,
    },

    #[error("component missing: {0}")]
    ComponentMissing(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}
