use crate::MinexError;
use serde::{Deserialize, Serialize};

/// FMR magic bytes per ISO/IEC 19794-2 / NIST MINEX III.
pub const MINEX_MAGIC: [u8; 4] = [b'F', b'M', b'R', 0x00];

/// Format version 2.0.
pub const MINEX_VERSION: [u8; 4] = [b'2', b'0', 0x00, 0x00];

/// Maximum minutiae count per MINEX III conformance criteria.
pub const MAX_MINUTIAE: usize = 128;

/// MINEX III image dimensions: 500x500 pixels at 500 ppi.
pub const IMAGE_WIDTH: u16 = 500;
pub const IMAGE_HEIGHT: u16 = 500;
pub const IMAGE_RESOLUTION: u16 = 500;

/// A single fingerprint minutia point.
///
/// mtype: 0 = other, 1 = termination (ridge ending), 2 = bifurcation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinexMinutia {
    pub x: u16,
    pub y: u16,
    /// Direction in units of 1.40625 degrees (256 units = 360 deg).
    pub angle: u8,
    /// Quality score 0-100.
    pub quality: u8,
    /// Minutia type: 0=other, 1=termination, 2=bifurcation.
    pub mtype: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinexTemplate {
    /// Finger position 1-10 per ANSI/NIST-ITL.
    pub finger_position: u8,
    pub view_number: u8,
    pub impression_type: u8,
    pub minutiae: Vec<MinexMinutia>,
}

impl MinexTemplate {
    pub fn new(finger_position: u8) -> Self {
        Self {
            finger_position,
            view_number: 0,
            impression_type: 0,
            minutiae: Vec::new(),
        }
    }

    /// Add a minutia after validating count, coordinate, and quality constraints.
    pub fn add_minutia(&mut self, m: MinexMinutia) -> Result<(), MinexError> {
        if self.minutiae.len() >= MAX_MINUTIAE {
            return Err(MinexError::InvalidMinutiaeCount {
                found: self.minutiae.len() + 1,
                max: MAX_MINUTIAE,
            });
        }
        if m.x >= IMAGE_WIDTH || m.y >= IMAGE_HEIGHT {
            return Err(MinexError::InvalidCoordinate { x: m.x, y: m.y });
        }
        if m.quality > 100 {
            return Err(MinexError::InvalidQuality(m.quality));
        }
        self.minutiae.push(m);
        Ok(())
    }

    /// Serialize to FMR wire format.
    ///
    /// Layout:
    ///   [0..4]   FMR magic
    ///   [4..8]   version
    ///   [8..12]  record length (u32 BE)
    ///   [12]     finger_position
    ///   [13]     view_number
    ///   [14]     impression_type
    ///   [15..17] minutiae_count (u16 BE)
    ///   then for each minutia: x(2 BE) + y(2 BE) + angle(1) + (quality<<2|mtype)(1)
    pub fn serialize(&self) -> Vec<u8> {
        let header_size = 17usize;
        let minutia_size = 6usize;
        let record_length = (header_size + self.minutiae.len() * minutia_size) as u32;

        let mut out = Vec::with_capacity(record_length as usize);
        out.extend_from_slice(&MINEX_MAGIC);
        out.extend_from_slice(&MINEX_VERSION);
        out.extend_from_slice(&record_length.to_be_bytes());
        out.push(self.finger_position);
        out.push(self.view_number);
        out.push(self.impression_type);
        let count = self.minutiae.len() as u16;
        out.extend_from_slice(&count.to_be_bytes());
        for m in &self.minutiae {
            // ISO 19794-2 minutia record layout (6 bytes):
            //   bytes 0-1: type(2 bits) | x(14 bits)  -- BE u16
            //   bytes 2-3: reserved(2 bits=0) | y(14 bits) -- BE u16
            //   byte 4:    angle
            //   byte 5:    quality (0-100)
            let x_word = (((m.mtype as u16) & 0x03) << 14) | (m.x & 0x3FFF);
            out.extend_from_slice(&x_word.to_be_bytes());
            let y_word = m.y & 0x3FFF;
            out.extend_from_slice(&y_word.to_be_bytes());
            out.push(m.angle);
            out.push(m.quality);
        }
        out
    }

    /// Parse an FMR-encoded MINEX template.
    pub fn parse(data: &[u8]) -> Result<Self, MinexError> {
        if data.len() < 17 {
            return Err(MinexError::InvalidHeader(format!(
                "too short: {} bytes",
                data.len()
            )));
        }
        if data[..4] != MINEX_MAGIC {
            return Err(MinexError::InvalidHeader(format!(
                "bad magic: {:?}",
                &data[..4]
            )));
        }
        let finger_position = data[12];
        let view_number = data[13];
        let impression_type = data[14];
        let count = u16::from_be_bytes([data[15], data[16]]) as usize;

        if data.len() < 17 + count * 6 {
            return Err(MinexError::InvalidHeader(format!(
                "truncated: need {} bytes for {} minutiae",
                17 + count * 6,
                count
            )));
        }

        let mut minutiae = Vec::with_capacity(count);
        for i in 0..count {
            let base = 17 + i * 6;
            let x_word = u16::from_be_bytes([data[base], data[base + 1]]);
            let mtype = ((x_word >> 14) & 0x03) as u8;
            let x = x_word & 0x3FFF;
            let y_word = u16::from_be_bytes([data[base + 2], data[base + 3]]);
            let y = y_word & 0x3FFF;
            let angle = data[base + 4];
            let quality = data[base + 5];
            minutiae.push(MinexMinutia {
                x,
                y,
                angle,
                quality,
                mtype,
            });
        }

        Ok(Self {
            finger_position,
            view_number,
            impression_type,
            minutiae,
        })
    }
}
