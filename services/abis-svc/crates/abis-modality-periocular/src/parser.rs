use byteorder::{LittleEndian, ReadBytesExt};
use chrono::Utc;
use std::io::Cursor;
use uuid::Uuid;

use crate::domain::{LbpHistogram, PeriocularTemplate, PATCH_SIZE};

const MAGIC: [u8; 4] = [0x50, 0x45, 0x52, 0x49];
const PATCH_BYTES: usize = PATCH_SIZE * PATCH_SIZE;
const LBP_BINS: usize = 256;
// Magic(4) + patch(16384) + histogram(256*4=1024) + sharpness(4) + occlusion(4) = 17420
const TOTAL_LEN: usize = 4 + PATCH_BYTES + LBP_BINS * 4 + 4 + 4;

#[derive(Debug, thiserror::Error)]
pub enum PeriocularError {
    #[error("buffer too short: expected {expected}, got {got}")]
    BufferTooShort { expected: usize, got: usize },
    #[error("invalid magic")]
    InvalidMagic,
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn parse_periocular(data: &[u8]) -> Result<PeriocularTemplate, PeriocularError> {
    if data.len() != TOTAL_LEN {
        return Err(PeriocularError::BufferTooShort {
            expected: TOTAL_LEN,
            got: data.len(),
        });
    }

    if &data[0..4] != MAGIC {
        return Err(PeriocularError::InvalidMagic);
    }

    let patch = data[4..4 + PATCH_BYTES].to_vec();

    let hist_offset = 4 + PATCH_BYTES;
    let mut cur = Cursor::new(&data[hist_offset..]);
    let mut bins = vec![0f32; LBP_BINS];
    for b in bins.iter_mut() {
        *b = cur.read_f32::<LittleEndian>()?;
    }
    let sharpness = cur.read_f32::<LittleEndian>()?;
    let occlusion_ratio = cur.read_f32::<LittleEndian>()?;

    Ok(PeriocularTemplate {
        template_id: Uuid::new_v4().to_string(),
        patch,
        lbp_histogram: LbpHistogram { bins },
        sharpness,
        occlusion_ratio,
        captured_at: Utc::now(),
    })
}

pub fn serialize_periocular(template: &PeriocularTemplate) -> Vec<u8> {
    let mut buf = Vec::with_capacity(TOTAL_LEN);
    buf.extend_from_slice(&MAGIC);
    buf.extend_from_slice(&template.patch);
    for &b in &template.lbp_histogram.bins {
        buf.extend_from_slice(&b.to_le_bytes());
    }
    buf.extend_from_slice(&template.sharpness.to_le_bytes());
    buf.extend_from_slice(&template.occlusion_ratio.to_le_bytes());
    buf
}

/// Compute Local Binary Pattern histogram for a grayscale patch.
///
/// For each interior pixel, compare with 8 neighbors and produce an 8-bit code.
/// The histogram is normalized so bins sum to 1.0.
pub fn compute_lbp_histogram(patch: &[u8], width: usize, height: usize) -> LbpHistogram {
    let mut counts = vec![0u32; 256];

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let center = patch[y * width + x];
            let neighbors = [
                patch[(y - 1) * width + (x - 1)],
                patch[(y - 1) * width + x],
                patch[(y - 1) * width + (x + 1)],
                patch[y * width + (x + 1)],
                patch[(y + 1) * width + (x + 1)],
                patch[(y + 1) * width + x],
                patch[(y + 1) * width + (x - 1)],
                patch[y * width + (x - 1)],
            ];
            let mut code: u8 = 0;
            for (i, &n) in neighbors.iter().enumerate() {
                if n >= center {
                    code |= 1 << i;
                }
            }
            counts[code as usize] += 1;
        }
    }

    let total: u32 = counts.iter().sum();
    let bins = if total == 0 {
        vec![0.0f32; 256]
    } else {
        counts.iter().map(|&c| c as f32 / total as f32).collect()
    };

    LbpHistogram { bins }
}
