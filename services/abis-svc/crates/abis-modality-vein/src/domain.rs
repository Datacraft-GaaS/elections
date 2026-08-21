use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VeinType {
    FingerVein,
    PalmVein,
    WristVein,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodySide {
    R,
    L,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeinTemplate {
    pub template_id: String,
    pub vein_type: VeinType,
    pub body_side: BodySide,
    pub nir_wavelength_nm: u16,
    pub width: u16,
    pub height: u16,
    /// Grayscale NIR image bytes (width * height).
    pub pattern_bitmap: Vec<u8>,
    pub captured_at: DateTime<Utc>,
}
