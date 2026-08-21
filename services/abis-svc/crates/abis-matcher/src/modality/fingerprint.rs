//! Fingerprint template: ISO/IEC 19794-2 minutiae records + NFIQ2-style quality.

use std::f32::consts::PI;

use thiserror::Error;

use super::{Modality, Template};

/// ANSI 378 / ISO 19794-2 finger positions 1-10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum FingerPosition {
    RightThumb = 1,
    RightIndex = 2,
    RightMiddle = 3,
    RightRing = 4,
    RightLittle = 5,
    LeftThumb = 6,
    LeftIndex = 7,
    LeftMiddle = 8,
    LeftRing = 9,
    LeftLittle = 10,
}

impl TryFrom<u8> for FingerPosition {
    type Error = ParseError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(Self::RightThumb),
            2 => Ok(Self::RightIndex),
            3 => Ok(Self::RightMiddle),
            4 => Ok(Self::RightRing),
            5 => Ok(Self::RightLittle),
            6 => Ok(Self::LeftThumb),
            7 => Ok(Self::LeftIndex),
            8 => Ok(Self::LeftMiddle),
            9 => Ok(Self::LeftRing),
            10 => Ok(Self::LeftLittle),
            _ => Err(ParseError::InvalidFingerPosition(v)),
        }
    }
}

/// ISO 19794-2 minutia type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum MinutiaeType {
    Ending = 1,
    Bifurcation = 2,
}

impl TryFrom<u8> for MinutiaeType {
    type Error = ParseError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(Self::Ending),
            2 => Ok(Self::Bifurcation),
            _ => Err(ParseError::InvalidMinutiaeType(v)),
        }
    }
}

/// Single minutia record.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Minutia {
    /// x coordinate in pixels (14 bits).
    pub x: u16,
    /// y coordinate in pixels (14 bits).
    pub y: u16,
    /// Orientation angle in degrees [0, 360).
    pub theta: f32,
    /// Type: ending or bifurcation.
    pub mtype: MinutiaeType,
    /// Per-minutia quality [0, 100].
    pub quality: u8,
}

/// Parse/serialise errors for ISO 19794-2 records.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("buffer too short: need {need} bytes, have {have}")]
    TooShort { need: usize, have: usize },
    #[error("invalid magic bytes")]
    BadMagic,
    #[error("invalid finger position: {0}")]
    InvalidFingerPosition(u8),
    #[error("invalid minutiae type: {0}")]
    InvalidMinutiaeType(u8),
    #[error("serialise error: {0}")]
    Serialise(String),
}

/// Fingerprint template with minutiae list and ridge-count matrix.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FingerprintTemplate {
    pub position: FingerPosition,
    pub impression_type: u8,
    pub image_width: u16,
    pub image_height: u16,
    pub x_resolution: u16,
    pub y_resolution: u16,
    pub minutiae: Vec<Minutia>,
    /// Ridge count matrix: ridge_counts[i][j] = approximate ridge count between minutia i and j.
    /// Stored as a flat lower-triangle (only j < i).
    pub ridge_counts: Vec<u8>,
    pub quality_score: u8,
}

impl FingerprintTemplate {
    /// Construct directly from a minutiae list (for tests / synthetic templates).
    pub fn from_minutiae(position: FingerPosition, minutiae: Vec<Minutia>) -> Self {
        let n = minutiae.len();
        let ridge_counts = vec![0u8; n * (n.saturating_sub(1)) / 2];
        let quality = quality_from_minutiae(&minutiae, 500, 500);
        Self {
            position,
            impression_type: 0,
            image_width: 500,
            image_height: 500,
            x_resolution: 500,
            y_resolution: 500,
            quality_score: quality,
            ridge_counts,
            minutiae,
        }
    }

