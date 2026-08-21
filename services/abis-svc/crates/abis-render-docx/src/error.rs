use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderDocxError {
    #[error("docx build: {0}")]
    DocxBuild(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("template: {0}")]
    Template(#[from] tera::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T, E = RenderDocxError> = std::result::Result<T, E>;
