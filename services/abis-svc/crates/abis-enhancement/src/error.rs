use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnhancementError {
    #[error("invalid image dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("insufficient data: {0}")]
    InsufficientData(String),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("segmentation failed: {0}")]
    SegmentationFailed(String),
}
