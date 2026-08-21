use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("serialisation error: {0}")]
    Serialise(#[from] serde_json::Error),
    #[error("action not found: {0}")]
    NotFound(String),
    #[error("duplicate 34B detected for constituency {0}")]
    Duplicate34B(String),
}

// Tauri requires serialisable errors for IPC commands.
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
