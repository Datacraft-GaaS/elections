use thiserror::Error;

#[derive(Debug, Error)]
pub enum RlaError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("ballot not found: {0}")]
    NotFound(String),
    #[error("sync conflict: {0}")]
    SyncConflict(String),
}

impl serde::Serialize for RlaError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