    /// Parse a compact ISO 19794-2 binary record.
    ///
    /// Layout (compact form, no CBEFF wrapper):
    /// ```text
    /// [0..2]  magic 'FM' (0x46 0x4D)
    /// [2..4]  format version '20' (0x32 0x30)
    /// [4..8]  total length (u32 BE)
    /// [8..10] capture device ID (u16 BE)
    /// [10..12] image width  (u16 BE)
    /// [12..14] image height (u16 BE)
    /// [14..16] x resolution (u16 BE)
    /// [16..18] y resolution (u16 BE)
    /// [18]    number of finger views
    /// [19]    reserved
    /// -- per finger view --
    /// [0]  finger position (u8)
    /// [1]  view number | impression type (nibbles)
    /// [2]  finger quality (u8)
    /// [3]  minutia count (u8)
    /// -- per minutia (6 bytes) --
    /// [0..2] type(2 bits) | x(14 bits) BE
    /// [2..4] reserved(2 bits) | y(14 bits) BE
    /// [4]    theta (u8, 0..255 maps to 0..360)
    /// [5]    quality (u8)
    /// -- end per minutia --
    /// [last 4 bytes of view] extended data length (u16) + data
    /// ```
    pub fn parse_iso19794_2(bytes: &[u8]) -> Result<Vec<Self>, ParseError> {
        const MIN_HDR: usize = 20;
        if bytes.len() < MIN_HDR {
            return Err(ParseError::TooShort {
                need: MIN_HDR,
                have: bytes.len(),
            });
        }
        // Allow CBEFF preamble: if first 4 bytes are not 'FM20', skip to offset where 'FM' appears.
        let start = if &bytes[0..2] == b"FM" {
            0
        } else {
            bytes
                .windows(2)
                .position(|w| w == b"FM")
                .ok_or(ParseError::BadMagic)?
        };
        let b = &bytes[start..];
        if b.len() < MIN_HDR {
            return Err(ParseError::TooShort {
                need: MIN_HDR,
                have: b.len(),
            });
        }
        if &b[0..2] != b"FM" {
            return Err(ParseError::BadMagic);
        }
        let image_width = u16::from_be_bytes([b[10], b[11]]);
        let image_height = u16::from_be_bytes([b[12], b[13]]);
        let x_res = u16::from_be_bytes([b[14], b[15]]);
        let y_res = u16::from_be_bytes([b[16], b[17]]);
        let num_views = b[18] as usize;

        let mut pos = MIN_HDR;
        let mut templates = Vec::with_capacity(num_views);

        for _ in 0..num_views {
            if b.len() < pos + 4 {
                return Err(ParseError::TooShort {
                    need: pos + 4,
                    have: b.len(),
                });
            }
            let finger_pos = FingerPosition::try_from(b[pos])?;
            let impression_type = b[pos + 1] & 0x0F;
            let finger_quality = b[pos + 2];
            let minutia_count = b[pos + 3] as usize;
            pos += 4;

            if b.len() < pos + minutia_count * 6 {
                return Err(ParseError::TooShort {
                    need: pos + minutia_count * 6,
                    have: b.len(),
                });
            }

            let mut minutiae = Vec::with_capacity(minutia_count);
            for _ in 0..minutia_count {
                let word0 = u16::from_be_bytes([b[pos], b[pos + 1]]);
                let word1 = u16::from_be_bytes([b[pos + 2], b[pos + 3]]);
                let mtype_raw = ((word0 >> 14) & 0x03) as u8;
                let x = word0 & 0x3FFF;
                let y = word1 & 0x3FFF;
                let theta_raw = b[pos + 4];
                let quality = b[pos + 5];
                pos += 6;

                let mtype = if mtype_raw == 2 {
                    MinutiaeType::Bifurcation
                } else {
                    MinutiaeType::Ending
                };
                let theta = (theta_raw as f32 / 256.0) * 360.0;
                minutiae.push(Minutia {
                    x,
                    y,
                    theta,
                    mtype,
                    quality,
                });
            }
            // Skip extended data block (2-byte length + data).
            if b.len() >= pos + 2 {
                let ext_len = u16::from_be_bytes([b[pos], b[pos + 1]]) as usize;
                pos += 2 + ext_len;
            }

            let quality =
                finger_quality.max(quality_from_minutiae(&minutiae, image_width, image_height));
            let n = minutiae.len();
            let ridge_counts = vec![0u8; n * (n.saturating_sub(1)) / 2];
            templates.push(FingerprintTemplate {
                position: finger_pos,
                impression_type,
                image_width,
                image_height,
                x_resolution: x_res,
                y_resolution: y_res,
                minutiae,
                ridge_counts,
                quality_score: quality,
            });
        }
        Ok(templates)
    }

