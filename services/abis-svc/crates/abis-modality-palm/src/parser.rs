//! ISO/IEC 19794-2 palm minutiae byte-layout parser and serializer.

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

use crate::domain::{Minutia, MinutiaType};

const MAGIC: [u8; 4] = [0x46, 0x4D, 0x52, 0x00];
const HEADER_LEN: usize = 16; // magic(4) + total_len(4) + img_x(2) + img_y(2) + x_res(2) + y_res(2) + views(1) + reserved(1)
const MINUTIA_RECORD_LEN: usize = 6;

/// Errors from palm ISO 19794-2 parsing.
#[derive(Debug, thiserror::Error)]
pub enum PalmError {
    #[error("buffer too short: need {need}, got {got}")]
    BufferTooShort { need: usize, got: usize },
    #[error("invalid magic bytes")]
    InvalidMagic,
    #[error("minutia index {idx} out of range")]
    MinutiaOutOfRange { idx: usize },
}

/// Parse a minimal ISO/IEC 19794-2 compliant byte buffer into minutiae.
///
/// Header layout:
/// - magic bytes: 4 bytes [0x46, 0x4D, 0x52, 0x00]
/// - total length: u32 big-endian
/// - image width:  u16 big-endian
/// - image height: u16 big-endian
/// - x resolution: u16 big-endian
/// - y resolution: u16 big-endian
/// - finger views: u8
/// - reserved:     u8
///
/// Per minutia record (6 bytes):
/// - type+x: u16 big-endian (top 2 bits = type, lower 14 bits = x)
/// - y:      u16 big-endian
/// - angle:  u8
/// - quality: u8
pub fn parse_iso19794_2(data: &[u8]) -> Result<Vec<Minutia>, PalmError> {
    if data.len() < HEADER_LEN {
        return Err(PalmError::BufferTooShort {
            need: HEADER_LEN,
            got: data.len(),
        });
    }
    if &data[..4] != &MAGIC {
        return Err(PalmError::InvalidMagic);
    }

    let mut cursor = Cursor::new(data);
    // skip magic
    cursor.set_position(4);
    let _total_len = cursor
        .read_u32::<BigEndian>()
        .map_err(|_| PalmError::BufferTooShort {
            need: 8,
            got: data.len(),
        })?;
    let _img_x = cursor
        .read_u16::<BigEndian>()
        .map_err(|_| PalmError::BufferTooShort {
            need: 10,
            got: data.len(),
        })?;
    let _img_y = cursor
        .read_u16::<BigEndian>()
        .map_err(|_| PalmError::BufferTooShort {
            need: 12,
            got: data.len(),
        })?;
    let _x_res = cursor
        .read_u16::<BigEndian>()
        .map_err(|_| PalmError::BufferTooShort {
            need: 14,
            got: data.len(),
        })?;
    let _y_res = cursor
        .read_u16::<BigEndian>()
        .map_err(|_| PalmError::BufferTooShort {
            need: 16,
            got: data.len(),
        })?;
    let views = cursor.read_u8().map_err(|_| PalmError::BufferTooShort {
        need: 17,
        got: data.len(),
    })?;
    let _reserved = cursor.read_u8().map_err(|_| PalmError::BufferTooShort {
        need: 18,
        got: data.len(),
    })?;

    let count = views as usize;
    let needed = HEADER_LEN + count * MINUTIA_RECORD_LEN;
    if data.len() < needed {
        return Err(PalmError::BufferTooShort {
            need: needed,
            got: data.len(),
        });
    }

    let mut minutiae = Vec::with_capacity(count);
    for idx in 0..count {
        let type_x = cursor
            .read_u16::<BigEndian>()
            .map_err(|_| PalmError::MinutiaOutOfRange { idx })?;
        let y = cursor
            .read_u16::<BigEndian>()
            .map_err(|_| PalmError::MinutiaOutOfRange { idx })?;
        let angle = cursor
            .read_u8()
            .map_err(|_| PalmError::MinutiaOutOfRange { idx })?;
        let quality = cursor
            .read_u8()
            .map_err(|_| PalmError::MinutiaOutOfRange { idx })?;

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

    Ok(minutiae)
}

/// Serialize minutiae back to an ISO/IEC 19794-2 compatible byte buffer.
pub fn serialize_iso19794_2(minutiae: &[Minutia], img_w: u16, img_h: u16) -> Vec<u8> {
    let count = minutiae.len();
    let total_len = (HEADER_LEN + count * MINUTIA_RECORD_LEN) as u32;
    let mut buf = Vec::with_capacity(total_len as usize);

    // magic
    buf.extend_from_slice(&MAGIC);
    // total length
    buf.write_u32::<BigEndian>(total_len)
        .expect("vec write cannot fail");
    // image dimensions
    buf.write_u16::<BigEndian>(img_w)
        .expect("vec write cannot fail");
    buf.write_u16::<BigEndian>(img_h)
        .expect("vec write cannot fail");
    // resolutions (500 ppi default)
    buf.write_u16::<BigEndian>(500)
        .expect("vec write cannot fail");
    buf.write_u16::<BigEndian>(500)
        .expect("vec write cannot fail");
    // views count (repurposed as minutia count)
    buf.push(count as u8);
    // reserved
    buf.push(0x00);

    for m in minutiae {
        let type_bits: u16 = match m.minutia_type {
            MinutiaType::RidgeEnding => 0,
            MinutiaType::Bifurcation => 1,
        };
        let type_x: u16 = (type_bits << 14) | (m.x & 0x3FFF);
        buf.write_u16::<BigEndian>(type_x)
            .expect("vec write cannot fail");
        buf.write_u16::<BigEndian>(m.y)
            .expect("vec write cannot fail");
        buf.push(m.angle);
        buf.push(m.quality);
    }

    buf
}
