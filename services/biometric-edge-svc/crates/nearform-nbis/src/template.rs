//! Minutia and template — the output of NBIS extraction, input to NBIS matching.

use serde::{Deserialize, Serialize};

/// A single minutia point: ridge ending or bifurcation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Minutia {
    /// X coordinate in pixels (NBIS internal: 0..width).
    pub x: u32,
    /// Y coordinate in pixels (NBIS internal: 0..height).
    pub y: u32,
    /// Angle in degrees (0..360). NBIS uses 0..1800 (tenths of degrees) internally.
    pub theta_deg: f32,
    /// Quality 0..100 (NBIS reliability score).
    pub quality: u32,
    /// Type of minutia.
    pub mclass: MinutiaType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinutiaType {
    /// Ridge ending.
    RidgeEnd,
    /// Ridge bifurcation (forks into two).
    Bifurcation,
    /// Excluded by quality/heuristics (retained for debugging).
    DontCare,
}

impl MinutiaType {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::RidgeEnd,
            1 => Self::Bifurcation,
            _ => Self::DontCare,
        }
    }
}

/// A complete fingerprint template: array of minutiae + image dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub width: u32,
    pub height: u32,
    pub minutiae: Vec<Minutia>,
}

impl Template {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, minutiae: Vec::new() }
    }

    pub fn add(&mut self, m: Minutia) {
        self.minutiae.push(m);
    }

    pub fn count(&self) -> usize {
        self.minutiae.len()
    }

    /// Filter to keep only high-quality minutiae (default ≥ 50).
    pub fn filter_quality(&self, min_q: u32) -> Self {
        Self {
            width: self.width,
            height: self.height,
            minutiae: self.minutiae.iter().copied()
                .filter(|m| m.quality >= min_q)
                .collect(),
        }
    }

    /// Serialize to NBIS .xyt format: `x y theta quality` per line.
    pub fn to_xyt(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# NBIS .xyt (Rust port)\n"));
        out.push_str(&format!("{} {}\n", self.width, self.height));
        for m in &self.minutiae {
            out.push_str(&format!("{} {} {} {}\n", m.x, m.y, (m.theta_deg * 10.0) as i32, m.quality));
        }
        out
    }

    /// Parse from NBIS .xyt text format.
    pub fn from_xyt(s: &str) -> Result<Self, String> {
        let mut t = Template::new(0, 0);
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            // First non-comment line may be "WIDTH HEIGHT" header
            if parts.len() == 2 && t.width == 0 {
                t.width = parts[0].parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
                t.height = parts[1].parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
                continue;
            }
            if parts.len() < 3 { continue; }
            let x: u32 = parts[0].parse().map_err(|e| e.to_string())?;
            let y: u32 = parts[1].parse().map_err(|e| e.to_string())?;
            let theta_raw: i32 = parts[2].parse().map_err(|e| e.to_string())?;
            let quality: u32 = if parts.len() > 3 {
                parts[3].parse().unwrap_or(0)
            } else {
                50
            };
            t.add(Minutia {
                x, y,
                theta_deg: theta_raw as f32 / 10.0,
                quality,
                mclass: MinutiaType::DontCare,
            });
        }
        Ok(t)
    }
}
