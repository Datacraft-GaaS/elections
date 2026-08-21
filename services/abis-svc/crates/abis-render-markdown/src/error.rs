use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderMarkdownError {
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("template: {0}")]
    Template(#[from] tera::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T, E = RenderMarkdownError> = std::result::Result<T, E>;
