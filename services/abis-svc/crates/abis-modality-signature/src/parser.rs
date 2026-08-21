use byteorder::{LittleEndian, ReadBytesExt};
use chrono::Utc;
use std::io::{Cursor, Read};
use uuid::Uuid;

use crate::domain::{SignatureSample, SignatureTemplate};

const MAGIC: [u8; 3] = [0x53, 0x49, 0x47];
const VERSION: u8 = 1;

#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("buffer too short: need {need}, got {got}")]
    BufferTooShort { need: usize, got: usize },
    #[error("invalid magic")]
    InvalidMagic,
    #[error("unsupported version: {0}")]
    UnsupportedVersion(u8),
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}

/// Minimum header: 3 magic + 1 version + 4 sample_count + 2 sample_rate = 10 bytes.
const HEADER_LEN: usize = 10;

pub fn parse_signature(data: &[u8]) -> Result<SignatureTemplate, SignatureError> {
    if data.len() < HEADER_LEN {
        return Err(SignatureError::BufferTooShort {
            need: HEADER_LEN,
            got: data.len(),
        });
    }

    if &data[0..3] != MAGIC {
        return Err(SignatureError::InvalidMagic);
    }

    let version = data[3];
    if version != VERSION {
        return Err(SignatureError::UnsupportedVersion(version));
    }

    let mut cur = Cursor::new(&data[4..]);
    let sample_count = cur.read_u32::<LittleEndian>()? as usize;
    let sample_rate_hz = cur.read_u16::<LittleEndian>()?;

    // Arrays: pressure (4n), velocity (4n), acceleration (4n), pen-up bitmap (ceil(n/8)).
    let pen_up_bytes = (sample_count + 7) / 8;
    let arrays_len = sample_count * 4 * 3 + pen_up_bytes;
    let needed = HEADER_LEN + arrays_len;

    if data.len() < needed {
        return Err(SignatureError::BufferTooShort {
            need: needed,
            got: data.len(),
        });
    }

    let base = 4 + 6; // 4 header bytes after magic+version, 6 bytes for count+rate
    let mut arr_cur = Cursor::new(&data[base..]);

    let mut pressures = vec![0f32; sample_count];
    let mut velocities = vec![0f32; sample_count];
    let mut accelerations = vec![0f32; sample_count];

    for v in pressures.iter_mut() {
        *v = arr_cur.read_f32::<LittleEndian>()?;
    }
    for v in velocities.iter_mut() {
        *v = arr_cur.read_f32::<LittleEndian>()?;
    }
    for v in accelerations.iter_mut() {
        *v = arr_cur.read_f32::<LittleEndian>()?;
    }

    let mut pen_up_buf = vec![0u8; pen_up_bytes];
    arr_cur.read_exact(&mut pen_up_buf)?;

    let mut samples = Vec::with_capacity(sample_count);
    let mut pen_up_count = 0u16;
    let mut stroke_count = 0u16;
    let mut last_was_up = true;

    for i in 0..sample_count {
        let byte_idx = i / 8;
        let bit_idx = i % 8;
        let pen_up = (pen_up_buf[byte_idx] >> bit_idx) & 1 == 1;

        if pen_up {
            pen_up_count += 1;
        }
        if !pen_up && last_was_up {
            stroke_count += 1;
        }
        last_was_up = pen_up;

        let timestamp_ms = (i as u32) * (1000 / sample_rate_hz.max(1) as u32);
        samples.push(SignatureSample {
            timestamp_ms,
            pressure: pressures[i],
            velocity: velocities[i],
            acceleration: accelerations[i],
            pen_up,
        });
    }

    let total_duration_ms = samples.last().map(|s| s.timestamp_ms).unwrap_or(0);

    Ok(SignatureTemplate {
        template_id: Uuid::new_v4().to_string(),
        capture_device: None,
        samples,
        pen_up_count,
        sample_rate_hz,
        total_duration_ms,
        stroke_count,
        captured_at: Utc::now(),
    })
}

pub fn serialize_signature(template: &SignatureTemplate) -> Vec<u8> {
    let n = template.samples.len();
    let pen_up_bytes = (n + 7) / 8;
    let mut buf = Vec::with_capacity(HEADER_LEN + n * 12 + pen_up_bytes);

    buf.extend_from_slice(&MAGIC);
    buf.push(VERSION);
    buf.extend_from_slice(&(n as u32).to_le_bytes());
    buf.extend_from_slice(&template.sample_rate_hz.to_le_bytes());

    for s in &template.samples {
        buf.extend_from_slice(&s.pressure.to_le_bytes());
    }
    for s in &template.samples {
        buf.extend_from_slice(&s.velocity.to_le_bytes());
    }
    for s in &template.samples {
        buf.extend_from_slice(&s.acceleration.to_le_bytes());
    }

    let mut pen_up_buf = vec![0u8; pen_up_bytes];
    for (i, s) in template.samples.iter().enumerate() {
        if s.pen_up {
            pen_up_buf[i / 8] |= 1 << (i % 8);
        }
    }
    buf.extend_from_slice(&pen_up_buf);
    buf
}
