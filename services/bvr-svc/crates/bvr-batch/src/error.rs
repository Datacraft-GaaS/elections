use thiserror::Error;

#[derive(Debug, Error)]
pub enum BatchError {
    #[error("row validation: {0}")]
    RowValidation(String),
    #[error("database: {0}")]
    Database(String),
    #[error("export failed: {0}")]
    Export(String),
    #[error("duplicate approval by: {0}")]
    DuplicateApproval(String),
}
