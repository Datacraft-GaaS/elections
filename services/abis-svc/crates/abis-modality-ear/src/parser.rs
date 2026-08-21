use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use chrono::Utc;
use std::io::Cursor;
use uuid::Uuid;

use crate::{
    domain::{EarTemplate, Point2D, NUM_LANDMARKS},
    EarError,
};

const MAGIC: [u8; 4] = [0x45, 0x41, 0x52, 0x4C];
const EXPECTED_LEN: usize = 4 + 4 + 4 + NUM_LANDMARKS * 8;

pub fn parse_ear(data: &[u8]) -> Result<EarTemplate, EarError> {
    if data.len() < EXPECTED_LEN {
        return Err(EarError::BufferTooShort {
            need: EXPECTED_LEN,
            got: data.len(),
        });
    }
    if data[..4] != MAGIC {
        return Err(EarError::InvalidMagic);
    }

    let mut cur = Cursor::new(&data[4..]);

    let visibility_mask = cur.read_u32::<LittleEndian>()?;
    let rotation_angle_deg = cur.read_f32::<LittleEndian>()?;

    let mut landmarks = [Point2D { x: 0.0, y: 0.0 }; NUM_LANDMARKS];
    for lm in landmarks.iter_mut() {
        lm.x = cur.read_f32::<LittleEndian>()?;
        lm.y = cur.read_f32::<LittleEndian>()?;
    }

    Ok(EarTemplate {
        template_id: Uuid::new_v4().to_string(),
        landmarks,
        visibility_mask,
        rotation_angle_deg,
        captured_at: Utc::now(),
    })
}

pub fn serialize_ear(template: &EarTemplate) -> Vec<u8> {
    let mut buf = Vec::with_capacity(EXPECTED_LEN);
    buf.extend_from_slice(&MAGIC);
    buf.write_u32::<LittleEndian>(template.visibility_mask)
        .expect("vec write never fails");
    buf.write_f32::<LittleEndian>(template.rotation_angle_deg)
        .expect("vec write never fails");
    for lm in &template.landmarks {
        buf.write_f32::<LittleEndian>(lm.x)
            .expect("vec write never fails");
        buf.write_f32::<LittleEndian>(lm.y)
            .expect("vec write never fails");
    }
    buf
}
