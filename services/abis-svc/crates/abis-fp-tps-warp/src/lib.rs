//! Thin-Plate Spline non-rigid distortion correction for fingerprint matching.
//!
//! Reference: F. L. Bookstein, "Principal warps: thin-plate splines and the decomposition of
//! deformations," IEEE TPAMI, vol. 11, no. 6, pp. 567-585, 1989.
//!
//! TPS is the minimum-bending interpolant between two sets of control points.
//! Applied to fingerprints: fit TPS from matched minutia pairs (probe -> gallery),
//! warp probe minutiae, then re-run minutiae matching on aligned data.
#![warn(missing_docs)]

pub mod matcher;
pub mod register;
pub mod tps;

pub use matcher::TpsWarpMatcher;
pub use register::register_minutiae;
pub use tps::TpsWarp;

use thiserror::Error;

/// A 2-D point (f64 for TPS numerical stability).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A minutia (compatible with ISO 19794-2 attributes used in abis-matcher).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Minutia {
    pub x: f64,
    pub y: f64,
    /// Angle in radians.
    pub theta: f64,
    /// Quality 0-100.
    pub quality: u8,
}

impl Minutia {
    pub fn new(x: f64, y: f64, theta: f64, quality: u8) -> Self {
        Self {
            x,
            y,
            theta,
            quality,
        }
    }

    pub fn position(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Warp the position using a fitted TPS, keeping theta unchanged.
    pub fn warped(&self, tps: &TpsWarp) -> Self {
        let (wx, wy) = tps.apply(self.x, self.y);
        Self {
            x: wx,
            y: wy,
            theta: self.theta,
            quality: self.quality,
        }
    }
}

/// Errors from this crate.
#[derive(Debug, Error)]
pub enum TpsError {
    #[error("need at least 3 control points to fit a TPS, got {0}")]
    TooFewPoints(usize),
    #[error("source and target point counts differ: {src} vs {target}")]
    PointCountMismatch { src: usize, target: usize },
    #[error("TPS linear system is singular")]
    SingularSystem,
    #[error("registration failed: no initial correspondences found")]
    NoCorrespondences,
}
