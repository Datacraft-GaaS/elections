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

//! ANSI/NIST-ITL 1-2011 Type-13 latent record parser.

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::Utc;
use std::io::Cursor;

use crate::domain::{CustodyEvent, LatentSource, LatentTemplate, Minutia, MinutiaType};

const MAGIC: [u8; 3] = [0x41, 0x4E, 0x13];
const HEADER_LEN: usize = 3 + 4 + 1 + 1 + 2 + 2 + 2; // magic + rec_len + idc + imp + hll + vll + count
const MINUTIA_RECORD_LEN: usize = 6;

/// Errors from latent NIST Type-13 parsing.
#[derive(Debug, thiserror::Error)]
pub enum LatentError {
    #[error("buffer too short: need {need}, got {got}")]
    BufferTooShort { need: usize, got: usize },
    #[error("invalid magic bytes")]
    InvalidMagic,
    #[error("minutia index {idx} out of range")]
    MinutiaOutOfRange { idx: usize },
}

/// Parse a simplified NIST Type-13 binary record into a [LatentTemplate].
///
/// Header layout:
/// - magic:          3 bytes [0x41, 0x4E, 0x13]
/// - record length:  u32 big-endian
/// - IDC:            u8
/// - IMP:            u8 (impression type)
/// - HLL:            u16 big-endian (width)
/// - VLL:            u16 big-endian (height)
/// - minutiae count: u16 big-endian
/// - minutia records: 6 bytes each (type+x u16, y u16, angle u8, quality u8)
pub fn parse_nist_type13(data: &[u8]) -> Result<LatentTemplate, LatentError> {
    if data.len() < HEADER_LEN {
        return Err(LatentError::BufferTooShort {
            need: HEADER_LEN,
            got: data.len(),
        });
    }
    if data[..3] != MAGIC {
        return Err(LatentError::InvalidMagic);
    }

    let mut cursor = Cursor::new(data);
    cursor.set_position(3);
    let _rec_len = cursor
        .read_u32::<BigEndian>()
        .map_err(|_| LatentError::BufferTooShort {
            need: 7,
            got: data.len(),
        })?;
    let _idc = cursor.read_u8().map_err(|_| LatentError::BufferTooShort {
        need: 8,
        got: data.len(),
    })?;
    let _imp = cursor.read_u8().map_err(|_| LatentError::BufferTooShort {
        need: 9,
        got: data.len(),
    })?;
    let _hll = cursor
        .read_u16::<BigEndian>()
        .map_err(|_| LatentError::BufferTooShort {
            need: 11,
            got: data.len(),
        })?;
    let _vll = cursor
        .read_u16::<BigEndian>()
        .map_err(|_| LatentError::BufferTooShort {
            need: 13,
            got: data.len(),
        })?;
    let count = cursor
        .read_u16::<BigEndian>()
        .map_err(|_| LatentError::BufferTooShort {
            need: 15,
            got: data.len(),
        })? as usize;

    let needed = HEADER_LEN + count * MINUTIA_RECORD_LEN;
    if data.len() < needed {
        return Err(LatentError::BufferTooShort {
            need: needed,
            got: data.len(),
        });
    }

    let mut minutiae = Vec::with_capacity(count);
    for idx in 0..count {
        let type_x = cursor
            .read_u16::<BigEndian>()
            .map_err(|_| LatentError::MinutiaOutOfRange { idx })?;
        let y = cursor
            .read_u16::<BigEndian>()
            .map_err(|_| LatentError::MinutiaOutOfRange { idx })?;
        let angle = cursor
            .read_u8()
            .map_err(|_| LatentError::MinutiaOutOfRange { idx })?;
        let quality = cursor
            .read_u8()
            .map_err(|_| LatentError::MinutiaOutOfRange { idx })?;

        let minutia_type = if (type_x >> 14) & 0x01 == 1 {
            MinutiaType::Bifurcation
        } else {
            MinutiaType::RidgeEnding
        };
        let x = type_x & 0x3FFF;

        minutiae.push(Minutia {
            x,
            y,
            angle,
            quality,
            minutia_type,
        });
    }

    Ok(LatentTemplate {
        template_id: uuid::Uuid::new_v4().to_string(),
        source: LatentSource::Digital,
        lift_id: None,
        case_ref: None,
        distortion_score: 0.0,
        usable_ridge_pct: 1.0,
        examiner_id: None,
        chain_of_custody: vec![],
        minutiae,
        captured_at: Utc::now(),
    })
}

/// Serialize a [LatentTemplate] back to a NIST Type-13 binary record.
pub fn serialize_nist_type13(template: &LatentTemplate) -> Vec<u8> {
    let count = template.minutiae.len() as u16;
    let rec_len = (HEADER_LEN + template.minutiae.len() * MINUTIA_RECORD_LEN) as u32;
    let mut buf = Vec::with_capacity(rec_len as usize);

    buf.extend_from_slice(&MAGIC);
    buf.write_u32::<BigEndian>(rec_len)
        .expect("vec write cannot fail");
    buf.push(0x00); // IDC
    buf.push(0x04); // IMP: latent lift
    buf.write_u16::<BigEndian>(500)
        .expect("vec write cannot fail"); // HLL
    buf.write_u16::<BigEndian>(400)
        .expect("vec write cannot fail"); // VLL
    buf.write_u16::<BigEndian>(count)
        .expect("vec write cannot fail");

    for m in &template.minutiae {
        let type_bits: u16 = match m.minutia_type {
            MinutiaType::RidgeEnding => 0,
            MinutiaType::Bifurcation => 1,
        };
        let type_x = (type_bits << 14) | (m.x & 0x3FFF);
        buf.write_u16::<BigEndian>(type_x)
            .expect("vec write cannot fail");
        buf.write_u16::<BigEndian>(m.y)
            .expect("vec write cannot fail");
        buf.push(m.angle);
        buf.push(m.quality);
    }

    buf
}

/// Append a custody event to a template, maintaining chronological order.
pub fn append_custody_event(template: &mut LatentTemplate, event: CustodyEvent) {
    template.chain_of_custody.push(event);
}
