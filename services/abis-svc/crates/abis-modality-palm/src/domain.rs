//! Domain types for palm biometric templates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Which palm surface was captured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PalmPosition {
    RightFull,
    LeftFull,
    RightWriter,
    LeftWriter,
    RightLower,
    LeftLower,
    RightUpper,
    LeftUpper,
}

/// A single ridge feature point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minutia {
    pub x: u16,
    pub y: u16,
    /// Orientation angle in 0-179 degree range (2-degree steps).
    pub angle: u8,
    /// Quality estimate 0-100.
    pub quality: u8,
    pub minutia_type: MinutiaType,
}

/// Classification of a minutia feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MinutiaType {
    RidgeEnding,
    Bifurcation,
}

/// A labeled region of the palm with its associated minutiae.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalmZone {
    pub zone: PalmZoneId,
    pub minutiae: Vec<Minutia>,
}

/// Anatomical zone identifier for palm segmentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PalmZoneId {
    Writer,
    Upper,
    Lower,
    Thenar,
    Hypothenar,
}

/// A full palm biometric template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalmTemplate {
    pub template_id: String,
    pub palm_position: PalmPosition,
    pub minutiae: Vec<Minutia>,
    /// Raw crease pattern bytes.
    pub crease_pattern: Vec<u8>,
    /// Optional orientation field bytes.
    pub orientation_field: Option<Vec<u8>>,
    /// NFIQ2 palm quality score if available.
    pub nfiq2_palm: Option<f32>,
    pub captured_at: DateTime<Utc>,
    pub zones: Vec<PalmZone>,
}
