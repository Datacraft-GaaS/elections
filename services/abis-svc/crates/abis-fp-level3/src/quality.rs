//! Quality assessment for Level-3 feature extraction.
//!
//! Reference: Jain, Chen, Demirkus, IEEE TPAMI 2007, Section II.
//!
//! Pore visibility and ridge edge sharpness are only reliable at >= 500 ppi.
//! Below this threshold Level-3 matching should be skipped or heavily weighted down.

use thiserror::Error;

/// Minimum image resolution (ppi) required for reliable Level-3 features.
pub const MIN_RESOLUTION_PPI: u16 = 500;

/// Error returned when image resolution is too low for Level-3 extraction.
#[derive(Debug, Error, PartialEq)]
pub enum Level3QualityError {
    #[error(
        "insufficient resolution: {actual} ppi, need >= {} ppi for Level-3 features",
        MIN_RESOLUTION_PPI
    )]
    InsufficientResolution { actual: u16 },
}

/// Level-3 quality report.
#[derive(Debug, Clone)]
pub struct Level3Quality {
    /// Image resolution in ppi.
    pub resolution_ppi: u16,
    /// Fraction of ridge area where pores are visible (0.0-1.0).
    pub pore_visibility: f32,
    /// Ridge edge sharpness metric (0.0-1.0).
    pub edge_sharpness: f32,
}

impl Level3Quality {
    pub fn new(resolution_ppi: u16, pore_visibility: f32, edge_sharpness: f32) -> Self {
        Self {
            resolution_ppi,
            pore_visibility: pore_visibility.clamp(0.0, 1.0),
            edge_sharpness: edge_sharpness.clamp(0.0, 1.0),
        }
    }

    /// Returns an error if resolution is below the Level-3 threshold.
    pub fn check_resolution(&self) -> Result<(), Level3QualityError> {
        if self.resolution_ppi < MIN_RESOLUTION_PPI {
            Err(Level3QualityError::InsufficientResolution {
                actual: self.resolution_ppi,
            })
        } else {
            Ok(())
        }
    }

    /// Overall quality scalar in [0.0, 1.0].
    pub fn overall(&self) -> f32 {
        (self.pore_visibility * 0.6 + self.edge_sharpness * 0.4).clamp(0.0, 1.0)
    }
}
