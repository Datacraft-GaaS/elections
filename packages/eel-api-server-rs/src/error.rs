use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiServerError {
    #[error("observability init failed: {0}")]
    Observability(#[from] eel_observability_rs::ObservabilityError),

    #[error("auth configuration error: {0}")]
    Auth(String),

    #[error("server bind error: {0}")]
    Bind(#[from] std::io::Error),

    #[error("server error: {0}")]
    Runtime(String),
}
