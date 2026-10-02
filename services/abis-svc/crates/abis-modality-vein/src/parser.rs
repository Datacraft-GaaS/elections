// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use byteorder::{LittleEndian, ReadBytesExt};
use chrono::Utc;
use std::io::Cursor;
use uuid::Uuid;

use crate::domain::{BodySide, VeinTemplate, VeinType};

const MAGIC: [u8; 4] = [0x56, 0x4E, 0x49, 0x52];
// Header: 4 (magic) + 2 (width) + 2 (height) + 2 (wavelength) + 1 (vein_type) + 1 (body_side) = 12
const HEADER_LEN: usize = 12;

#[derive(Debug, thiserror::Error)]
pub enum VeinError {
    #[error("buffer too short: need {need}, got {got}")]
    BufferTooShort { need: usize, got: usize },
    #[error("invalid magic")]
    InvalidMagic,
    #[error("unknown vein type byte: {0}")]
    UnknownVeinType(u8),
    #[error("unknown body side byte: {0}")]
    UnknownBodySide(u8),
    #[error("pixel data length mismatch: expected {expected}, got {got}")]
    PixelDataMismatch { expected: usize, got: usize },
}

pub fn parse_nir_bitmap(data: &[u8]) -> Result<VeinTemplate, VeinError> {
    if data.len() < HEADER_LEN {
        return Err(VeinError::BufferTooShort {
            need: HEADER_LEN,
            got: data.len(),
        });
    }

    if &data[0..4] != MAGIC {
        return Err(VeinError::InvalidMagic);
    }

    let mut cur = Cursor::new(&data[4..]);

    let width = cur
        .read_u16::<LittleEndian>()
        .map_err(|_| VeinError::BufferTooShort {
            need: HEADER_LEN,
            got: data.len(),
        })?;
    let height = cur
        .read_u16::<LittleEndian>()
        .map_err(|_| VeinError::BufferTooShort {
            need: HEADER_LEN,
            got: data.len(),
        })?;
    let nir_wavelength_nm =
        cur.read_u16::<LittleEndian>()
            .map_err(|_| VeinError::BufferTooShort {
                need: HEADER_LEN,
                got: data.len(),
            })?;

    let vein_type_byte = cur.read_u8().map_err(|_| VeinError::BufferTooShort {
        need: HEADER_LEN,
        got: data.len(),
    })?;
    let body_side_byte = cur.read_u8().map_err(|_| VeinError::BufferTooShort {
        need: HEADER_LEN,
        got: data.len(),
    })?;

    let vein_type = match vein_type_byte {
        0 => VeinType::FingerVein,
        1 => VeinType::PalmVein,
        2 => VeinType::WristVein,
        b => return Err(VeinError::UnknownVeinType(b)),
    };

    let body_side = match body_side_byte {
        0 => BodySide::R,
        1 => BodySide::L,
        b => return Err(VeinError::UnknownBodySide(b)),
    };

    let expected = (width as usize) * (height as usize);
    let pixel_start = HEADER_LEN;
    let total_needed = pixel_start + expected;

    if data.len() < total_needed {
        return Err(VeinError::PixelDataMismatch {
            expected,
            got: data.len().saturating_sub(pixel_start),
        });
    }

    let actual_pixel_data = data.len() - pixel_start;
    if actual_pixel_data != expected {
        return Err(VeinError::PixelDataMismatch {
            expected,
            got: actual_pixel_data,
        });
    }

    let pattern_bitmap = data[pixel_start..pixel_start + expected].to_vec();

    Ok(VeinTemplate {
        template_id: Uuid::new_v4().to_string(),
        vein_type,
        body_side,
        nir_wavelength_nm,
        width,
        height,
        pattern_bitmap,
        captured_at: Utc::now(),
    })
}

pub fn serialize_nir_bitmap(template: &VeinTemplate) -> Vec<u8> {
    let pixel_count = (template.width as usize) * (template.height as usize);
    let mut buf = Vec::with_capacity(HEADER_LEN + pixel_count);

    buf.extend_from_slice(&MAGIC);

    buf.extend_from_slice(&template.width.to_le_bytes());
    buf.extend_from_slice(&template.height.to_le_bytes());
    buf.extend_from_slice(&template.nir_wavelength_nm.to_le_bytes());

    let vein_byte: u8 = match template.vein_type {
        VeinType::FingerVein => 0,
        VeinType::PalmVein => 1,
        VeinType::WristVein => 2,
    };
    buf.push(vein_byte);

    let side_byte: u8 = match template.body_side {
        BodySide::R => 0,
        BodySide::L => 1,
    };
    buf.push(side_byte);

    buf.extend_from_slice(&template.pattern_bitmap);
    buf
}