    /// Serialise to compact ISO 19794-2 binary format (single finger view).
    pub fn serialise_iso19794_2(&self) -> Vec<u8> {
        let minutia_count = self.minutiae.len();
        // header (20) + view header (4) + minutiae (6 each) + ext_data (2)
        let total = 20 + 4 + minutia_count * 6 + 2;
        let mut buf = vec![0u8; total];

        // Global header
        buf[0..2].copy_from_slice(b"FM");
        buf[2..4].copy_from_slice(b"20");
        let total_u32 = total as u32;
        buf[4..8].copy_from_slice(&total_u32.to_be_bytes());
        // capture device = 0
        // image dimensions
        buf[10..12].copy_from_slice(&self.image_width.to_be_bytes());
        buf[12..14].copy_from_slice(&self.image_height.to_be_bytes());
        buf[14..16].copy_from_slice(&self.x_resolution.to_be_bytes());
        buf[16..18].copy_from_slice(&self.y_resolution.to_be_bytes());
        buf[18] = 1; // one finger view
        buf[19] = 0; // reserved

        // Finger view header
        let mut p = 20;
        buf[p] = self.position as u8;
        buf[p + 1] = self.impression_type & 0x0F;
        buf[p + 2] = self.quality_score;
        buf[p + 3] = minutia_count.min(255) as u8;
        p += 4;

        for m in &self.minutiae {
            let mtype_bits: u16 = match m.mtype {
                MinutiaeType::Ending => 1,
                MinutiaeType::Bifurcation => 2,
            };
            let word0: u16 = (mtype_bits << 14) | (m.x & 0x3FFF);
            let word1: u16 = m.y & 0x3FFF;
            let theta_raw = ((m.theta / 360.0) * 256.0) as u8;
            buf[p..p + 2].copy_from_slice(&word0.to_be_bytes());
            buf[p + 2..p + 4].copy_from_slice(&word1.to_be_bytes());
            buf[p + 4] = theta_raw;
            buf[p + 5] = m.quality;
            p += 6;
        }
        // No extended data
        buf[p] = 0;
        buf[p + 1] = 0;
        buf
    }
}

/// NFIQ2-style quality stub: score from minutiae density and spatial spread.
/// Returns a [0, 100] quality value.
pub fn quality_from_minutiae(minutiae: &[Minutia], width: u16, height: u16) -> u8 {
    if minutiae.is_empty() {
        return 0;
    }
    let w = width.max(1) as f32;
    let h = height.max(1) as f32;
    let area = w * h;

    // Optimal density: ~40 minutiae per 500x500 = 40/250000
    let density = minutiae.len() as f32 / area;
    let optimal_density = 40.0 / 250_000.0;
    let density_score = (density / optimal_density).min(1.0);

    // Spatial spread: standard deviation of x and y coordinates.
    let n = minutiae.len() as f32;
    let mean_x = minutiae.iter().map(|m| m.x as f32).sum::<f32>() / n;
    let mean_y = minutiae.iter().map(|m| m.y as f32).sum::<f32>() / n;
    let var_x = minutiae
        .iter()
        .map(|m| (m.x as f32 - mean_x).powi(2))
        .sum::<f32>()
        / n;
    let var_y = minutiae
        .iter()
        .map(|m| (m.y as f32 - mean_y).powi(2))
        .sum::<f32>()
        / n;
    let spread = ((var_x + var_y).sqrt()) / ((w + h) / 2.0);
    let spread_score = (spread / 0.3).min(1.0);

    // Average per-minutia quality
    let avg_q = minutiae.iter().map(|m| m.quality as f32).sum::<f32>() / (n * 100.0);

    let score = (density_score * 0.4 + spread_score * 0.4 + avg_q * 0.2) * 100.0;
    score.clamp(0.0, 100.0) as u8
}

impl Template for FingerprintTemplate {
    fn modality() -> Modality {
        Modality::Fingerprint
    }

    fn quality(&self) -> f32 {
        self.quality_score as f32 / 100.0
    }

    fn feature_bytes(&self) -> Vec<u8> {
        self.serialise_iso19794_2()
    }

    fn normalise(&self) -> Self {
        // Normalisation: translate centroid to (image_width/2, image_height/2).
        if self.minutiae.is_empty() {
            return self.clone();
        }
        let n = self.minutiae.len() as f32;
        let cx = self.minutiae.iter().map(|m| m.x as f32).sum::<f32>() / n;
        let cy = self.minutiae.iter().map(|m| m.y as f32).sum::<f32>() / n;
        let dx = (self.image_width as f32 / 2.0) - cx;
        let dy = (self.image_height as f32 / 2.0) - cy;

        let minutiae = self
            .minutiae
            .iter()
            .map(|m| {
                let nx = ((m.x as f32 + dx).clamp(0.0, self.image_width as f32 - 1.0)) as u16;
                let ny = ((m.y as f32 + dy).clamp(0.0, self.image_height as f32 - 1.0)) as u16;
                Minutia { x: nx, y: ny, ..*m }
            })
            .collect();

        FingerprintTemplate {
            minutiae,
            ..self.clone()
        }
    }
}

/// Rotate angle by `delta_degrees`, keeping result in [0, 360).
#[inline]
pub fn rotate_angle(theta: f32, delta: f32) -> f32 {
    (theta + delta).rem_euclid(360.0)
}

/// Angular difference in [-180, 180].
#[inline]
pub fn angular_diff(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 {
        d - 360.0
    } else {
        d
    }
}

/// Convert degrees to radians.
#[inline]
pub fn deg_to_rad(d: f32) -> f32 {
    d * PI / 180.0
}
